use synapseflow_domain::{GenerationPolicy, GenerationRequest};

use super::support;

#[test]
fn local_node_assigns_a_session_to_the_shared_generation_workflow() {
    let request = GenerationRequest::new(
        support::reference(),
        "test".to_owned(),
        GenerationPolicy::new(2, 0.7, 0.9, 42).expect("test policy should be valid"),
    )
    .expect("test request should be valid");

    let generation = support::node()
        .execute(request)
        .expect("fake workflow should succeed");

    assert_ne!(generation.session_id, uuid::Uuid::nil());
    assert_eq!(generation.output.text, "hello world");
}
