//! Deterministic planning and session ownership for loopback shard execution.

mod plan;
mod session;

#[cfg(test)]
mod test_support;

pub use plan::{ExecutionRoute, ShardAssignment, ShardPlanner};
pub use session::{
    IdempotencyKey, RecoveryAttempt, SessionConfiguration, SessionManager, SessionSnapshot,
};
