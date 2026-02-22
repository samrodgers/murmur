//! Application state management — holds the identity, database, and network node.

use murmur_core::event::validate::validate_event;
use murmur_core::identity::Identity;
use murmur_core::network::node::{MurmurNode, NodeEvent};
use murmur_core::storage::Database;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tracing::{info, warn};

pub struct AppState {
    pub identity: Option<Identity>,
    pub db: Database,
    pub node: Option<MurmurNode>,
    pub net_rx: Option<mpsc::Receiver<NodeEvent>>,
    pub data_dir: PathBuf,
    pub peer_count: u32,
    pub display_name: String,
    pub bio: String,
    pub cache_limit_mb: u32,
}

impl AppState {
    pub async fn new(data_dir: PathBuf) -> anyhow::Result<Self> {
        let db_path = data_dir.join("murmur.db");
        let db = Database::open(&db_path)?;

        // Try to load existing identity
        let identity_path = data_dir.join("identity.json");
        let identity = if identity_path.exists() {
            match Identity::load(&identity_path, "") {
                Ok(id) => {
                    info!(pubkey = %id.pubkey_short(), "Loaded existing identity");
                    Some(id)
                }
                Err(e) => {
                    warn!("Failed to load identity: {e}");
                    None
                }
            }
        } else {
            None
        };

        // Load profile metadata
        let profile_path = data_dir.join("profile.json");
        let (display_name, bio) = if profile_path.exists() {
            match std::fs::read_to_string(&profile_path) {
                Ok(json) => {
                    let profile: serde_json::Value = serde_json::from_str(&json).unwrap_or_default();
                    (
                        profile["name"].as_str().unwrap_or("").to_string(),
                        profile["bio"].as_str().unwrap_or("").to_string(),
                    )
                }
                Err(_) => (String::new(), String::new()),
            }
        } else {
            (String::new(), String::new())
        };

        Ok(Self {
            identity,
            db,
            node: None,
            net_rx: None,
            data_dir,
            peer_count: 0,
            display_name,
            bio,
            cache_limit_mb: 500,
        })
    }

    pub fn has_identity(&self) -> bool {
        self.identity.is_some()
    }

    pub fn own_pubkey_hex(&self) -> String {
        self.identity
            .as_ref()
            .map(|id| id.pubkey_hex())
            .unwrap_or_default()
    }

    pub fn save_profile(&self) -> anyhow::Result<()> {
        let profile = serde_json::json!({
            "name": self.display_name,
            "bio": self.bio,
        });
        let path = self.data_dir.join("profile.json");
        std::fs::write(path, serde_json::to_string_pretty(&profile)?)?;
        Ok(())
    }

    pub async fn start_network(&mut self) -> anyhow::Result<()> {
        if self.node.is_some() {
            return Ok(());
        }
        let (node, net_rx) = MurmurNode::start(9000).await?;
        info!(peer_id = %node.peer_id(), "Network node started");
        self.node = Some(node);
        self.net_rx = Some(net_rx);
        Ok(())
    }
}

/// Background task that listens for network events and forwards them to the frontend.
pub async fn run_network_listener(app_handle: AppHandle) {
    // Take the receiver out of the state
    let mut net_rx = {
        let state = app_handle.state::<Mutex<AppState>>();
        let mut state = state.lock().unwrap();
        match state.net_rx.take() {
            Some(rx) => rx,
            None => return,
        }
    };

    loop {
        match net_rx.recv().await {
            Some(NodeEvent::PeerDiscovered(peer_id)) => {
                let count = {
                    let state = app_handle.state::<Mutex<AppState>>();
                    let mut state = state.lock().unwrap();
                    state.peer_count += 1;
                    state.peer_count
                };
                let _ = app_handle.emit("peer-changed", serde_json::json!({
                    "connected_count": count
                }));
                info!(%peer_id, "Peer discovered");

                // Push our events to the new peer via gossipsub
                {
                    let state = app_handle.state::<Mutex<AppState>>();
                    let state = state.lock().unwrap();
                    let events = state.db.get_own_events().unwrap_or_default();
                    if let Some(node) = &state.node {
                        for event in events {
                            let _ = node.try_broadcast_event(event);
                        }
                    }
                }
            }
            Some(NodeEvent::PeerLost(peer_id)) => {
                let count = {
                    let state = app_handle.state::<Mutex<AppState>>();
                    let mut state = state.lock().unwrap();
                    state.peer_count = state.peer_count.saturating_sub(1);
                    state.peer_count
                };
                let _ = app_handle.emit("peer-changed", serde_json::json!({
                    "connected_count": count
                }));
                info!(%peer_id, "Peer lost");
            }
            Some(NodeEvent::PeerCountChanged(count)) => {
                {
                    let state = app_handle.state::<Mutex<AppState>>();
                    let mut state = state.lock().unwrap();
                    state.peer_count = count as u32;
                }
                let _ = app_handle.emit("peer-changed", serde_json::json!({
                    "connected_count": count
                }));
            }
            Some(NodeEvent::EventReceived(event)) => {
                if let Err(e) = validate_event(&event) {
                    warn!("Rejected invalid event: {e}");
                    continue;
                }
                let post = {
                    let state = app_handle.state::<Mutex<AppState>>();
                    let state = state.lock().unwrap();
                    if state.db.has_event(&event.id).unwrap_or(true) {
                        continue;
                    }
                    let _ = state.db.insert_event(&event, false);
                    let pubkey = state.own_pubkey_hex();
                    crate::types::event_to_post(&event, &pubkey, &state.db)
                };
                let _ = app_handle.emit("new-post", &post);
            }
            Some(NodeEvent::EventsReceived(events)) => {
                for event in events {
                    if let Err(e) = validate_event(&event) {
                        warn!("Rejected invalid event: {e}");
                        continue;
                    }
                    let post = {
                        let state = app_handle.state::<Mutex<AppState>>();
                        let state = state.lock().unwrap();
                        if state.db.has_event(&event.id).unwrap_or(true) {
                            continue;
                        }
                        let _ = state.db.insert_event(&event, false);
                        let pubkey = state.own_pubkey_hex();
                        crate::types::event_to_post(&event, &pubkey, &state.db)
                    };
                    let _ = app_handle.emit("new-post", &post);
                }
            }
            None => break,
        }
    }
}
