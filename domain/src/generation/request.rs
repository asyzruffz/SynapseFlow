use crate::{DomainError, DomainResult, GenerationPolicy, ModelReference};

/// A request that the application service can execute without knowing its transport.
#[derive(Clone, Debug, PartialEq)]
pub struct GenerationRequest {
    pub model: ModelReference,
    pub prompt: String,
    pub policy: GenerationPolicy,
}

impl GenerationRequest {
    pub fn new(
        model: ModelReference,
        prompt: String,
        policy: GenerationPolicy,
    ) -> DomainResult<Self> {
        if prompt.is_empty() {
            return Err(DomainError::GenerationPolicyInvalid);
        }

        Ok(Self {
            model,
            prompt,
            policy,
        })
    }
}
