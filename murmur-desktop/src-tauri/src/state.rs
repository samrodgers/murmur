//! Application state shared across Tauri commands.

use murmur_core::identity::Identity;
use murmur_core::network::node::{MurmurNode, NodeEvent};
use murmur_core::storage::db::Database;

use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::info;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub identity: Identity,
    pub db: Database,
    pub node: MurmurNode,
    pub peer_count: usize,
    pub data_dir: PathBuf,
}

impl AppState {
    /// Initialize the application state. Returns the state and a channel
    /// receiver for network events (to be processed by the event loop).
    pub async fn new() -> anyhow::Result<(Self, mpsc::Receiver<NodeEvent>)> {
        let data_dir = Self::resolve_data_dir();
        std::fs::create_dir_all(&data_dir)?;

        // Load or generate identity
        let identity_path = data_dir.join("identity.json");
        let identity = if identity_path.exists() {
            info!("Loading existing identity");
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
        let listen_port: u16 = std::env::var("MURMUR_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(9000);

        let (node, net_rx) = MurmurNode::start(listen_port).await?;
        info!(peer_id = %node.peer_id(), "Network node started on port {}", listen_port);

        // Push own events on startup so new peers can sync
        if let Ok(events) = db.get_own_events() {
            for event in &events {
                if let Err(e) = node.broadcast_event(event.clone()).await {
                    tracing::warn!("Failed to broadcast own event on startup: {e}");
                }
            }
        }

        Ok((
            Self {
                identity,
                db,
                node,
                peer_count: 0,
                data_dir,
            },
            net_rx,
        ))
    }

    fn resolve_data_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("MURMUR_DATA") {
            PathBuf::from(dir)
        } else {
            dirs_or_default()
        }
    }
}

/// Platform-appropriate default data directory.
fn dirs_or_default() -> PathBuf {
    // Try standard app data locations
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(home).join(".murmur");
        return path;
    }
    PathBuf::from("./murmur-data")
}
