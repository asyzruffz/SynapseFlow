use synapseflow_domain::{DomainResult, GenerationRequest, ModelConfig};

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
}
