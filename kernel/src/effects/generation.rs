use crux_core::capability::Operation;
use synapseflow_domain::{DomainResult, GenerationOutput, GenerationRequest};
use uuid::Uuid;

/// Requests that the shell execute the existing SynapseFlow generation use case.
#[derive(Clone, Debug, PartialEq)]
pub struct ExecuteGeneration {
    /// The validated request to execute through the shell's configured runtime.
    pub request: GenerationRequest,
}

/// Shell-owned completion metadata and the result of executing a generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationExecution {
    /// Opaque session identifier issued by the shell.
    pub session_id: Uuid,
    /// The completed use-case result.
    pub result: DomainResult<GenerationOutput>,
}

impl Operation for ExecuteGeneration {
    type Output = GenerationExecution;
}

/// A successfully completed generation ready for a client to present.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationCompletion {
    /// Opaque session identifier issued by the shell.
    pub session_id: Uuid,
    /// Generated tokens and decoded text.
    pub output: GenerationOutput,
}
