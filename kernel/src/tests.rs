use crux_core::Core;
use synapseflow_domain::{
    DomainError, GeneratedToken, GenerationOutput, GenerationPolicy, GenerationRequest,
    ModelReference,
};
use uuid::Uuid;

use crate::{
    effects::generation::ExecuteGeneration, Effect, Event, GenerationExecution, SynapseFlow,
    ViewModel,
};

fn request() -> GenerationRequest {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/test@sha256:{}",
        "a".repeat(64)
    ))
    .expect("test model reference should be valid");
    GenerationRequest::new(
        reference,
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

fn generation_request(effects: Vec<Effect>) -> crux_core::Request<ExecuteGeneration> {
    effects
        .into_iter()
        .find_map(|effect| match effect {
            Effect::Generation(request) => Some(request),
            Effect::Render(_) => None,
        })
        .expect("generation submission should request shell execution")
}

#[test]
fn submission_requests_generation_and_exposes_running_state() {
    let core = Core::<SynapseFlow>::new();
    let request = request();

    let effect = generation_request(core.process_event(Event::SubmitGeneration(request.clone())));

    assert_eq!(effect.operation.request, request);
    assert_eq!(core.view(), ViewModel::Generating);
}

#[test]
fn successful_shell_resolution_completes_the_kernel_workflow() {
    let core = Core::<SynapseFlow>::new();
    let mut effect = generation_request(core.process_event(Event::SubmitGeneration(request())));
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
fn failed_shell_resolution_preserves_a_safe_domain_failure() {
    let core = Core::<SynapseFlow>::new();
    let mut effect = generation_request(core.process_event(Event::SubmitGeneration(request())));

    core.resolve(
        &mut effect,
        GenerationExecution {
            session_id: Uuid::nil(),
            result: Err(DomainError::BackendUnavailable),
        },
    )
    .expect("a valid generation request should resolve");

    assert_eq!(
        core.view(),
        ViewModel::Failed(DomainError::BackendUnavailable)
    );
}
