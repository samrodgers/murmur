//! Murmur wire protocol — the messages nodes exchange.

use crate::event::types::{Event, EventId};
use crate::identity::keypair::PubKey;
use serde::{Deserialize, Serialize};

/// Requests a node can make to peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MurmurRequest {
    /// "Do you have this event?"
    GetEvent { id: EventId },
    /// "Here's an event I think you should store"
    PushEvent { event: Event },
    /// "Give me recent events from this pubkey"
    GetFeed {
        pubkey: PubKey,
        since: i64,
        limit: u32,
    },
    /// "What are the hottest events you know about?"
    GetHot {
        min_temperature: f64,
        limit: u32,
    },
    /// "Give me the full thread for this post"
    GetThread { root_id: EventId },
    /// "Give me all your recent events" (for initial sync)
    SyncRequest { since: i64 },
}

/// Responses sent back to requesting peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MurmurResponse {
    /// Single event
    Event(Event),
    /// Multiple events
    Events(Vec<Event>),
    /// A full thread
    Thread {
        root: Option<Event>,
        replies: Vec<Event>,
    },
    /// Event not found
    NotFound,
    /// Event accepted for storage
    Accepted,
    /// Error
    Error(String),
}
