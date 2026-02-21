//! Canonical serialization for event hashing and signing.
//!
//! The canonical form is a JSON array with fields in a fixed order:
//! [pubkey, created_at, kind, content, tags]
//!
//! This ensures that the event ID (SHA-256 of this serialization)
//! is deterministic regardless of field ordering in the struct.

use crate::event::types::{EventKind, Tag};
use crate::identity::keypair::PubKey;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Produce the canonical byte representation of an event's content
/// (everything except the id and sig, which are derived from this).
pub fn canonical_bytes(
    pubkey: &PubKey,
    created_at: i64,
    kind: EventKind,
    content: &str,
    tags: &[Tag],
) -> Vec<u8> {
    let canonical = serde_json::json!([
        hex::encode(pubkey.0),
        created_at,
        kind.as_u8(),
        content,
        tags_to_value(tags),
    ]);
    // serde_json::to_vec produces compact JSON with no extra whitespace
    serde_json::to_vec(&canonical).expect("canonical serialization should never fail")
}

/// Compute the SHA-256 hash of the canonical event content.
pub fn compute_event_id(
    pubkey: &PubKey,
    created_at: i64,
    kind: EventKind,
    content: &str,
    tags: &[Tag],
) -> [u8; 32] {
    let bytes = canonical_bytes(pubkey, created_at, kind, content, tags);
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.finalize().into()
}

fn tags_to_value(tags: &[Tag]) -> Value {
    let arr: Vec<Value> = tags
        .iter()
        .map(|tag| match tag {
            Tag::ParentId(id) => serde_json::json!(["parent", hex::encode(id)]),
            Tag::ThreadRootId(id) => serde_json::json!(["root", hex::encode(id)]),
            Tag::CorrectsId(id) => serde_json::json!(["corrects", hex::encode(id)]),
            Tag::Topic(s) => serde_json::json!(["topic", s]),
            Tag::Mention(pk) => serde_json::json!(["mention", hex::encode(pk.0)]),
            Tag::Url(u) => serde_json::json!(["url", u]),
        })
        .collect();
    Value::Array(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_determinism() {
        let pk = PubKey([0xAB; 32]);
        let tags = vec![Tag::Topic("test".into())];
        let id1 = compute_event_id(&pk, 1000, EventKind::Post, "hello", &tags);
        let id2 = compute_event_id(&pk, 1000, EventKind::Post, "hello", &tags);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_different_content_different_id() {
        let pk = PubKey([0xAB; 32]);
        let id1 = compute_event_id(&pk, 1000, EventKind::Post, "hello", &[]);
        let id2 = compute_event_id(&pk, 1000, EventKind::Post, "world", &[]);
        assert_ne!(id1, id2);
    }
}
