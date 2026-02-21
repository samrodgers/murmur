//! Storage budget management & eviction policy.
//! Phase 1: Stubbed out. No eviction — store everything.
//! In a full implementation, the coldest cached events would be
//! evicted when the storage budget is exceeded.

pub struct StorageBudget {
    /// Maximum number of cached (non-own) events to store.
    pub max_cached_events: u64,
}

impl Default for StorageBudget {
    fn default() -> Self {
        Self {
            max_cached_events: 10_000, // Phase 1: generous default, no actual eviction
        }
    }
}
