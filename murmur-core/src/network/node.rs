//! Local node lifecycle — start, discover peers, exchange events.
//!
//! Phase 1 uses mDNS for local network discovery and gossipsub
//! for broadcasting new events. Request-response handles direct
//! queries (get event, get thread, sync).

use crate::event::types::Event;
use crate::network::protocol::{MurmurRequest, MurmurResponse};

use futures::StreamExt;
use libp2p::{
    gossipsub, mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, SwarmBuilder,
};

use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Protocol identifier for Murmur request-response.
const MURMUR_PROTOCOL: StreamProtocol = StreamProtocol::new("/murmur/sync/1.0.0");

/// Gossipsub topic for broadcasting new events.
const EVENTS_TOPIC: &str = "murmur-events";

/// Commands that can be sent to the network node from the application.
#[derive(Debug)]
pub enum NodeCommand {
    /// Broadcast a new event to all connected peers.
    BroadcastEvent(Event),
    /// Request sync from a specific peer.
    RequestSync { peer: PeerId, since: i64 },
    /// Get the current peer list.
    GetPeers(mpsc::Sender<Vec<PeerId>>),
    /// Shutdown the node.
    Shutdown,
}

/// Events emitted by the network node to the application.
#[derive(Debug, Clone)]
pub enum NodeEvent {
    /// A new peer was discovered.
    PeerDiscovered(PeerId),
    /// A peer disconnected.
    PeerLost(PeerId),
    /// Received an event from the network.
    EventReceived(Event),
    /// Received a batch of events (from sync).
    EventsReceived(Vec<Event>),
    /// Number of connected peers changed.
    PeerCountChanged(usize),
}

/// The Murmur network node.
pub struct MurmurNode {
    /// Channel to send commands to the node task.
    cmd_tx: mpsc::Sender<NodeCommand>,
    /// Our local peer ID.
    peer_id: PeerId,
}

/// Composed network behaviour for Murmur.
#[derive(NetworkBehaviour)]
struct MurmurBehaviour {
    /// Gossipsub for broadcasting events to subscribed peers.
    gossipsub: gossipsub::Behaviour,
    /// mDNS for local network peer discovery (Phase 1).
    mdns: mdns::tokio::Behaviour,
    /// Request-response for direct peer queries.
    request_response: request_response::cbor::Behaviour<MurmurRequest, MurmurResponse>,
}

impl MurmurNode {
    /// Start a new Murmur node. Returns the node handle and a receiver
    /// for network events.
    pub async fn start(
        listen_port: u16,
    ) -> anyhow::Result<(Self, mpsc::Receiver<NodeEvent>)> {
        let (cmd_tx, cmd_rx) = mpsc::channel(256);
        let (event_tx, event_rx) = mpsc::channel(256);

        // Build the swarm
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                // Gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .build()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                // mDNS for local peer discovery
                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;

                // Request-response for direct queries
                let request_response = request_response::cbor::Behaviour::new(
                    [(MURMUR_PROTOCOL, ProtocolSupport::Full)],
                    request_response::Config::default(),
                );

                Ok(MurmurBehaviour {
                    gossipsub,
                    mdns,
                    request_response,
                })
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(300)))
            .build();

        let peer_id = *swarm.local_peer_id();
        info!(%peer_id, "Starting Murmur node");

        // Subscribe to the events topic
        let topic = gossipsub::IdentTopic::new(EVENTS_TOPIC);
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        // Listen on TCP
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", listen_port).parse()?;
        swarm.listen_on(listen_addr)?;

        // Spawn the event loop
        tokio::spawn(run_event_loop(swarm, cmd_rx, event_tx, topic));

        Ok((Self { cmd_tx, peer_id }, event_rx))
    }

    /// Get our local peer ID.
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Broadcast a new event to all connected peers via gossipsub.
    pub async fn broadcast_event(&self, event: Event) -> anyhow::Result<()> {
        self.cmd_tx
            .send(NodeCommand::BroadcastEvent(event))
            .await
            .map_err(|_| anyhow::anyhow!("Node task has shut down"))?;
        Ok(())
    }

    /// Get the current list of connected peers.
    pub async fn get_peers(&self) -> anyhow::Result<Vec<PeerId>> {
        let (tx, mut rx) = mpsc::channel(1);
        self.cmd_tx
            .send(NodeCommand::GetPeers(tx))
            .await
            .map_err(|_| anyhow::anyhow!("Node task has shut down"))?;
        rx.recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to get peer list"))
    }

    /// Shut down the node.
    pub async fn shutdown(&self) -> anyhow::Result<()> {
        let _ = self.cmd_tx.send(NodeCommand::Shutdown).await;
        Ok(())
    }
}

/// Main event loop for the swarm.
async fn run_event_loop(
    mut swarm: libp2p::Swarm<MurmurBehaviour>,
    mut cmd_rx: mpsc::Receiver<NodeCommand>,
    event_tx: mpsc::Sender<NodeEvent>,
    topic: gossipsub::IdentTopic,
) {
    let mut known_peers: HashSet<PeerId> = HashSet::new();

    loop {
        tokio::select! {
            // Handle commands from the application
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(NodeCommand::BroadcastEvent(event)) => {
                        match serde_json::to_vec(&event) {
                            Ok(data) => {
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), data) {
                                    warn!("Failed to broadcast event: {e}");
                                }
                            }
                            Err(e) => warn!("Failed to serialize event: {e}"),
                        }
                    }
                    Some(NodeCommand::RequestSync { peer, since }) => {
                        let req = MurmurRequest::SyncRequest { since };
                        swarm.behaviour_mut().request_response.send_request(&peer, req);
                    }
                    Some(NodeCommand::GetPeers(reply)) => {
                        let peers: Vec<PeerId> = known_peers.iter().copied().collect();
                        let _ = reply.send(peers).await;
                    }
                    Some(NodeCommand::Shutdown) | None => {
                        info!("Node shutting down");
                        return;
                    }
                }
            }

            // Handle swarm events
            event = swarm.select_next_some() => {
                match event {
                    // mDNS: peer discovered
                    SwarmEvent::Behaviour(MurmurBehaviourEvent::Mdns(
                        mdns::Event::Discovered(peers)
                    )) => {
                        for (peer_id, addr) in peers {
                            if known_peers.insert(peer_id) {
                                info!(%peer_id, %addr, "Discovered peer via mDNS");
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                let _ = event_tx.send(NodeEvent::PeerDiscovered(peer_id)).await;
                                let _ = event_tx.send(NodeEvent::PeerCountChanged(known_peers.len())).await;

                                // Request sync from new peer
                                let req = MurmurRequest::SyncRequest { since: 0 };
                                swarm.behaviour_mut().request_response.send_request(&peer_id, req);
                            }
                        }
                    }

                    // mDNS: peer expired
                    SwarmEvent::Behaviour(MurmurBehaviourEvent::Mdns(
                        mdns::Event::Expired(peers)
                    )) => {
                        for (peer_id, _) in peers {
                            if known_peers.remove(&peer_id) {
                                info!(%peer_id, "Peer expired");
                                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                let _ = event_tx.send(NodeEvent::PeerLost(peer_id)).await;
                                let _ = event_tx.send(NodeEvent::PeerCountChanged(known_peers.len())).await;
                            }
                        }
                    }

                    // Gossipsub: received a broadcast message
                    SwarmEvent::Behaviour(MurmurBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { message, .. }
                    )) => {
                        match serde_json::from_slice::<Event>(&message.data) {
                            Ok(event) => {
                                debug!(id = %hex::encode(event.id), "Received broadcast event");
                                let _ = event_tx.send(NodeEvent::EventReceived(event)).await;
                            }
                            Err(e) => {
                                warn!("Failed to deserialize broadcast event: {e}");
                            }
                        }
                    }

                    // Request-response: incoming request
                    SwarmEvent::Behaviour(MurmurBehaviourEvent::RequestResponse(
                        request_response::Event::Message {
                            peer,
                            message: request_response::Message::Request { request, channel, .. },
                            ..
                        }
                    )) => {
                        debug!(%peer, "Received request: {:?}", std::mem::discriminant(&request));
                        // For Phase 1, we handle sync requests by sending back all our events
                        // The actual handling is done by the application layer
                        let response = match request {
                            MurmurRequest::SyncRequest { .. } => {
                                // We can't access the DB here, so we reply with empty
                                // and rely on gossipsub for event distribution.
                                // A more complete implementation would use a shared DB handle.
                                MurmurResponse::Events(vec![])
                            }
                            MurmurRequest::GetEvent { .. } => MurmurResponse::NotFound,
                            MurmurRequest::PushEvent { event } => {
                                let _ = event_tx.send(NodeEvent::EventReceived(event)).await;
                                MurmurResponse::Accepted
                            }
                            _ => MurmurResponse::Error("Not implemented yet".into()),
                        };
                        let _ = swarm.behaviour_mut().request_response.send_response(channel, response);
                    }

                    // Request-response: received response
                    SwarmEvent::Behaviour(MurmurBehaviourEvent::RequestResponse(
                        request_response::Event::Message {
                            message: request_response::Message::Response { response, .. },
                            ..
                        }
                    )) => {
                        match response {
                            MurmurResponse::Events(events) if !events.is_empty() => {
                                debug!("Received {} events from sync", events.len());
                                let _ = event_tx.send(NodeEvent::EventsReceived(events)).await;
                            }
                            MurmurResponse::Event(event) => {
                                let _ = event_tx.send(NodeEvent::EventReceived(event)).await;
                            }
                            _ => {}
                        }
                    }

                    // New listening address
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!(%address, "Listening on");
                    }

                    _ => {}
                }
            }
        }
    }
}
