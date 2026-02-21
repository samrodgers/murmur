//! Event validation — verify signatures, check hashes, enforce limits.

use crate::event::serialize::compute_event_id;
use crate::event::types::{Event, EventKind, MAX_POST_LENGTH};
use crate::identity::keypair;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Event ID mismatch: computed {computed} but event claims {claimed}")]
    IdMismatch { computed: String, claimed: String },
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Content too long: {length} chars (max {max})")]
    ContentTooLong { length: usize, max: usize },
    #[error("Timestamp is in the future")]
    FutureTimestamp,
    #[error("Reply event missing parent_id tag")]
    MissingParentId,
    #[error("Correction event missing corrects_id tag")]
    MissingCorrectsId,
}

/// Validate an event: check the hash, verify the signature,
/// and enforce content constraints.
pub fn validate_event(event: &Event) -> Result<(), ValidationError> {
    // 1. Recompute the event ID from the canonical content
    let computed_id = compute_event_id(
        &event.pubkey,
        event.created_at,
        event.kind,
        &event.content,
        &event.tags,
    );
    if computed_id != event.id {
        return Err(ValidationError::IdMismatch {
            computed: hex::encode(computed_id),
            claimed: hex::encode(event.id),
        });
    }

    // 2. Verify the signature over the event ID
    if !keypair::verify(&event.pubkey, &event.id, &event.sig) {
        return Err(ValidationError::InvalidSignature);
    }

    // 3. Content length check for post-like events
    match event.kind {
        EventKind::Post | EventKind::Reply | EventKind::Correction => {
            if event.content.chars().count() > MAX_POST_LENGTH {
                return Err(ValidationError::ContentTooLong {
                    length: event.content.chars().count(),
                    max: MAX_POST_LENGTH,
                });
            }
        }
        _ => {}
    }

    // 4. Structural checks for specific event kinds
    if event.kind == EventKind::Reply && event.parent_id().is_none() {
        return Err(ValidationError::MissingParentId);
    }
    if event.kind == EventKind::Correction && event.corrects_id().is_none() {
        return Err(ValidationError::MissingCorrectsId);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::builder::EventBuilder;
    use crate::identity::Identity;

    #[test]
    fn test_valid_post() {
        let id = Identity::generate();
        let event = EventBuilder::post("hello world").sign(&id);
        assert!(validate_event(&event).is_ok());
    }

    #[test]
    fn test_tampered_content_fails() {
        let id = Identity::generate();
        let mut event = EventBuilder::post("hello world").sign(&id);
        event.content = "tampered".to_string();
        assert!(validate_event(&event).is_err());
    }

    #[test]
    fn test_content_too_long() {
        let id = Identity::generate();
        let long_content: String = "x".repeat(501);
        let event = EventBuilder::post(&long_content).sign(&id);
        let result = validate_event(&event);
        assert!(matches!(result, Err(ValidationError::ContentTooLong { .. })));
    }
}
