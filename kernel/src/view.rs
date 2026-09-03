use synapseflow_domain::{DomainError, PublicSessionId};

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
    /// A submitted workflow is waiting for an application-issued session handle.
    Starting,
    /// A session is emitting ordered token events.
    Generating { session_id: PublicSessionId },
    /// Cancellation was requested; a terminal generation event remains authoritative.
    Cancelling { session_id: PublicSessionId },
    /// The completed generation and application-issued session identifier.
    Completed(GenerationCompletion),
    /// The application confirmed cancellation for this session.
    Cancelled { session_id: PublicSessionId },
    /// A stable, sanitized domain failure.
    Failed(DomainError),
}

impl From<&SynapseFlowState> for ViewModel {
    fn from(value: &SynapseFlowState) -> Self {
        match value {
            SynapseFlowState::Uninitialized => Self::Uninitialized,
            SynapseFlowState::Initializing => Self::Initializing,
            SynapseFlowState::Ready => Self::Ready,
            SynapseFlowState::Starting => Self::Starting,
            SynapseFlowState::Generating { session_id, .. } => Self::Generating {
                session_id: session_id.clone(),
            },
            SynapseFlowState::Cancelling { session_id, .. } => Self::Cancelling {
                session_id: session_id.clone(),
            },
            SynapseFlowState::Completed(completion) => Self::Completed(completion.clone()),
            SynapseFlowState::Cancelled(session_id) => Self::Cancelled {
                session_id: session_id.clone(),
            },
            SynapseFlowState::Failed(error) => Self::Failed(error.clone()),
        }
    }
}
