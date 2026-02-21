//! Event synchronization logic.
//!
//! When two peers connect:
//! 1. Each pushes their own recent events to the other
//! 2. Each requests events from the other
//!
//! Phase 1: Simple full sync — every peer gets everything.
//! Phase 2+: Temperature-aware selective replication.

use crate::event::types::Event;
use crate::event::validate::validate_event;
use crate::storage::db::{Database, DbError};
use tracing::debug;

/// Process an incoming event from a peer. Validates and stores it.
/// Returns true if the event was new and stored, false if duplicate.
pub fn process_incoming_event(db: &Database, event: &Event) -> Result<bool, SyncError> {
    // Check if we already have it
    if db.has_event(&event.id).map_err(SyncError::Storage)? {
        debug!(id = %hex::encode(event.id), "Already have event, skipping");
        return Ok(false);
    }

    // Validate signature and content
    validate_event(event).map_err(SyncError::Validation)?;

    // Store as a cached event (not our own)
    db.cache_event(event).map_err(SyncError::Storage)?;

    debug!(id = %hex::encode(event.id), kind = ?event.kind, "Stored new event from peer");
    Ok(true)
}

/// Get events to push to a newly connected peer.
/// Phase 1: Push all our own events.
pub fn events_to_push(db: &Database) -> Result<Vec<Event>, SyncError> {
    db.get_own_events().map_err(SyncError::Storage)
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Storage error: {0}")]
    Storage(DbError),
    #[error("Validation error: {0}")]
    Validation(#[from] crate::event::validate::ValidationError),
}
