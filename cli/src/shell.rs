use synapseflow_application::GenerationOrchestrator;
use synapseflow_domain::{
    CancellationResult, DomainError, DomainResult, GenerationEvent, GenerationRequest, ModelConfig,
    ModelReference, PublicSessionState,
};
use synapseflow_kernel::{
    Core, Effect, Event, GenerationCompletion, GenerationExecution, SynapseFlow, ViewModel,
};

use crate::runtime::build_generation_orchestrator;

/// The current platform shell for the SynapseFlow application.
///
/// It composes the generation orchestrator only when the kernel requests
/// initialization. Each execution drives a new `Core`, preserving isolation
/// between CLI invocations.
pub(super) struct CliShell {
    generation: Option<GenerationOrchestrator>,
}

impl CliShell {
    pub(super) fn new() -> Self {
        Self { generation: None }
    }

    #[cfg(test)]
    pub(super) fn with_generation_orchestrator(generation: GenerationOrchestrator) -> Self {
        Self {
            generation: Some(generation),
        }
    }

    pub(super) fn execute(
        &mut self,
        request: GenerationRequest,
        config: ModelConfig,
    ) -> DomainResult<GenerationCompletion> {
        let core = Core::<SynapseFlow>::new();

        let effects = core.process_event(Event::Initialize { request, config });
        self.process_effects(&core, effects)?;

        match core.view() {
            ViewModel::Completed(completion) => Ok(completion),
            ViewModel::Failed(error) => Err(error),
            ViewModel::Uninitialized
            | ViewModel::Initializing
            | ViewModel::Ready
            | ViewModel::Starting
            | ViewModel::Generating { .. }
            | ViewModel::Cancelling { .. }
            | ViewModel::Cancelled { .. } => Err(DomainError::GenerationFailed),
        }
    }

    fn process_effects(
        &mut self,
        core: &Core<SynapseFlow>,
        effects: Vec<Effect>,
    ) -> DomainResult<()> {
        for effect in effects {
            match effect {
                Effect::Render(_) => {}
                Effect::InitializeGeneration(mut request) => {
                    let result = self.initialize_generation_orchestrator(
                        &request.operation.model,
                        request.operation.config.clone(),
                    );
                    let next_effects = core
                        .resolve(&mut request, result)
                        .map_err(|_| DomainError::GenerationFailed)?;
                    self.process_effects(core, next_effects)?;
                }
                Effect::Generation(mut request) => {
                    let generation = self
                        .generation
                        .as_ref()
                        .ok_or(DomainError::BackendUnavailable)?;
                    let session_id = generation.issue_transient_session_id()?;
                    let events = match generation.generate(request.operation.request.clone()) {
                        Ok(output) => {
                            let token_count = output.tokens.len();
                            let mut events = output
                                .tokens
                                .into_iter()
                                .map(GenerationEvent::Token)
                                .collect::<Vec<_>>();
                            events.push(GenerationEvent::Completed { token_count });
                            events
                        }
                        Err(error) => vec![GenerationEvent::Failed { code: error.code() }],
                    };
                    let execution = GenerationExecution { session_id, events };
                    let next_effects = core
                        .resolve(&mut request, execution)
                        .map_err(|_| DomainError::GenerationFailed)?;
                    self.process_effects(core, next_effects)?;
                }
                Effect::CancelGeneration(mut request) => {
                    let next_effects = core
                        .resolve(
                            &mut request,
                            Ok(CancellationResult::AlreadyTerminal(
                                PublicSessionState::Failed,
                            )),
                        )
                        .map_err(|_| DomainError::GenerationFailed)?;
                    self.process_effects(core, next_effects)?;
                }
            }
        }
        Ok(())
    }

    fn initialize_generation_orchestrator(
        &mut self,
        model: &ModelReference,
        config: ModelConfig,
    ) -> DomainResult<()> {
        if self.generation.is_none() {
            self.generation = Some(build_generation_orchestrator(model, config)?);
        }
        Ok(())
    }
}
