//! Tauri command handlers — the IPC bridge between Svelte and murmur-core.

use crate::state::AppState;
use crate::types::{self, NetworkStatus, Post, Profile, StorageStats, Thread};
use murmur_core::event::builder::EventBuilder;
use murmur_core::identity::Identity;
use std::sync::Mutex;
use tauri::State;
use tracing::info;

type AppResult<T> = Result<T, String>;

fn map_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ─── Identity ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn has_identity(state: State<'_, Mutex<AppState>>) -> AppResult<bool> {
    let state = state.lock().map_err(map_err)?;
    Ok(state.has_identity())
}

#[tauri::command]
pub fn create_identity(
    name: String,
    bio: String,
    state: State<'_, Mutex<AppState>>,
    runtime: State<'_, tokio::runtime::Handle>,
) -> AppResult<Profile> {
    let mut state = state.lock().map_err(map_err)?;

    // Generate identity
    let identity = Identity::generate();
    let pubkey_hex = identity.pubkey_hex();
    let identity_path = state.data_dir.join("identity.json");
    identity.save(&identity_path, "").map_err(map_err)?;

    info!(pubkey = %pubkey_hex, name = %name, "Created new identity");

    state.identity = Some(identity);
    state.display_name = name.clone();
    state.bio = bio.clone();
    state.save_profile().map_err(map_err)?;

    // Try to start network (don't fail identity creation if network fails)
    let online = match runtime.block_on(async { state.start_network().await }) {
        Ok(()) => true,
        Err(e) => {
            tracing::warn!("Network failed to start after identity creation: {e}");
            false
        }
    };

    Ok(Profile {
        pubkey: pubkey_hex.clone(),
        name,
        bio,
        temperature: 0.0,
        post_count: 0,
        joined_at: chrono::Utc::now().timestamp(),
        is_following: false,
        is_blocked: false,
        is_online: online,
        avatar_colour: types::avatar_colour_from_pubkey(&pubkey_hex),
    })
}

#[tauri::command]
pub fn get_own_profile(state: State<'_, Mutex<AppState>>) -> AppResult<Profile> {
    let state = state.lock().map_err(map_err)?;
    let pubkey_hex = state.own_pubkey_hex();
    let post_count = state.db.get_own_events().map(|e| e.len() as u32).unwrap_or(0);

    Ok(Profile {
        pubkey: pubkey_hex.clone(),
        name: state.display_name.clone(),
        bio: state.bio.clone(),
        temperature: 0.0,
        post_count,
        joined_at: 0,
        is_following: false,
        is_blocked: false,
        is_online: state.node.is_some(),
        avatar_colour: types::avatar_colour_from_pubkey(&pubkey_hex),
    })
}

#[tauri::command]
pub fn update_profile(
    name: String,
    bio: String,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<Profile> {
    let mut state = state.lock().map_err(map_err)?;
    state.display_name = name;
    state.bio = bio;
    state.save_profile().map_err(map_err)?;

    let pubkey_hex = state.own_pubkey_hex();
    Ok(Profile {
        pubkey: pubkey_hex.clone(),
        name: state.display_name.clone(),
        bio: state.bio.clone(),
        temperature: 0.0,
        post_count: 0,
        joined_at: 0,
        is_following: false,
        is_blocked: false,
        is_online: state.node.is_some(),
        avatar_colour: types::avatar_colour_from_pubkey(&pubkey_hex),
    })
}

#[tauri::command]
pub fn export_identity(
    path: String,
    passphrase: String,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<()> {
    let state = state.lock().map_err(map_err)?;
    let identity = state.identity.as_ref().ok_or("No identity")?;
    identity
        .save(std::path::Path::new(&path), &passphrase)
        .map_err(map_err)
}

#[tauri::command]
pub fn import_identity(
    path: String,
    passphrase: String,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<Profile> {
    let mut state = state.lock().map_err(map_err)?;
    let identity = Identity::load(std::path::Path::new(&path), &passphrase).map_err(map_err)?;
    let pubkey_hex = identity.pubkey_hex();

    // Save to our data dir
    let identity_path = state.data_dir.join("identity.json");
    identity.save(&identity_path, "").map_err(map_err)?;
    state.identity = Some(identity);

    Ok(Profile {
        pubkey: pubkey_hex.clone(),
        name: state.display_name.clone(),
        bio: state.bio.clone(),
        temperature: 0.0,
        post_count: 0,
        joined_at: 0,
        is_following: false,
        is_blocked: false,
        is_online: false,
        avatar_colour: types::avatar_colour_from_pubkey(&pubkey_hex),
    })
}

// ─── Posts ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_post(
    content: String,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<Post> {
    let state = state.lock().map_err(map_err)?;
    let identity = state.identity.as_ref().ok_or("No identity")?;

    let event = EventBuilder::post(&content).sign(identity);
    state.db.insert_event(&event, true).map_err(map_err)?;

    // Broadcast to network (sync, non-blocking channel send)
    if let Some(node) = &state.node {
        let _ = node.try_broadcast_event(event.clone());
    }

    let pubkey = state.own_pubkey_hex();
    Ok(types::event_to_post(&event, &pubkey, &state.db))
}

#[tauri::command]
pub fn create_reply(
    content: String,
    parent_id: String,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<Post> {
    let state = state.lock().map_err(map_err)?;
    let identity = state.identity.as_ref().ok_or("No identity")?;

    let parent_id_bytes: [u8; 32] = hex::decode(&parent_id)
        .map_err(map_err)?
        .try_into()
        .map_err(|_| "Invalid parent ID".to_string())?;

    // Determine thread root
    let root_id = match state.db.get_event(&parent_id_bytes).map_err(map_err)? {
        Some(parent) => parent
            .thread_root_id()
            .copied()
            .unwrap_or(parent_id_bytes),
        None => parent_id_bytes,
    };

    let event = if root_id == parent_id_bytes {
        EventBuilder::reply(&content, parent_id_bytes).sign(identity)
    } else {
        EventBuilder::reply_in_thread(&content, parent_id_bytes, root_id).sign(identity)
    };

    state.db.insert_event(&event, true).map_err(map_err)?;

    if let Some(node) = &state.node {
        let _ = node.try_broadcast_event(event.clone());
    }

    let pubkey = state.own_pubkey_hex();
    Ok(types::event_to_post(&event, &pubkey, &state.db))
}

#[tauri::command]
pub fn react_to_post(
    post_id: String,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<()> {
    let state = state.lock().map_err(map_err)?;
    let identity = state.identity.as_ref().ok_or("No identity")?;

    let post_id_bytes: [u8; 32] = hex::decode(&post_id)
        .map_err(map_err)?
        .try_into()
        .map_err(|_| "Invalid post ID".to_string())?;

    let event = EventBuilder::reaction(post_id_bytes, "❤️").sign(identity);
    state.db.insert_event(&event, true).map_err(map_err)?;

    if let Some(node) = &state.node {
        let _ = node.try_broadcast_event(event.clone());
    }

    Ok(())
}

#[tauri::command]
pub fn repost(
    post_id: String,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<()> {
    let state = state.lock().map_err(map_err)?;
    let identity = state.identity.as_ref().ok_or("No identity")?;

    let post_id_bytes: [u8; 32] = hex::decode(&post_id)
        .map_err(map_err)?
        .try_into()
        .map_err(|_| "Invalid post ID".to_string())?;

    let event = EventBuilder::repost(post_id_bytes).sign(identity);
    state.db.insert_event(&event, true).map_err(map_err)?;

    if let Some(node) = &state.node {
        let _ = node.try_broadcast_event(event.clone());
    }

    Ok(())
}

// ─── Feeds ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_following_feed(
    limit: u32,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<Vec<Post>> {
    let state = state.lock().map_err(map_err)?;
    let pubkey = state.own_pubkey_hex();
    let events = state.db.get_posts(limit).map_err(map_err)?;
    Ok(events
        .iter()
        .map(|e| types::event_to_post(e, &pubkey, &state.db))
        .collect())
}

#[tauri::command]
pub fn get_discover_feed(limit: u32, state: State<'_, Mutex<AppState>>) -> AppResult<Vec<Post>> {
    // Phase 1: same as following feed (all posts)
    let state = state.lock().map_err(map_err)?;
    let pubkey = state.own_pubkey_hex();
    let events = state.db.get_posts(limit).map_err(map_err)?;
    Ok(events
        .iter()
        .map(|e| types::event_to_post(e, &pubkey, &state.db))
        .collect())
}

#[tauri::command]
pub fn get_new_voices_feed(
    limit: u32,
    state: State<'_, Mutex<AppState>>,
) -> AppResult<Vec<Post>> {
    // Phase 1: same as following feed (all posts)
    let state = state.lock().map_err(map_err)?;
    let pubkey = state.own_pubkey_hex();
    let events = state.db.get_posts(limit).map_err(map_err)?;
    Ok(events
        .iter()
        .map(|e| types::event_to_post(e, &pubkey, &state.db))
        .collect())
}

#[tauri::command]
pub fn get_thread(post_id: String, state: State<'_, Mutex<AppState>>) -> AppResult<Thread> {
    let state = state.lock().map_err(map_err)?;
    let pubkey = state.own_pubkey_hex();

    let post_id_bytes: [u8; 32] = hex::decode(&post_id)
        .map_err(map_err)?
        .try_into()
        .map_err(|_| "Invalid post ID".to_string())?;

    let (root_event, reply_events) = state.db.get_thread(&post_id_bytes).map_err(map_err)?;
    let root = root_event
        .as_ref()
        .ok_or("Thread root not found")?;

    Ok(Thread {
        root: types::event_to_post(root, &pubkey, &state.db),
        replies: reply_events
            .iter()
            .map(|e| types::event_to_post(e, &pubkey, &state.db))
            .collect(),
    })
}

// ─── Social ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn follow_user(pubkey: String, _state: State<'_, Mutex<AppState>>) -> AppResult<()> {
    // Phase 2: Implement follow event creation and storage
    info!(%pubkey, "Follow requested (not yet implemented)");
    Ok(())
}

#[tauri::command]
pub fn unfollow_user(pubkey: String, _state: State<'_, Mutex<AppState>>) -> AppResult<()> {
    info!(%pubkey, "Unfollow requested (not yet implemented)");
    Ok(())
}

#[tauri::command]
pub fn block_user(pubkey: String, _state: State<'_, Mutex<AppState>>) -> AppResult<()> {
    info!(%pubkey, "Block requested (not yet implemented)");
    Ok(())
}

#[tauri::command]
pub fn get_profile(pubkey: String, state: State<'_, Mutex<AppState>>) -> AppResult<Profile> {
    let state = state.lock().map_err(map_err)?;

    let post_count = match murmur_core::identity::keypair::PubKey::from_hex(&pubkey) {
        Ok(pk) => state
            .db
            .get_events_by_pubkey(&pk, 0, 1000)
            .map(|e| e.len() as u32)
            .unwrap_or(0),
        Err(_) => 0,
    };

    Ok(Profile {
        pubkey: pubkey.clone(),
        name: format!("{}…", &pubkey[..8]),
        bio: String::new(),
        temperature: 0.0,
        post_count,
        joined_at: 0,
        is_following: false,
        is_blocked: false,
        is_online: false,
        avatar_colour: types::avatar_colour_from_pubkey(&pubkey),
    })
}

#[tauri::command]
pub fn get_following_list(_state: State<'_, Mutex<AppState>>) -> AppResult<Vec<Profile>> {
    // Phase 2: Return actual following list
    Ok(vec![])
}

// ─── Network ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn start_network(
    state: State<'_, Mutex<AppState>>,
    runtime: State<'_, tokio::runtime::Handle>,
    app_handle: tauri::AppHandle,
) -> AppResult<NetworkStatus> {
    let mut state = state.lock().map_err(map_err)?;
    if state.node.is_some() {
        return Ok(NetworkStatus {
            online: true,
            peer_count: state.peer_count,
            peers: vec![],
            error: None,
        });
    }

    runtime
        .block_on(async { state.start_network().await })
        .map_err(map_err)?;

    // Spawn network listener for the newly created net_rx.
    // Clone the inner Handle (not the State wrapper) so it's 'static.
    let rt: tokio::runtime::Handle = (*runtime).clone();
    std::thread::spawn(move || {
        rt.block_on(crate::state::run_network_listener(app_handle));
    });

    Ok(NetworkStatus {
        online: state.node.is_some(),
        peer_count: state.peer_count,
        peers: vec![],
        error: state.network_error.clone(),
    })
}

#[tauri::command]
pub fn get_network_status(state: State<'_, Mutex<AppState>>) -> AppResult<NetworkStatus> {
    let state = state.lock().map_err(map_err)?;
    Ok(NetworkStatus {
        online: state.node.is_some(),
        peer_count: state.peer_count,
        peers: vec![], // Phase 2: populate from node
        error: state.network_error.clone(),
    })
}

#[tauri::command]
pub fn get_storage_stats(state: State<'_, Mutex<AppState>>) -> AppResult<StorageStats> {
    let state = state.lock().map_err(map_err)?;
    let own = state.db.get_own_events().map(|e| e.len() as u32).unwrap_or(0);
    let cached = state.db.cached_event_count().unwrap_or(0) as u32;

    // Estimate storage size (rough: ~500 bytes per event average)
    let total_events = own + cached;
    let storage_mb = (total_events as f64 * 500.0) / (1024.0 * 1024.0);

    Ok(StorageStats {
        own_posts: own,
        cached_posts: cached,
        storage_used_mb: storage_mb,
        cache_limit_mb: state.cache_limit_mb,
    })
}

#[tauri::command]
pub fn set_cache_limit(megabytes: u32, state: State<'_, Mutex<AppState>>) -> AppResult<()> {
    let mut state = state.lock().map_err(map_err)?;
    state.cache_limit_mb = megabytes;
    Ok(())
}
