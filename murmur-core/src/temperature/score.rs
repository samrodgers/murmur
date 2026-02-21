//! Temperature calculation engine.
//!
//! T(post) = Σ(engagement_weight × recency_factor)
//! recency_factor = e^(-λt), where t = hours since engagement
//!
//! Phase 1: All posts treated equally (temperature not used for replication).
//! This module exists so the data model is ready for Phase 2.

use crate::event::types::EventKind;

pub struct TemperatureConfig {
    /// Decay constant. λ = 0.05 gives half-life ~14 hours.
    pub decay_lambda: f64,
    /// Weight for a reaction event.
    pub weight_reaction: f64,
    /// Weight for a reply event.
    pub weight_reply: f64,
    /// Weight for a repost event.
    pub weight_repost: f64,
    /// Minimum replication target for any live post.
    pub replication_floor: u32,
    /// Maximum replication target cap.
    pub replication_ceiling: u32,
    /// Scaling factor: maps temperature to node count.
    pub replication_scale: f64,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        Self {
            decay_lambda: 0.05,
            weight_reaction: 1.0,
            weight_reply: 3.0,
            weight_repost: 5.0,
            replication_floor: 3,
            replication_ceiling: 200,
            replication_scale: 10.0,
        }
    }
}

pub struct EngagementEvent {
    pub kind: EventKind,
    pub author_temperature: f64,
    pub timestamp: i64,
}

impl TemperatureConfig {
    /// Calculate current temperature for a post given its engagement history.
    pub fn calculate(&self, engagements: &[EngagementEvent], now: i64) -> f64 {
        engagements
            .iter()
            .map(|e| {
                let weight = match e.kind {
                    EventKind::Reaction => self.weight_reaction,
                    EventKind::Reply => self.weight_reply,
                    EventKind::Repost => self.weight_repost,
                    _ => 0.0,
                };

                let hours_elapsed = (now - e.timestamp) as f64 / 3600.0;
                let recency = (-self.decay_lambda * hours_elapsed).exp();

                // Warmer authors contribute more (author_temp is a multiplier >= 0)
                let author_factor = 1.0 + e.author_temperature.ln().max(0.0);

                weight * recency * author_factor
            })
            .sum()
    }

    /// How many nodes should replicate this post right now.
    pub fn replication_target(&self, temperature: f64) -> u32 {
        let raw = (temperature * self.replication_scale) as u32;
        raw.max(self.replication_floor)
            .min(self.replication_ceiling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_increases_with_engagement() {
        let config = TemperatureConfig::default();
        let now = 1000 * 3600; // arbitrary timestamp

        let engagements = vec![EngagementEvent {
            kind: EventKind::Reply,
            author_temperature: 1.0,
            timestamp: now - 3600, // 1 hour ago
        }];

        let temp = config.calculate(&engagements, now);
        assert!(temp > 0.0);
    }

    #[test]
    fn test_temperature_decays_over_time() {
        let config = TemperatureConfig::default();
        let now = 1000 * 3600;

        let recent = vec![EngagementEvent {
            kind: EventKind::Reply,
            author_temperature: 1.0,
            timestamp: now - 3600, // 1 hour ago
        }];

        let old = vec![EngagementEvent {
            kind: EventKind::Reply,
            author_temperature: 1.0,
            timestamp: now - 72 * 3600, // 72 hours ago
        }];

        let temp_recent = config.calculate(&recent, now);
        let temp_old = config.calculate(&old, now);
        assert!(temp_recent > temp_old);
    }

    #[test]
    fn test_replication_floor() {
        let config = TemperatureConfig::default();
        assert!(config.replication_target(0.0) >= config.replication_floor);
    }
}
