use synapseflow_domain::{DomainResult, GenerationOutput, GenerationRequest};

use crate::VerifiedModel;

/// Tokenizes and executes a verified full model.
pub trait ModelBackend: Send + Sync {
    fn generate(
        &self,
        model: &VerifiedModel,
        request: &GenerationRequest,
    ) -> DomainResult<GenerationOutput>;
}
