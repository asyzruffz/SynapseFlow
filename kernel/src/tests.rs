use std::path::PathBuf;

use crux_core::Core;
use synapseflow_domain::{
    DomainError, GeneratedToken, GenerationOutput, GenerationPolicy, GenerationRequest,
    ModelConfig, ModelReference,
};
use uuid::Uuid;

use crate::{
    effects::{generation::ExecuteGeneration, initialization::InitializeGeneration},
    Effect, Event, GenerationExecution, SynapseFlow, ViewModel,
};

fn reference() -> ModelReference {
    ModelReference::parse(format!(
        "registry://fixtures/test@sha256:{}",
        "a".repeat(64)
    ))
    .expect("test model reference should be valid")
}

fn config() -> ModelConfig {
    ModelConfig {
        manifest_path: PathBuf::from("manifest.json"),
        artifact_path: PathBuf::from("weights.gguf"),
        cache_directory: PathBuf::from("cache"),
        publisher_public_key: "test-key".to_owned(),
    }
}

fn request() -> GenerationRequest {
    GenerationRequest::new(
        reference(),
        "test".to_owned(),
        GenerationPolicy::new(2, 0.7, 0.9, 42).expect("test policy should be valid"),
    )
    .expect("test request should be valid")
}

fn output() -> GenerationOutput {
    GenerationOutput::from_tokens(vec![GeneratedToken {
        id: 10,
        text: "hello".to_owned(),
    }])
}

fn initialization_request(effects: Vec<Effect>) -> crux_core::Request<InitializeGeneration> {
    effects
        .into_iter()
        .find_map(|effect| match effect {
            Effect::InitializeGeneration(request) => Some(request),
            Effect::Render(_) | Effect::Generation(_) => None,
        })
        .expect("initialization should request shell composition")
}

fn generation_request(effects: Vec<Effect>) -> crux_core::Request<ExecuteGeneration> {
    effects
        .into_iter()
        .find_map(|effect| match effect {
            Effect::Generation(request) => Some(request),
            Effect::Render(_) | Effect::InitializeGeneration(_) => None,
        })
        .expect("generation submission should request shell execution")
}

fn initialize(core: &Core<SynapseFlow>) -> crux_core::Request<ExecuteGeneration> {
    let mut effect = initialization_request(core.process_event(Event::Initialize {
        request: request(),
        config: config(),
    }));
    let effects = core
        .resolve(&mut effect, Ok(()))
        .expect("a valid initialization request should resolve");
    generation_request(effects)
}

#[test]
fn initialization_requests_the_shell_service_for_the_model_and_config() {
    let core = Core::<SynapseFlow>::new();
    let request = request();
    let runtime_config = config();

    let effect = initialization_request(core.process_event(Event::Initialize {
        request: request.clone(),
        config: runtime_config.clone(),
    }));

    assert_eq!(effect.operation.model, request.model);
    assert_eq!(effect.operation.config, runtime_config);
    assert_eq!(core.view(), ViewModel::Initializing);
}

#[test]
fn successful_initialization_schedules_generation_submission() {
    let core = Core::<SynapseFlow>::new();

    let request = request();
    let effect = initialize(&core);

    assert_eq!(effect.operation.request.model, request.model);
    assert_eq!(core.view(), ViewModel::Generating);
}

#[test]
fn submission_before_initialization_does_not_request_generation() {
    let core = Core::<SynapseFlow>::new();

    let effects = core.process_event(Event::SubmitGeneration(request()));

    assert!(effects.is_empty());
    assert_eq!(core.view(), ViewModel::Uninitialized);
}

#[test]
fn successful_shell_resolution_completes_the_initialized_workflow() {
    let core = Core::<SynapseFlow>::new();
    let mut effect = initialize(&core);
    let session_id = Uuid::nil();
    let output = output();

    let effects = core
        .resolve(
            &mut effect,
            GenerationExecution {
                session_id,
                result: Ok(output.clone()),
            },
        )
        .expect("a valid generation request should resolve");

    assert!(effects
        .iter()
        .any(|effect| matches!(effect, Effect::Render(_))));
    assert_eq!(
        core.view(),
        ViewModel::Completed(crate::GenerationCompletion { session_id, output })
    );
}

#[test]
fn failed_initialization_preserves_a_safe_domain_failure() {
    let core = Core::<SynapseFlow>::new();
    let mut effect = initialization_request(core.process_event(Event::Initialize {
        request: request(),
        config: config(),
    }));

    core.resolve(&mut effect, Err(DomainError::BackendUnavailable))
        .expect("a valid initialization request should resolve");

    assert_eq!(
        core.view(),
        ViewModel::Failed(DomainError::BackendUnavailable)
    );
}
