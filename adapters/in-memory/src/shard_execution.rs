use std::sync::Mutex;

use synapseflow_domain::{DomainError, DomainResult, ExecutionStrategy};
use synapseflow_ports::{
    ShardExecutionBackend, ShardExecutionOutput, ShardExecutionRequest, VerifiedModel,
};

/// Deterministic configured shard executor for planning and session tests.
pub struct InMemoryShardExecutionBackend {
    output: Option<ShardExecutionOutput>,
    requests: Mutex<Vec<ShardExecutionRequest>>,
}

impl InMemoryShardExecutionBackend {
    pub fn with_output(output: ShardExecutionOutput) -> Self {
        Self {
            output: Some(output),
            requests: Mutex::new(Vec::new()),
        }
    }

    pub fn requests(&self) -> DomainResult<Vec<ShardExecutionRequest>> {
        self.requests
            .lock()
            .map(|requests| requests.clone())
            .map_err(|_| DomainError::CacheFailure)
    }
}

impl ShardExecutionBackend for InMemoryShardExecutionBackend {
    fn supports(&self, strategy: &ExecutionStrategy) -> bool {
        strategy.is_layer_range()
    }

    fn execute(
        &self,
        model: &VerifiedModel,
        request: &ShardExecutionRequest,
    ) -> DomainResult<ShardExecutionOutput> {
        request.validate_for(model)?;
        let output = self.output.clone().ok_or(DomainError::BackendUnavailable)?;
        output.validate_for(request)?;
        self.requests
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .push(request.clone());
        Ok(output)
    }
}
