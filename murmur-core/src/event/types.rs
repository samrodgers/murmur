use crate::identity::keypair::PubKey;
use serde::{Deserialize, Serialize};

/// SHA-256 hash used as event identifier.
pub type EventId = [u8; 32];

/// The universal event envelope — every piece of data on the network is an Event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// SHA-256 hash of the canonical serialized content
    pub id: EventId,
    /// Author's public key
    pub pubkey: PubKey,
    /// Unix timestamp (seconds)
    pub created_at: i64,
    /// What type of event this is
    pub kind: EventKind,
    /// Payload (max 500 chars for posts)
    pub content: String,
    /// Optional metadata tags
    pub tags: Vec<Tag>,
    /// Ed25519 signature of the id
    #[serde(with = "hex_sig")]
    pub sig: [u8; 64],
}

/// Event types in the Murmur protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EventKind {
    Registration = 0,
    Post = 1,
    Reply = 2,
    Repost = 3,
    Reaction = 4,
    Profile = 5,
    Follow = 6,
    Unfollow = 7,
    Block = 8,
    Correction = 9,
}

impl EventKind {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Registration),
            1 => Some(Self::Post),
            2 => Some(Self::Reply),
            3 => Some(Self::Repost),
            4 => Some(Self::Reaction),
            5 => Some(Self::Profile),
            6 => Some(Self::Follow),
            7 => Some(Self::Unfollow),
            8 => Some(Self::Block),
            9 => Some(Self::Correction),
            _ => None,
        }
    }
}

/// Metadata tags attached to events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Tag {
    /// For replies: which event is this replying to
    ParentId(EventId),
    /// For nested replies: the original post in the thread
    ThreadRootId(EventId),
    /// For corrections: which event is being amended
    CorrectsId(EventId),
    /// Optional hashtag/topic
    Topic(String),
    /// Mentioned user
    Mention(PubKey),
    /// Embedded link
    Url(String),
}

/// Maximum content length for posts (in characters).
pub const MAX_POST_LENGTH: usize = 500;

impl Event {
    /// Get the parent event ID if this is a reply.
    pub fn parent_id(&self) -> Option<&EventId> {
        self.tags.iter().find_map(|t| match t {
            Tag::ParentId(id) => Some(id),
            _ => None,
        })
    }

    /// Get the thread root ID if present.
    pub fn thread_root_id(&self) -> Option<&EventId> {
        self.tags.iter().find_map(|t| match t {
            Tag::ThreadRootId(id) => Some(id),
            _ => None,
        })
    }

    /// Get the ID this event corrects, if any.
    pub fn corrects_id(&self) -> Option<&EventId> {
        self.tags.iter().find_map(|t| match t {
            Tag::CorrectsId(id) => Some(id),
            _ => None,
        })
    }

    /// Get the hex representation of the event ID.
    pub fn id_hex(&self) -> String {
        hex::encode(self.id)
    }

    /// Short display form of the event ID.
    pub fn id_short(&self) -> String {
        let full = self.id_hex();
        format!("{}…", &full[..12])
    }
}

/// Serde helper for [u8; 64] (signatures) as hex strings.
mod hex_sig {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 bytes"))
    }
}
