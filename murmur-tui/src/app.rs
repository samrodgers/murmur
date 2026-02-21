//! Application state and input handling for the Murmur TUI.

use murmur_core::event::builder::EventBuilder;
use murmur_core::event::types::{Event, EventId};
use murmur_core::event::validate::validate_event;
use murmur_core::identity::Identity;
use murmur_core::network::node::{MurmurNode, NodeEvent};
use murmur_core::storage::db::Database;

use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Which screen the TUI is showing.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Timeline,
    Compose,
    Thread(EventId),
    Profile,
}

/// Input mode for the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    /// Navigating the UI (vim-style keys).
    Normal,
    /// Typing into an input field.
    Editing,
}

/// Main application state.
pub struct App {
    /// The local user's identity.
    pub identity: Identity,
    /// Local database.
    pub db: Database,
    /// Network node handle.
    pub node: MurmurNode,
    /// Receiver for network events.
    pub net_rx: mpsc::Receiver<NodeEvent>,
    /// Current screen.
    pub screen: Screen,
    /// Current input mode.
    pub input_mode: InputMode,
    /// Text input buffer (for composing posts).
    pub input: String,
    /// Cursor position in the input buffer.
    pub cursor_pos: usize,
    /// Timeline of events (posts + replies), ordered by time desc.
    pub timeline: Vec<Event>,
    /// Currently selected index in the timeline.
    pub selected: usize,
    /// Number of connected peers.
    pub peer_count: usize,
    /// Status messages to display.
    pub status_message: Option<String>,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Data directory path.
    pub data_dir: PathBuf,
    /// Thread being viewed: (root, replies).
    pub current_thread: Option<(Event, Vec<Event>)>,
}

impl App {
    /// Create a new App instance.
    pub async fn new(data_dir: PathBuf, listen_port: u16) -> anyhow::Result<Self> {
        // Ensure data directory exists
        std::fs::create_dir_all(&data_dir)?;

        // Load or generate identity
        let identity_path = data_dir.join("identity.json");
        let identity = if identity_path.exists() {
            info!("Loading existing identity");
            // Phase 1: No passphrase protection
            Identity::load(&identity_path, "")?
        } else {
            info!("Generating new identity");
            let id = Identity::generate();
            id.save(&identity_path, "")?;
            id
        };

        info!(pubkey = %identity.pubkey_short(), "Identity loaded");

        // Open database
        let db_path = data_dir.join("murmur.db");
        let db = Database::open(&db_path)?;

        // Start network node
        let (node, net_rx) = MurmurNode::start(listen_port).await?;
        info!(peer_id = %node.peer_id(), "Network node started");

        // Load existing timeline
        let timeline = db.get_posts(100)?;

        Ok(Self {
            identity,
            db,
            node,
            net_rx,
            screen: Screen::Timeline,
            input_mode: InputMode::Normal,
            input: String::new(),
            cursor_pos: 0,
            timeline,
            selected: 0,
            peer_count: 0,
            status_message: Some("Welcome to Murmur! Press 'n' to compose, 'q' to quit.".into()),
            should_quit: false,
            data_dir,
            current_thread: None,
        })
    }

    /// Process any pending network events.
    pub async fn process_network_events(&mut self) {
        while let Ok(event) = self.net_rx.try_recv() {
            match event {
                NodeEvent::PeerDiscovered(peer_id) => {
                    self.status_message = Some(format!("Peer discovered: {}", peer_id));

                    // Push our own events to the new peer
                    match self.db.get_own_events() {
                        Ok(events) => {
                            for event in events {
                                if let Err(e) = self.node.broadcast_event(event).await {
                                    warn!("Failed to push event to peer: {e}");
                                }
                            }
                        }
                        Err(e) => warn!("Failed to get own events for sync: {e}"),
                    }
                }
                NodeEvent::PeerLost(peer_id) => {
                    self.status_message = Some(format!("Peer disconnected: {}", peer_id));
                }
                NodeEvent::PeerCountChanged(count) => {
                    self.peer_count = count;
                }
                NodeEvent::EventReceived(event) => {
                    self.handle_incoming_event(event);
                }
                NodeEvent::EventsReceived(events) => {
                    for event in events {
                        self.handle_incoming_event(event);
                    }
                }
            }
        }
    }

    /// Handle an incoming event from the network.
    fn handle_incoming_event(&mut self, event: Event) {
        // Validate the event
        if let Err(e) = validate_event(&event) {
            warn!(id = %hex::encode(event.id), "Rejected invalid event: {e}");
            return;
        }

        // Check if we already have it
        match self.db.has_event(&event.id) {
            Ok(true) => return,
            Err(e) => {
                warn!("DB error checking event: {e}");
                return;
            }
            Ok(false) => {}
        }

        // Store it
        if let Err(e) = self.db.cache_event(&event) {
            warn!("Failed to store event: {e}");
            return;
        }

        info!(
            id = %hex::encode(event.id),
            kind = ?event.kind,
            author = %event.pubkey.short(),
            "Stored new event from network"
        );

        // Refresh timeline
        self.refresh_timeline();
    }

    /// Compose and publish a new post.
    pub async fn publish_post(&mut self, content: &str) -> anyhow::Result<()> {
        let event = EventBuilder::post(content).sign(&self.identity);

        // Store locally
        self.db.store_own_event(&event)?;

        // Broadcast to network
        self.node.broadcast_event(event).await?;

        // Refresh timeline
        self.refresh_timeline();
        self.status_message = Some("Post published!".into());

        Ok(())
    }

    /// Compose and publish a reply.
    pub async fn publish_reply(
        &mut self,
        content: &str,
        parent_id: EventId,
    ) -> anyhow::Result<()> {
        // Determine thread root
        let root_id = match self.db.get_event(&parent_id)? {
            Some(parent) => parent.thread_root_id().copied().unwrap_or(parent_id),
            None => parent_id,
        };

        let event = if root_id == parent_id {
            EventBuilder::reply(content, parent_id).sign(&self.identity)
        } else {
            EventBuilder::reply_in_thread(content, parent_id, root_id).sign(&self.identity)
        };

        self.db.store_own_event(&event)?;
        self.node.broadcast_event(event).await?;
        self.refresh_timeline();
        self.status_message = Some("Reply published!".into());

        Ok(())
    }

    /// Refresh the timeline from the database.
    pub fn refresh_timeline(&mut self) {
        match self.db.get_posts(100) {
            Ok(events) => {
                self.timeline = events;
                if self.selected >= self.timeline.len() && !self.timeline.is_empty() {
                    self.selected = self.timeline.len() - 1;
                }
            }
            Err(e) => {
                warn!("Failed to refresh timeline: {e}");
            }
        }
    }

    /// Load a thread for viewing.
    pub fn load_thread(&mut self, root_id: &EventId) {
        match self.db.get_thread(root_id) {
            Ok((root, replies)) => {
                if let Some(root) = root {
                    self.current_thread = Some((root, replies));
                    self.screen = Screen::Thread(*root_id);
                    self.selected = 0;
                } else {
                    self.status_message = Some("Thread root not found".into());
                }
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to load thread: {e}"));
            }
        }
    }

    /// Get the currently selected event on the timeline.
    pub fn selected_event(&self) -> Option<&Event> {
        self.timeline.get(self.selected)
    }

    /// Move selection up.
    pub fn select_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down.
    pub fn select_down(&mut self) {
        match &self.screen {
            Screen::Timeline => {
                if self.selected < self.timeline.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            Screen::Thread(_) => {
                if let Some((_, replies)) = &self.current_thread {
                    if self.selected < replies.len() {
                        self.selected += 1;
                    }
                }
            }
            _ => {}
        }
    }
}
