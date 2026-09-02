use synapseflow_application::GenerationService;
use synapseflow_domain::{DomainError, DomainResult, GenerationRequest};
use synapseflow_kernel::{
    effects, Core, Effect, Event, GenerationCompletion, GenerationExecution, SynapseFlow, ViewModel,
};

/// The current platform shell for the SynapseFlow application.
///
/// It owns no application state beyond one composed generation service. Each
/// execution drives a new `Core`, preserving isolation between CLI invocations.
pub(super) struct CliShell {
    generation: GenerationService,
}

impl CliShell {
    pub(super) fn new(generation: GenerationService) -> Self {
        Self { generation }
    }

    pub(super) fn execute(&self, request: GenerationRequest) -> DomainResult<GenerationCompletion> {
        let core = Core::<SynapseFlow>::new();

        let effects = core.process_event(Event::SubmitGeneration(request));
        self.process_effects(&core, effects)?;

        match core.view() {
            ViewModel::Completed(completion) => Ok(completion),
            ViewModel::Failed(error) => Err(error),
            ViewModel::Ready | ViewModel::Generating => Err(DomainError::GenerationFailed),
        }
    }

    fn process_effects(&self, core: &Core<SynapseFlow>, effects: Vec<Effect>) -> DomainResult<()> {
        for effect in effects {
            match effect {
                Effect::Render(_) => {}
                Effect::Generation(mut request) => {
                    let execution = GenerationExecution {
                        session_id: uuid::Uuid::new_v4(),
                        result: self.generation.generate(request.operation.request.clone()),
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
}
