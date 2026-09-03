use synapseflow_domain::{DomainResult, GenerationRequest, GenerationTerminal};

use crate::{ExecutionCancellation, GeneratedTokenSink, VerifiedModel};

/// Executes a verified sharded model after the application selects its profile.
///
/// Implementations receive neither a manifest registry nor a client-selected
/// backend. Application implementations may plan and manage the selected
/// profile through lower-level ports; runtime adapters only provide
/// tokenization, stage execution, transport, and worker capabilities.
pub trait ShardedGenerationRuntime: Send + Sync {
    fn generate(
        &self,
        model: &VerifiedModel,
        request: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<GenerationTerminal>;
}
