//! Bootstrap node management.
//!
//! Phase 1: Uses mDNS for local network discovery only.
//! Phase 2+: Will use hardcoded bootstrap nodes for DHT entry,
//! with DNS-based discovery and manual peer entry as fallbacks.

/// Hardcoded bootstrap nodes. Empty for Phase 1 (mDNS only).
pub const BOOTSTRAP_NODES: &[&str] = &[
    // Phase 2: Add bootstrap multiaddrs here, e.g.:
    // "/ip4/1.2.3.4/tcp/9000/p2p/12D3KooW..."
];

/// Get the list of bootstrap peer addresses.
pub fn bootstrap_addrs() -> Vec<String> {
    BOOTSTRAP_NODES.iter().map(|s| s.to_string()).collect()
}
