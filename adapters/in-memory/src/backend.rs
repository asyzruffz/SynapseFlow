use synapseflow_domain::{
    DomainError, DomainResult, GenerationOutput, GenerationRequest, GenerationTerminal,
};
use synapseflow_ports::{ExecutionCancellation, GeneratedTokenSink, ModelBackend, VerifiedModel};

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
    fn generate(
        &self,
        _: &VerifiedModel,
        _: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<GenerationTerminal> {
        let output = self.output.clone().ok_or(DomainError::BackendUnavailable)?;
        let token_count = output.tokens.len();
        for token in output.tokens {
            if cancellation.is_cancelled() {
                return Ok(GenerationTerminal::Cancelled);
            }
            tokens.emit_token(token)?;
        }
        Ok(GenerationTerminal::Completed { token_count })
    }
}
