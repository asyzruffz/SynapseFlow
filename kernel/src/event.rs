use synapseflow_domain::{
    CancellationResult, DomainResult, GenerationEvent, GenerationRequest, ModelConfig,
    PublicSessionId,
};

use crate::GenerationExecution;

/// Actions accepted by a SynapseFlow kernel instance.
#[derive(Debug)]
pub enum Event {
    Initialize {
        request: GenerationRequest,
        config: ModelConfig,
    },
    InitializationResolved(DomainResult<()>),
    SubmitGeneration(GenerationRequest),
    GenerationResolved(GenerationExecution),
    GenerationEvent {
        session_id: PublicSessionId,
        event: GenerationEvent,
    },
    CancelSession(PublicSessionId),
    CancellationResolved {
        session_id: PublicSessionId,
        result: DomainResult<CancellationResult>,
    },
}
