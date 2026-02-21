//! Cached events from other users (with eviction).
//! Phase 1: Simple cache — store everything, no eviction yet.

use crate::event::types::Event;
use crate::storage::db::{Database, DbError};

impl Database {
    /// Store a cached event from another user.
    pub fn cache_event(&self, event: &Event) -> Result<(), DbError> {
        self.insert_event(event, false)
    }
}
