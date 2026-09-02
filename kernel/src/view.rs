use synapseflow_domain::DomainError;

use crate::{state::SynapseFlowState, GenerationCompletion};

/// Presentation-safe state produced by the kernel for a client shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewModel {
    /// The shell has not started initializing.
    Uninitialized,
    /// The shell is composing a generation service from config.
    Initializing,
    /// Runtime initialization has completed and generation may begin.
    Ready,
    /// A submitted workflow is awaiting the shell result.
    Generating,
    /// The completed generation and shell-issued session identifier.
    Completed(GenerationCompletion),
    /// A stable, sanitized domain failure.
    Failed(DomainError),
}

impl From<&SynapseFlowState> for ViewModel {
    fn from(value: &SynapseFlowState) -> Self {
        match value {
            SynapseFlowState::Uninitialized => Self::Uninitialized,
            SynapseFlowState::Initializing => Self::Initializing,
            SynapseFlowState::Ready => Self::Ready,
            SynapseFlowState::Generating => Self::Generating,
            SynapseFlowState::Completed(completion) => Self::Completed(completion.clone()),
            SynapseFlowState::Failed(error) => Self::Failed(error.clone()),
        }
    }
}
