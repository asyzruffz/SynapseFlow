use synapseflow_domain::execution::FrameTarget;
use synapseflow_domain::{DomainResult, ExecutionStrategy};

use crate::{WorkerCapability, WorkerId};

/// Resolves static loopback worker capabilities and eligible replicas.
pub trait PeerDirectory: Send + Sync {
    fn worker(&self, worker: &WorkerId) -> DomainResult<Option<WorkerCapability>>;

    /// Returns all workers that advertise the target and strategy, including
    /// their health so the planner can make a deterministic policy decision.
    fn replicas(
        &self,
        target: &FrameTarget,
        strategy: &ExecutionStrategy,
    ) -> DomainResult<Vec<WorkerCapability>>;
}
