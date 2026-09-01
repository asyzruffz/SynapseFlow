use synapseflow_domain::GenerationRequest;

use crate::GenerationExecution;

/// Actions accepted by a SynapseFlow kernel instance.
#[derive(Debug)]
pub enum Event {
    /// Begins one generation workflow.
    SubmitGeneration(GenerationRequest),
    /// Delivers the result of the shell-executed generation capability.
    GenerationResolved(GenerationExecution),
}
