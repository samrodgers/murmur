//! Fluent API for constructing events.
//!
//! Usage:
//!   let event = EventBuilder::post("hello world").sign(&identity);
//!   let reply = EventBuilder::reply("nice!", parent_id).sign(&identity);

use crate::event::serialize::compute_event_id;
use crate::event::types::{Event, EventId, EventKind, Tag};
use crate::identity::Identity;
use chrono::Utc;

pub struct EventBuilder {
    kind: EventKind,
    content: String,
    tags: Vec<Tag>,
    timestamp: Option<i64>,
}

impl EventBuilder {
    /// Create a new top-level post.
    pub fn post(content: &str) -> Self {
        Self {
            kind: EventKind::Post,
            content: content.to_string(),
            tags: Vec::new(),
            timestamp: None,
        }
    }

    /// Create a reply to an existing event.
    pub fn reply(content: &str, parent_id: EventId) -> Self {
        Self {
            kind: EventKind::Reply,
            content: content.to_string(),
            tags: vec![Tag::ParentId(parent_id)],
            timestamp: None,
        }
    }

    /// Create a reply with a thread root reference (for nested replies).
    pub fn reply_in_thread(content: &str, parent_id: EventId, root_id: EventId) -> Self {
        Self {
            kind: EventKind::Reply,
            content: content.to_string(),
            tags: vec![Tag::ParentId(parent_id), Tag::ThreadRootId(root_id)],
            timestamp: None,
        }
    }

    /// Create a repost (amplification) of another event.
    pub fn repost(event_id: EventId) -> Self {
        Self {
            kind: EventKind::Repost,
            content: String::new(),
            tags: vec![Tag::ParentId(event_id)],
            timestamp: None,
        }
    }

    /// Create a reaction to an event.
    pub fn reaction(event_id: EventId, emoji: &str) -> Self {
        Self {
            kind: EventKind::Reaction,
            content: emoji.to_string(),
            tags: vec![Tag::ParentId(event_id)],
            timestamp: None,
        }
    }

    /// Create a correction of a previous post.
    pub fn correction(content: &str, corrects_id: EventId) -> Self {
        Self {
            kind: EventKind::Correction,
            content: content.to_string(),
            tags: vec![Tag::CorrectsId(corrects_id)],
            timestamp: None,
        }
    }

    /// Add a topic tag.
    pub fn topic(mut self, topic: &str) -> Self {
        self.tags.push(Tag::Topic(topic.to_string()));
        self
    }

    /// Add a mention.
    pub fn mention(mut self, pubkey: crate::identity::keypair::PubKey) -> Self {
        self.tags.push(Tag::Mention(pubkey));
        self
    }

    /// Add a URL tag.
    pub fn url(mut self, url: &str) -> Self {
        self.tags.push(Tag::Url(url.to_string()));
        self
    }

    /// Override the timestamp (useful for testing).
    pub fn at(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Sign the event with the given identity, producing a complete Event.
    pub fn sign(self, identity: &Identity) -> Event {
        let pubkey = identity.pubkey();
        let created_at = self.timestamp.unwrap_or_else(|| Utc::now().timestamp());
        let id = compute_event_id(&pubkey, created_at, self.kind, &self.content, &self.tags);
        let signature = identity.sign(&id);

        Event {
            id,
            pubkey,
            created_at,
            kind: self.kind,
            content: self.content,
            tags: self.tags,
            sig: signature.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::validate::validate_event;

    #[test]
    fn test_build_post() {
        let id = Identity::generate();
        let event = EventBuilder::post("hello").topic("test").sign(&id);
        assert_eq!(event.kind, EventKind::Post);
        assert_eq!(event.content, "hello");
        assert!(validate_event(&event).is_ok());
    }

    #[test]
    fn test_build_reply() {
        let id = Identity::generate();
        let parent = EventBuilder::post("parent").sign(&id);
        let reply = EventBuilder::reply("child", parent.id).sign(&id);
        assert_eq!(reply.kind, EventKind::Reply);
        assert_eq!(reply.parent_id(), Some(&parent.id));
        assert!(validate_event(&reply).is_ok());
    }
}
