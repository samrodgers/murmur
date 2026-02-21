//! SQLite-backed event store.
//!
//! Two logical partitions:
//! 1. "own" — events authored by this node's identity (never evicted)
//! 2. "cache" — events from others (subject to budget-based eviction)

use crate::event::types::{Event, EventId, EventKind, Tag};
use crate::identity::keypair::PubKey;
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct Database {
    conn: Connection,
}

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS events (
        id              BLOB PRIMARY KEY,
        pubkey          BLOB NOT NULL,
        created_at      INTEGER NOT NULL,
        kind            INTEGER NOT NULL,
        content         TEXT NOT NULL,
        tags            TEXT NOT NULL,
        sig             BLOB NOT NULL,
        is_own          BOOLEAN NOT NULL,
        temperature     REAL DEFAULT 0.0,
        temp_updated_at INTEGER DEFAULT 0,
        received_at     INTEGER NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_pubkey ON events(pubkey);
    CREATE INDEX IF NOT EXISTS idx_kind ON events(kind);
    CREATE INDEX IF NOT EXISTS idx_created ON events(created_at);
    CREATE INDEX IF NOT EXISTS idx_temperature ON events(temperature);
    CREATE INDEX IF NOT EXISTS idx_is_own ON events(is_own);

    CREATE TABLE IF NOT EXISTS thread_map (
        reply_id    BLOB PRIMARY KEY,
        parent_id   BLOB NOT NULL,
        root_id     BLOB NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_parent ON thread_map(parent_id);
    CREATE INDEX IF NOT EXISTS idx_root ON thread_map(root_id);

    CREATE TABLE IF NOT EXISTS follows (
        pubkey      BLOB NOT NULL,
        followed_at INTEGER NOT NULL,
        PRIMARY KEY (pubkey)
    );

    CREATE TABLE IF NOT EXISTS blocks (
        pubkey      BLOB NOT NULL,
        blocked_at  INTEGER NOT NULL,
        PRIMARY KEY (pubkey)
    );

    CREATE TABLE IF NOT EXISTS budget (
        key         TEXT PRIMARY KEY,
        value       INTEGER NOT NULL
    );
";

impl Database {
    /// Open or create a database at the given path.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing).
    pub fn open_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Insert an event into the store.
    /// `is_own` should be true if this event was authored by the local identity.
    pub fn insert_event(&self, event: &Event, is_own: bool) -> Result<(), DbError> {
        let tags_json = serde_json::to_string(&event.tags)?;
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT OR IGNORE INTO events (id, pubkey, created_at, kind, content, tags, sig, is_own, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                event.id.as_slice(),
                event.pubkey.0.as_slice(),
                event.created_at,
                event.kind.as_u8() as i64,
                event.content,
                tags_json,
                event.sig.as_slice(),
                is_own,
                now,
            ],
        )?;

        // If this is a reply, populate the thread map
        if event.kind == EventKind::Reply {
            if let Some(parent_id) = event.parent_id() {
                let root_id = event.thread_root_id().unwrap_or(parent_id);
                self.conn.execute(
                    "INSERT OR IGNORE INTO thread_map (reply_id, parent_id, root_id) VALUES (?1, ?2, ?3)",
                    params![
                        event.id.as_slice(),
                        parent_id.as_slice(),
                        root_id.as_slice(),
                    ],
                )?;
            }
        }

        Ok(())
    }

    /// Get an event by its ID.
    pub fn get_event(&self, id: &EventId) -> Result<Option<Event>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pubkey, created_at, kind, content, tags, sig FROM events WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.as_slice()], |row| {
            Ok(RawEvent {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                created_at: row.get(2)?,
                kind: row.get(3)?,
                content: row.get(4)?,
                tags: row.get(5)?,
                sig: row.get(6)?,
            })
        });

        match result {
            Ok(raw) => Ok(Some(raw_to_event(raw)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Check if we already have an event.
    pub fn has_event(&self, id: &EventId) -> Result<bool, DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM events WHERE id = ?1",
            params![id.as_slice()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get recent events from a specific pubkey.
    pub fn get_events_by_pubkey(
        &self,
        pubkey: &PubKey,
        since: i64,
        limit: u32,
    ) -> Result<Vec<Event>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pubkey, created_at, kind, content, tags, sig FROM events
             WHERE pubkey = ?1 AND created_at >= ?2
             ORDER BY created_at DESC
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(params![pubkey.0.as_slice(), since, limit], |row| {
            Ok(RawEvent {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                created_at: row.get(2)?,
                kind: row.get(3)?,
                content: row.get(4)?,
                tags: row.get(5)?,
                sig: row.get(6)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(raw_to_event(row?)?);
        }
        Ok(events)
    }

    /// Get all events (for timeline display), ordered by creation time.
    pub fn get_timeline(&self, limit: u32) -> Result<Vec<Event>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pubkey, created_at, kind, content, tags, sig FROM events
             WHERE kind IN (1, 2)
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(RawEvent {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                created_at: row.get(2)?,
                kind: row.get(3)?,
                content: row.get(4)?,
                tags: row.get(5)?,
                sig: row.get(6)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(raw_to_event(row?)?);
        }
        Ok(events)
    }

    /// Get top-level posts only (no replies).
    pub fn get_posts(&self, limit: u32) -> Result<Vec<Event>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pubkey, created_at, kind, content, tags, sig FROM events
             WHERE kind = 1
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(RawEvent {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                created_at: row.get(2)?,
                kind: row.get(3)?,
                content: row.get(4)?,
                tags: row.get(5)?,
                sig: row.get(6)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(raw_to_event(row?)?);
        }
        Ok(events)
    }

    /// Get all replies to a given event (direct children).
    pub fn get_replies(&self, parent_id: &EventId) -> Result<Vec<Event>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.pubkey, e.created_at, e.kind, e.content, e.tags, e.sig
             FROM events e
             INNER JOIN thread_map t ON e.id = t.reply_id
             WHERE t.parent_id = ?1
             ORDER BY e.created_at ASC",
        )?;

        let rows = stmt.query_map(params![parent_id.as_slice()], |row| {
            Ok(RawEvent {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                created_at: row.get(2)?,
                kind: row.get(3)?,
                content: row.get(4)?,
                tags: row.get(5)?,
                sig: row.get(6)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(raw_to_event(row?)?);
        }
        Ok(events)
    }

    /// Get the full thread for a root post (root + all replies).
    pub fn get_thread(&self, root_id: &EventId) -> Result<(Option<Event>, Vec<Event>), DbError> {
        let root = self.get_event(root_id)?;

        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.pubkey, e.created_at, e.kind, e.content, e.tags, e.sig
             FROM events e
             INNER JOIN thread_map t ON e.id = t.reply_id
             WHERE t.root_id = ?1 OR t.parent_id = ?1
             ORDER BY e.created_at ASC",
        )?;

        let rows = stmt.query_map(params![root_id.as_slice()], |row| {
            Ok(RawEvent {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                created_at: row.get(2)?,
                kind: row.get(3)?,
                content: row.get(4)?,
                tags: row.get(5)?,
                sig: row.get(6)?,
            })
        })?;

        let mut replies = Vec::new();
        for row in rows {
            replies.push(raw_to_event(row?)?);
        }
        Ok((root, replies))
    }

    /// Get all own events.
    pub fn get_own_events(&self) -> Result<Vec<Event>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pubkey, created_at, kind, content, tags, sig FROM events
             WHERE is_own = 1
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RawEvent {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                created_at: row.get(2)?,
                kind: row.get(3)?,
                content: row.get(4)?,
                tags: row.get(5)?,
                sig: row.get(6)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(raw_to_event(row?)?);
        }
        Ok(events)
    }

    /// Get the number of cached (non-own) events.
    pub fn cached_event_count(&self) -> Result<i64, DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM events WHERE is_own = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get total event count.
    pub fn total_event_count(&self) -> Result<i64, DbError> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
        Ok(count)
    }
}

/// Internal struct for reading raw rows before conversion.
struct RawEvent {
    id: Vec<u8>,
    pubkey: Vec<u8>,
    created_at: i64,
    kind: i64,
    content: String,
    tags: String,
    sig: Vec<u8>,
}

fn raw_to_event(raw: RawEvent) -> Result<Event, DbError> {
    let id: EventId = raw
        .id
        .try_into()
        .map_err(|_| rusqlite::Error::InvalidParameterCount(32, 0))?;

    let pubkey_bytes: [u8; 32] = raw
        .pubkey
        .try_into()
        .map_err(|_| rusqlite::Error::InvalidParameterCount(32, 0))?;

    let sig: [u8; 64] = raw
        .sig
        .try_into()
        .map_err(|_| rusqlite::Error::InvalidParameterCount(64, 0))?;

    let kind = EventKind::from_u8(raw.kind as u8)
        .ok_or_else(|| rusqlite::Error::InvalidParameterCount(0, raw.kind as usize))?;

    let tags: Vec<Tag> = serde_json::from_str(&raw.tags)?;

    Ok(Event {
        id,
        pubkey: PubKey(pubkey_bytes),
        created_at: raw.created_at,
        kind,
        content: raw.content,
        tags,
        sig,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::builder::EventBuilder;
    use crate::identity::Identity;

    #[test]
    fn test_insert_and_retrieve() {
        let db = Database::open_memory().unwrap();
        let id = Identity::generate();
        let event = EventBuilder::post("hello world").sign(&id);

        db.insert_event(&event, true).unwrap();

        let retrieved = db.get_event(&event.id).unwrap().unwrap();
        assert_eq!(retrieved.id, event.id);
        assert_eq!(retrieved.content, "hello world");
    }

    #[test]
    fn test_thread_retrieval() {
        let db = Database::open_memory().unwrap();
        let id = Identity::generate();

        let post = EventBuilder::post("original post").sign(&id);
        db.insert_event(&post, true).unwrap();

        let reply = EventBuilder::reply("reply 1", post.id).sign(&id);
        db.insert_event(&reply, false).unwrap();

        let (root, replies) = db.get_thread(&post.id).unwrap();
        assert!(root.is_some());
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].content, "reply 1");
    }

    #[test]
    fn test_timeline() {
        let db = Database::open_memory().unwrap();
        let id = Identity::generate();

        for i in 0..5 {
            let event = EventBuilder::post(&format!("post {}", i)).sign(&id);
            db.insert_event(&event, true).unwrap();
        }

        let timeline = db.get_timeline(10).unwrap();
        assert_eq!(timeline.len(), 5);
    }

    #[test]
    fn test_duplicate_insert_ignored() {
        let db = Database::open_memory().unwrap();
        let id = Identity::generate();
        let event = EventBuilder::post("hello").sign(&id);

        db.insert_event(&event, true).unwrap();
        db.insert_event(&event, true).unwrap(); // Should not error
        assert_eq!(db.total_event_count().unwrap(), 1);
    }
}
