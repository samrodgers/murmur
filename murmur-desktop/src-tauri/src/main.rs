// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;

use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use tracing::info;

/// Serializable event for the frontend.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct EventDto {
    pub id: String,
    pub pubkey: String,
    pub pubkey_short: String,
    pub created_at: i64,
    pub kind: u8,
    pub kind_label: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub thread_root_id: Option<String>,
    pub is_own: bool,
    pub time_ago: String,
}

/// Convert a core Event to a frontend DTO.
fn event_to_dto(event: &murmur_core::event::types::Event, own_pubkey: &str) -> EventDto {
    let id = hex::encode(event.id);
    let pubkey = event.pubkey.to_hex();
    let is_own = pubkey == own_pubkey;

    EventDto {
        id,
        pubkey: pubkey.clone(),
        pubkey_short: event.pubkey.short(),
        created_at: event.created_at,
        kind: event.kind.as_u8(),
        kind_label: match event.kind {
            murmur_core::event::types::EventKind::Post => "post".into(),
            murmur_core::event::types::EventKind::Reply => "reply".into(),
            murmur_core::event::types::EventKind::Repost => "repost".into(),
            murmur_core::event::types::EventKind::Reaction => "reaction".into(),
            murmur_core::event::types::EventKind::Correction => "correction".into(),
            _ => "other".into(),
        },
        content: event.content.clone(),
        parent_id: event.parent_id().map(hex::encode),
        thread_root_id: event.thread_root_id().map(hex::encode),
        is_own,
        time_ago: format_time_ago(event.created_at),
    }
}

fn format_time_ago(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;
    if diff < 60 {
        "just now".into()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

// ── Tauri Commands ───────────────────────────────────────────

#[tauri::command]
async fn get_identity(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<serde_json::Value, String> {
    let app = state.lock().await;
    Ok(serde_json::json!({
        "pubkey": app.identity.pubkey_hex(),
        "pubkey_short": app.identity.pubkey_short(),
    }))
}

#[tauri::command]
async fn get_timeline(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<EventDto>, String> {
    let app = state.lock().await;
    let own_pubkey = app.identity.pubkey_hex();
    let events = app.db.get_timeline(100).map_err(|e| e.to_string())?;
    Ok(events.iter().map(|e| event_to_dto(e, &own_pubkey)).collect())
}

#[tauri::command]
async fn get_posts(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<EventDto>, String> {
    let app = state.lock().await;
    let own_pubkey = app.identity.pubkey_hex();
    let events = app.db.get_posts(100).map_err(|e| e.to_string())?;
    Ok(events.iter().map(|e| event_to_dto(e, &own_pubkey)).collect())
}

#[tauri::command]
async fn get_thread(
    event_id: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    let app = state.lock().await;
    let own_pubkey = app.identity.pubkey_hex();

    let id_bytes: [u8; 32] = hex::decode(&event_id)
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|_| "Invalid event ID length".to_string())?;

    let (root, replies) = app.db.get_thread(&id_bytes).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "root": root.map(|r| event_to_dto(&r, &own_pubkey)),
        "replies": replies.iter().map(|r| event_to_dto(r, &own_pubkey)).collect::<Vec<_>>(),
    }))
}

#[tauri::command]
async fn publish_post(
    content: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<EventDto, String> {
    let app = state.lock().await;
    let own_pubkey = app.identity.pubkey_hex();

    let event = murmur_core::event::builder::EventBuilder::post(&content)
        .sign(&app.identity);

    app.db.store_own_event(&event).map_err(|e| e.to_string())?;
    app.node.broadcast_event(event.clone()).await.map_err(|e| e.to_string())?;

    info!(id = %hex::encode(event.id), "Published post");
    Ok(event_to_dto(&event, &own_pubkey))
}

#[tauri::command]
async fn publish_reply(
    content: String,
    parent_id: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<EventDto, String> {
    let app = state.lock().await;
    let own_pubkey = app.identity.pubkey_hex();

    let parent_bytes: [u8; 32] = hex::decode(&parent_id)
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|_| "Invalid event ID length".to_string())?;

    // Determine thread root
    let root_id = match app.db.get_event(&parent_bytes).map_err(|e| e.to_string())? {
        Some(parent) => parent.thread_root_id().copied().unwrap_or(parent_bytes),
        None => parent_bytes,
    };

    let event = if root_id == parent_bytes {
        murmur_core::event::builder::EventBuilder::reply(&content, parent_bytes)
            .sign(&app.identity)
    } else {
        murmur_core::event::builder::EventBuilder::reply_in_thread(&content, parent_bytes, root_id)
            .sign(&app.identity)
    };

    app.db.store_own_event(&event).map_err(|e| e.to_string())?;
    app.node.broadcast_event(event.clone()).await.map_err(|e| e.to_string())?;

    info!(id = %hex::encode(event.id), "Published reply");
    Ok(event_to_dto(&event, &own_pubkey))
}

#[tauri::command]
async fn get_stats(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<serde_json::Value, String> {
    let app = state.lock().await;
    let total = app.db.total_event_count().map_err(|e| e.to_string())?;
    let cached = app.db.cached_event_count().map_err(|e| e.to_string())?;
    let peers = app.peer_count;

    Ok(serde_json::json!({
        "total_events": total,
        "cached_events": cached,
        "own_events": total - cached,
        "peer_count": peers,
    }))
}

#[tauri::command]
async fn get_peer_count(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<usize, String> {
    let app = state.lock().await;
    Ok(app.peer_count)
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize the async runtime for app state
            tauri::async_runtime::spawn(async move {
                match AppState::new().await {
                    Ok((state, mut event_rx)) => {
                        let state = Arc::new(Mutex::new(state));
                        handle.manage(state.clone());

                        // Spawn network event processor
                        let emit_handle = handle.clone();
                        tauri::async_runtime::spawn(async move {
                            while let Some(event) = event_rx.recv().await {
                                let mut app = state.lock().await;
                                match event {
                                    murmur_core::network::node::NodeEvent::PeerDiscovered(peer_id) => {
                                        info!(%peer_id, "Peer discovered");
                                        let _ = emit_handle.emit("peer-discovered", peer_id.to_string());
                                    }
                                    murmur_core::network::node::NodeEvent::PeerLost(peer_id) => {
                                        info!(%peer_id, "Peer lost");
                                        let _ = emit_handle.emit("peer-lost", peer_id.to_string());
                                    }
                                    murmur_core::network::node::NodeEvent::PeerCountChanged(count) => {
                                        app.peer_count = count;
                                        let _ = emit_handle.emit("peer-count", count);
                                    }
                                    murmur_core::network::node::NodeEvent::EventReceived(event) => {
                                        use murmur_core::event::validate::validate_event;
                                        if validate_event(&event).is_err() {
                                            continue;
                                        }
                                        if app.db.has_event(&event.id).unwrap_or(true) {
                                            continue;
                                        }
                                        if let Err(e) = app.db.cache_event(&event) {
                                            tracing::warn!("Failed to cache event: {e}");
                                            continue;
                                        }
                                        let own_pubkey = app.identity.pubkey_hex();
                                        let dto = event_to_dto(&event, &own_pubkey);
                                        let _ = emit_handle.emit("new-event", dto);
                                    }
                                    murmur_core::network::node::NodeEvent::EventsReceived(events) => {
                                        use murmur_core::event::validate::validate_event;
                                        let own_pubkey = app.identity.pubkey_hex();
                                        for event in events {
                                            if validate_event(&event).is_err() {
                                                continue;
                                            }
                                            if app.db.has_event(&event.id).unwrap_or(true) {
                                                continue;
                                            }
                                            if let Err(e) = app.db.cache_event(&event) {
                                                tracing::warn!("Failed to cache event: {e}");
                                                continue;
                                            }
                                            let dto = event_to_dto(&event, &own_pubkey);
                                            let _ = emit_handle.emit("new-event", dto);
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize app state: {e}");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_identity,
            get_timeline,
            get_posts,
            get_thread,
            publish_post,
            publish_reply,
            get_stats,
            get_peer_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running murmur desktop");
}
