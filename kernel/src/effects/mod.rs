//! Typed requests that the kernel delegates to a client shell.

pub mod generation;
pub mod initialization;

use crux_core::{render::RenderOperation, Request};

use generation::ExecuteGeneration;
pub use generation::{GenerationCompletion, GenerationExecution};
pub use initialization::InitializeGeneration;

/// Every side effect that a SynapseFlow client shell must support.
#[derive(Debug)]
pub enum Effect {
    /// Notifies the shell that a fresh [`crate::ViewModel`] is available.
    Render(Request<RenderOperation>),
    InitializeGeneration(Request<InitializeGeneration>),
    Generation(Request<ExecuteGeneration>),
}

impl crux_core::Effect for Effect {}

impl From<Request<RenderOperation>> for Effect {
    fn from(request: Request<RenderOperation>) -> Self {
        Self::Render(request)
    }
}

impl From<Request<ExecuteGeneration>> for Effect {
    fn from(request: Request<ExecuteGeneration>) -> Self {
        Self::Generation(request)
    }
}

impl From<Request<InitializeGeneration>> for Effect {
    fn from(request: Request<InitializeGeneration>) -> Self {
        Self::InitializeGeneration(request)
    }
}
