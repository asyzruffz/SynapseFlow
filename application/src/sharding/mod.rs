//! Deterministic planning and session ownership for loopback shard execution.

mod plan;
mod runtime;
mod session;

#[cfg(test)]
mod test_support;

pub use plan::{ExecutionRoute, ShardAssignment, ShardPlanner};
pub use runtime::LayerRangeShardedGenerationRuntime;
pub use session::{
    IdempotencyKey, RecoveryAttempt, SessionConfiguration, SessionManager, SessionSnapshot,
};
