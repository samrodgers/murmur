//! Kademlia DHT integration via libp2p.
//! Phase 1: Stubbed — mDNS handles all discovery.
//! Phase 2 will enable Kademlia for content-addressable routing
//! and wide-area peer discovery.

/// DHT configuration for Phase 2+.
pub struct DhtConfig {
    /// How many closest peers to query for content.
    pub replication_factor: usize,
    /// How often to refresh the routing table (seconds).
    pub refresh_interval_secs: u64,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            replication_factor: 20,
            refresh_interval_secs: 300,
        }
    }
}
