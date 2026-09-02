use crux_core::{render::render, Command};
use synapseflow_domain::{DomainError, GenerationRequest, ModelConfig};

use crate::{
    effects::{generation::ExecuteGeneration, initialization::InitializeGeneration},
    Effect, Event, GenerationCompletion, GenerationExecution,
};

/// Internal state for one client-owned generation workflow.
#[derive(Default)]
pub enum SynapseFlowState {
    #[default]
    Uninitialized,
    Initializing,
    Ready,
    Generating,
    Completed(GenerationCompletion),
    Failed(DomainError),
}

impl SynapseFlowState {
    pub(crate) fn update(&mut self, event: Event) -> Command<Effect, Event> {
        match event {
            Event::Initialize { request, config } => self.initialize_and_submit(request, config),
            Event::InitializationResolved(result) => self.resolve_initialization(result),
            Event::SubmitGeneration(request) => self.execute_generation(request),
            Event::GenerationResolved(execution) => self.resolve_generation(execution),
        }
    }

    fn initialize_and_submit(
        &mut self,
        request: GenerationRequest,
        config: ModelConfig,
    ) -> Command<Effect, Event> {
        if !matches!(self, Self::Uninitialized) {
            return Command::done();
        }

        *self = Self::Initializing;
        Command::request_from_shell(InitializeGeneration {
            model: request.model.clone(),
            config,
        })
        .then_send(Event::InitializationResolved)
        .and(render())
        .then(Command::event(Event::SubmitGeneration(request)))
    }

    fn resolve_initialization(
        &mut self,
        result: synapseflow_domain::DomainResult<()>,
    ) -> Command<Effect, Event> {
        if !matches!(self, Self::Initializing) {
            return Command::done();
        }

        *self = match result {
            Ok(()) => Self::Ready,
            Err(error) => Self::Failed(error),
        };
        render()
    }

    fn execute_generation(&mut self, request: GenerationRequest) -> Command<Effect, Event> {
        if !matches!(self, Self::Ready) {
            return Command::done();
        }

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
