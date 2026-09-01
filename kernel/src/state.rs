use crux_core::{render::render, Command};
use synapseflow_domain::{DomainError, GenerationRequest};

use crate::{
    effects::generation::ExecuteGeneration, Effect, Event, GenerationCompletion,
    GenerationExecution,
};

/// Internal state for one client-owned generation workflow.
#[derive(Default)]
pub enum SynapseFlowState {
    /// No workflow has been submitted.
    #[default]
    Ready,
    /// The shell has received a generation request and has not resolved it yet.
    Generating,
    /// A completed workflow ready for presentation.
    Completed(GenerationCompletion),
    /// A safe domain failure ready for presentation.
    Failed(DomainError),
}

impl SynapseFlowState {
    pub(crate) fn update(&mut self, event: Event) -> Command<Effect, Event> {
        match event {
            Event::SubmitGeneration(request) => self.execute_generation(request),
            Event::GenerationResolved(execution) => self.resolve_generation(execution),
        }
    }

    fn execute_generation(&mut self, request: GenerationRequest) -> Command<Effect, Event> {
        *self = Self::Generating;
        Command::request_from_shell(ExecuteGeneration { request })
            .then_send(Event::GenerationResolved)
            .and(render())
    }

    fn resolve_generation(&mut self, execution: GenerationExecution) -> Command<Effect, Event> {
        *self = match execution.result {
            Ok(output) => Self::Completed(GenerationCompletion {
                session_id: execution.session_id,
                output,
            }),
            Err(error) => Self::Failed(error),
        };
        render()
    }
}
