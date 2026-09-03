use crux_core::capability::Operation;
use synapseflow_domain::{
    CancellationResult, DomainResult, GenerationEvent, GenerationOutput, GenerationRequest,
    PublicSessionId,
};

/// Requests that the shell execute the existing SynapseFlow generation use case.
#[derive(Clone, Debug, PartialEq)]
pub struct ExecuteGeneration {
    /// The validated request to execute through the shell's configured runtime.
    pub request: GenerationRequest,
}

impl Operation for ExecuteGeneration {
    type Output = GenerationExecution;
}

/// Application-issued session handle and the ordered events resolved by a shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationExecution {
    pub session_id: PublicSessionId,
    pub events: Vec<GenerationEvent>,
}

/// A successfully completed generation ready for a client to present.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationCompletion {
    /// Application-issued opaque session identifier.
    pub session_id: PublicSessionId,
    /// Generated tokens and decoded text.
    pub output: GenerationOutput,
}

/// Requests cancellation of an application-owned session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelGeneration {
    pub session_id: PublicSessionId,
}

impl Operation for CancelGeneration {
    type Output = DomainResult<CancellationResult>;
}
