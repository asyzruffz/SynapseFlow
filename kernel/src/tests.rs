use std::path::PathBuf;

use crux_core::Core;
use synapseflow_domain::{
    DomainError, GeneratedToken, GenerationEvent, GenerationOutput, GenerationPolicy,
    GenerationRequest, ModelConfig, ModelReference, PublicSessionId,
};

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

fn session_id() -> PublicSessionId {
    PublicSessionId::new("application-session-0001".to_owned())
        .expect("fixture session should be valid")
}

fn initialization_request(effects: Vec<Effect>) -> crux_core::Request<InitializeGeneration> {
    effects
        .into_iter()
        .find_map(|effect| match effect {
            Effect::InitializeGeneration(request) => Some(request),
            Effect::Render(_) | Effect::Generation(_) | Effect::CancelGeneration(_) => None,
        })
        .expect("initialization should request shell composition")
}

fn generation_request(effects: Vec<Effect>) -> crux_core::Request<ExecuteGeneration> {
    effects
        .into_iter()
        .find_map(|effect| match effect {
            Effect::Generation(request) => Some(request),
            Effect::Render(_) | Effect::InitializeGeneration(_) | Effect::CancelGeneration(_) => {
                None
            }
        })
        .expect("generation submission should request shell execution")
}

fn cancellation_request(
    effects: Vec<Effect>,
) -> crux_core::Request<crate::effects::generation::CancelGeneration> {
    effects
        .into_iter()
        .find_map(|effect| match effect {
            Effect::CancelGeneration(request) => Some(request),
            Effect::Render(_) | Effect::Generation(_) | Effect::InitializeGeneration(_) => None,
        })
        .expect("cancellation should request shell control")
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
    assert_eq!(core.view(), ViewModel::Starting);
}

#[test]
fn submission_before_initialization_does_not_request_generation() {
    let core = Core::<SynapseFlow>::new();

    let effects = core.process_event(Event::SubmitGeneration(request()));

    assert!(effects.is_empty());
    assert_eq!(core.view(), ViewModel::Uninitialized);
}

#[test]
fn node_owned_session_start_accepts_live_generation_events() {
    let core = Core::<SynapseFlow>::new();
    let session_id = synapseflow_domain::PublicSessionId::new("node-session-00000001".to_owned())
        .expect("fixture session id");

    core.process_event(Event::SessionStarted(session_id.clone()));
    core.process_event(Event::GenerationEvent {
        session_id: session_id.clone(),
        event: GenerationEvent::Cancelled,
    });

    assert_eq!(core.view(), ViewModel::Cancelled { session_id });
}

#[test]
fn successful_shell_resolution_completes_the_initialized_workflow() {
    let core = Core::<SynapseFlow>::new();
    let mut effect = initialize(&core);
    let active_session = session_id();
    let output = output();

    let effects = core
        .resolve(
            &mut effect,
            GenerationExecution {
                session_id: active_session.clone(),
                events: vec![
                    GenerationEvent::Token(GeneratedToken {
                        id: 10,
                        text: "hello".to_owned(),
                    }),
                    GenerationEvent::Completed { token_count: 1 },
                ],
            },
        )
        .expect("a valid generation request should resolve");

    assert!(effects
        .iter()
        .any(|effect| matches!(effect, Effect::Render(_))));
    assert_eq!(
        core.view(),
        ViewModel::Completed(crate::GenerationCompletion {
            session_id: active_session,
            output
        })
    );
}

#[test]
fn client_events_render_one_ordered_terminal_session_outcome() {
    let core = Core::<SynapseFlow>::new();
    let mut effect = initialize(&core);
    let active_session = session_id();
    core.resolve(
        &mut effect,
        GenerationExecution {
            session_id: active_session.clone(),
            events: Vec::new(),
        },
    )
    .expect("session start should resolve");
    assert_eq!(
        core.view(),
        ViewModel::Generating {
            session_id: active_session.clone()
        }
    );

    core.process_event(Event::GenerationEvent {
        session_id: active_session.clone(),
        event: GenerationEvent::Token(GeneratedToken {
            id: 11,
            text: "world".to_owned(),
        }),
    });
    core.process_event(Event::GenerationEvent {
        session_id: active_session.clone(),
        event: GenerationEvent::Completed { token_count: 1 },
    });
    core.process_event(Event::GenerationEvent {
        session_id: active_session,
        event: GenerationEvent::Cancelled,
    });

    assert_eq!(
        core.view(),
        ViewModel::Completed(crate::GenerationCompletion {
            session_id: session_id(),
            output: GenerationOutput::from_tokens(vec![GeneratedToken {
                id: 11,
                text: "world".to_owned(),
            }]),
        })
    );
}

#[test]
fn cancellation_is_requested_only_for_the_active_session() {
    let core = Core::<SynapseFlow>::new();
    let mut effect = initialize(&core);
    let active_session = session_id();
    core.resolve(
        &mut effect,
        GenerationExecution {
            session_id: active_session.clone(),
            events: Vec::new(),
        },
    )
    .expect("session start should resolve");

    let cancellation =
        cancellation_request(core.process_event(Event::CancelSession(active_session.clone())));
    assert_eq!(cancellation.operation.session_id, active_session);
    assert_eq!(
        core.view(),
        ViewModel::Cancelling {
            session_id: session_id()
        }
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
