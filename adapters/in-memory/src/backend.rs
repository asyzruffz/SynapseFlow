use synapseflow_domain::{DomainError, DomainResult, GenerationOutput, GenerationRequest};
use synapseflow_ports::{ModelBackend, VerifiedModel};

/// A deterministic backend that supplies a configured output without model-runtime dependencies.
#[derive(Default)]
pub struct InMemoryModelBackend {
    output: Option<GenerationOutput>,
}

impl InMemoryModelBackend {
    pub fn with_output(output: GenerationOutput) -> Self {
        Self {
            output: Some(output),
        }
    }
}

impl ModelBackend for InMemoryModelBackend {
    fn generate(&self, _: &VerifiedModel, _: &GenerationRequest) -> DomainResult<GenerationOutput> {
        self.output.clone().ok_or(DomainError::BackendUnavailable)
    }
}
