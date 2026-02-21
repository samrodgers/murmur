//! Author's permanent archive.
//! Own events are never evicted from storage.

use crate::event::types::Event;
use crate::storage::db::{Database, DbError};

impl Database {
    /// Store an event authored by the local identity.
    pub fn store_own_event(&self, event: &Event) -> Result<(), DbError> {
        self.insert_event(event, true)
    }

    /// Get all events authored by the local identity.
    pub fn get_all_own_events(&self) -> Result<Vec<Event>, DbError> {
        self.get_own_events()
    }

    /// Get own events count.
    pub fn own_event_count(&self) -> Result<i64, DbError> {
        // Note: own_events is just a convenience wrapper
        let events = self.get_own_events()?;
        Ok(events.len() as i64)
    }
}
