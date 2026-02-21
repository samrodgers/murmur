//! Maps temperature to target replication count.
//! Phase 1: Not used — every connected peer gets everything.
//! This module provides the mapping logic for Phase 2+.

use super::score::TemperatureConfig;

/// Given a post temperature, determine replication strategy.
pub fn replication_strategy(config: &TemperatureConfig, temperature: f64) -> ReplicationStrategy {
    let target = config.replication_target(temperature);

    if temperature < 0.1 {
        ReplicationStrategy::Cold { target }
    } else if temperature < 10.0 {
        ReplicationStrategy::Warm { target }
    } else {
        ReplicationStrategy::Hot { target }
    }
}

#[derive(Debug)]
pub enum ReplicationStrategy {
    /// Post is cold — only replicate to minimum floor nodes.
    Cold { target: u32 },
    /// Post has moderate engagement — replicate to a moderate number.
    Warm { target: u32 },
    /// Post is hot — replicate widely, use gossipsub broadcast.
    Hot { target: u32 },
}
