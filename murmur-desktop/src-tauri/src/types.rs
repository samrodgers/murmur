//! Types serialized between Rust and the Svelte frontend via Tauri IPC.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub author: Profile,
    pub content: String,
    pub created_at: i64,
    pub temperature: f64,
    pub reply_count: u32,
    pub repost_count: u32,
    pub reaction_count: u32,
    pub is_reply: bool,
    pub parent_id: Option<String>,
    pub has_reacted: bool,
    pub has_reposted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub pubkey: String,
    pub name: String,
    pub bio: String,
    pub temperature: f64,
    pub post_count: u32,
    pub joined_at: i64,
    pub is_following: bool,
    pub is_blocked: bool,
    pub is_online: bool,
    pub avatar_colour: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub root: Post,
    pub replies: Vec<Post>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub online: bool,
    pub peer_count: u32,
    pub peers: Vec<PeerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub display_name: Option<String>,
    pub connected_since: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub own_posts: u32,
    pub cached_posts: u32,
    pub storage_used_mb: f64,
    pub cache_limit_mb: u32,
}

/// Derive an avatar colour from a pubkey hex string.
/// Hashes the pubkey to produce a consistent HSL colour.
pub fn avatar_colour_from_pubkey(pubkey_hex: &str) -> String {
    // Use first 6 chars of the pubkey hex as a simple hash
    let hue = u32::from_str_radix(&pubkey_hex[..6], 16).unwrap_or(0) % 360;
    format!("hsl({}, 65%, 55%)", hue)
}

/// Convert a murmur-core Event to a frontend Post.
pub fn event_to_post(
    event: &murmur_core::event::types::Event,
    own_pubkey_hex: &str,
    db: &murmur_core::storage::Database,
) -> Post {
    let author_pubkey_hex = event.pubkey.to_hex();
    let is_own = author_pubkey_hex == own_pubkey_hex;

    // Count engagements from the DB
    let reply_count = db
        .get_replies(&event.id)
        .map(|r| r.len() as u32)
        .unwrap_or(0);

    let author = Profile {
        pubkey: author_pubkey_hex.clone(),
        name: if is_own {
            "You".to_string()
        } else {
            format!("{}…", &author_pubkey_hex[..8])
        },
        bio: String::new(),
        temperature: 0.0,
        post_count: 0,
        joined_at: event.created_at,
        is_following: false,
        is_blocked: false,
        is_online: false,
        avatar_colour: avatar_colour_from_pubkey(&author_pubkey_hex),
    };

    Post {
        id: hex::encode(event.id),
        author,
        content: event.content.clone(),
        created_at: event.created_at,
        temperature: 0.0,
        reply_count,
        repost_count: 0,
        reaction_count: 0,
        is_reply: event.kind == murmur_core::event::types::EventKind::Reply,
        parent_id: event.parent_id().map(hex::encode),
        has_reacted: false,
        has_reposted: false,
    }
}
