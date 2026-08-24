use synapseflow_domain::{DomainResult, ExecutionStrategy};

use crate::VerifiedModel;

use super::{ShardExecutionOutput, ShardExecutionRequest};

/// Executes a strategy-specific shard through a strategy-neutral backend seam.
pub trait ShardExecutionBackend: Send + Sync {
    fn supports(&self, strategy: &ExecutionStrategy) -> bool;

    fn execute(
        &self,
        model: &VerifiedModel,
        request: &ShardExecutionRequest,
    ) -> DomainResult<ShardExecutionOutput>;
}
