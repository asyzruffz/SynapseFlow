use synapseflow_domain::{DomainResult, GenerationRequest, GenerationTerminal};

use crate::{ExecutionCancellation, GeneratedTokenSink, VerifiedModel};

/// Tokenizes and executes a verified full model.
pub trait ModelBackend: Send + Sync {
    fn generate(
        &self,
        model: &VerifiedModel,
        request: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<GenerationTerminal>;
}
