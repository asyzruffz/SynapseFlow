use synapseflow_application::GenerationService;
use synapseflow_domain::{
    DomainError, DomainResult, GenerationRequest, ModelConfig, ModelReference,
};
use synapseflow_kernel::{
    Core, Effect, Event, GenerationCompletion, GenerationExecution, SynapseFlow, ViewModel,
};

use crate::runtime::build_verified_local_generation_service;

/// The current platform shell for the SynapseFlow application.
///
/// It composes the generation service only when the kernel requests
/// initialization. Each execution drives a new `Core`, preserving isolation
/// between CLI invocations.
pub(super) struct CliShell {
    generation: Option<GenerationService>,
}

impl CliShell {
    pub(super) fn new() -> Self {
        Self { generation: None }
    }

    #[cfg(test)]
    pub(super) fn with_generation_service(generation: GenerationService) -> Self {
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

        let effects = core.process_event(Event::Initialize {
            request: request,
            config,
        });
        self.process_effects(&core, effects)?;

        match core.view() {
            ViewModel::Completed(completion) => Ok(completion),
            ViewModel::Failed(error) => Err(error),
            ViewModel::Uninitialized
            | ViewModel::Initializing
            | ViewModel::Ready
            | ViewModel::Generating => Err(DomainError::GenerationFailed),
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
                    let result = self.initialize_generation_service(
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
                    let execution = GenerationExecution {
                        session_id: uuid::Uuid::new_v4(),
                        result: generation.generate(request.operation.request.clone()),
                    };
                    let next_effects = core
                        .resolve(&mut request, execution)
                        .map_err(|_| DomainError::GenerationFailed)?;
                    self.process_effects(core, next_effects)?;
                }
            }
        }
        Ok(())
    }

    fn initialize_generation_service(
        &mut self,
        model: &ModelReference,
        config: ModelConfig,
    ) -> DomainResult<()> {
        if self.generation.is_none() {
            self.generation = Some(build_verified_local_generation_service(model, config)?);
        }
        Ok(())
    }
}
