use crux_core::{render::render, Command};
use synapseflow_domain::{
    DomainError, GenerationEvent, GenerationOutput, GenerationRequest, ModelConfig, PublicSessionId,
};

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
    Starting,
    Generating {
        session_id: PublicSessionId,
        tokens: Vec<synapseflow_domain::GeneratedToken>,
    },
    Cancelling {
        session_id: PublicSessionId,
        tokens: Vec<synapseflow_domain::GeneratedToken>,
    },
    Completed(GenerationCompletion),
    Cancelled(PublicSessionId),
    Failed(DomainError),
}

impl SynapseFlowState {
    pub(crate) fn update(&mut self, event: Event) -> Command<Effect, Event> {
        match event {
            Event::Initialize { request, config } => self.initialize_and_submit(request, config),
            Event::InitializationResolved(result) => self.resolve_initialization(result),
            Event::SubmitGeneration(request) => self.execute_generation(request),
            Event::GenerationResolved(execution) => self.resolve_generation(execution),
            Event::SessionStarted(session_id) => self.start_session(session_id),
            Event::GenerationEvent { session_id, event } => {
                self.apply_generation_event(session_id, event)
            }
            Event::CancelSession(session_id) => self.cancel_session(session_id),
            Event::CancellationResolved { session_id, result } => {
                self.resolve_cancellation(session_id, result)
            }
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

        *self = Self::Starting;
        Command::request_from_shell(ExecuteGeneration { request })
            .then_send(Event::GenerationResolved)
            .and(render())
    }

    fn resolve_generation(&mut self, execution: GenerationExecution) -> Command<Effect, Event> {
        if !matches!(self, Self::Starting) {
            return Command::done();
        }
        *self = Self::Generating {
            session_id: execution.session_id.clone(),
            tokens: Vec::new(),
        };
        for event in execution.events {
            self.apply_generation_event_inner(&execution.session_id, event);
        }
        render()
    }

    fn start_session(&mut self, session_id: PublicSessionId) -> Command<Effect, Event> {
        if !matches!(self, Self::Uninitialized) {
            return Command::done();
        }
        *self = Self::Generating {
            session_id,
            tokens: Vec::new(),
        };
        render()
    }

    fn apply_generation_event(
        &mut self,
        session_id: PublicSessionId,
        event: GenerationEvent,
    ) -> Command<Effect, Event> {
        self.apply_generation_event_inner(&session_id, event);
        render()
    }

    fn apply_generation_event_inner(
        &mut self,
        session_id: &PublicSessionId,
        event: GenerationEvent,
    ) {
        let (active_id, tokens) = match self {
            Self::Generating { session_id, tokens } | Self::Cancelling { session_id, tokens } => {
                (session_id.clone(), tokens.clone())
            }
            _ => return,
        };
        if &active_id != session_id {
            return;
        }
        match event {
            GenerationEvent::Token(token) => match self {
                Self::Generating { tokens, .. } | Self::Cancelling { tokens, .. } => {
                    tokens.push(token)
                }
                _ => {}
            },
            GenerationEvent::Completed { token_count } if token_count == tokens.len() => {
                *self = Self::Completed(GenerationCompletion {
                    session_id: active_id,
                    output: GenerationOutput::from_tokens(tokens),
                });
            }
            GenerationEvent::Cancelled => *self = Self::Cancelled(active_id),
            GenerationEvent::Failed { .. } | GenerationEvent::Completed { .. } => {
                *self = Self::Failed(DomainError::GenerationStreamInvalid)
            }
        }
    }

    fn cancel_session(&mut self, session_id: PublicSessionId) -> Command<Effect, Event> {
        let tokens = match self {
            Self::Generating {
                session_id: active_id,
                tokens,
            } if *active_id == session_id => tokens.clone(),
            _ => return Command::done(),
        };
        *self = Self::Cancelling {
            session_id: session_id.clone(),
            tokens,
        };
        let requested_session = session_id.clone();
        Command::request_from_shell(crate::effects::generation::CancelGeneration { session_id })
            .then_send(move |result| Event::CancellationResolved {
                session_id: requested_session,
                result,
            })
    }

    fn resolve_cancellation(
        &mut self,
        session_id: PublicSessionId,
        result: synapseflow_domain::DomainResult<synapseflow_domain::CancellationResult>,
    ) -> Command<Effect, Event> {
        if !matches!(self, Self::Cancelling { session_id: active_id, .. } if *active_id == session_id)
        {
            return Command::done();
        }
        if let Err(error) = result {
            *self = Self::Failed(error);
        }
        render()
    }
}
