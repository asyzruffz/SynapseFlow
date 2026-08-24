use std::sync::Arc;

use synapseflow_adapter_in_memory::InMemoryPeerDirectory;
use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, ExecutionStrategy, LayerRange, ModelFormat, ModelManifest,
    ModelReference, ShardId, ShardPlan, ShardSpec, TokenizerDeclaration, TokenizerKind,
    LOOM_RUNTIME_PROFILE, LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION,
};
use synapseflow_ports::{ShardAvailability, WorkerCapability, WorkerHealth, WorkerId};

pub(super) fn manifest(minimum_replicas: u8) -> ModelManifest {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/tinyllama@sha256:{}",
        "a".repeat(64)
    ))
    .expect("fixture reference is valid");
    let artifact = ArtifactId::new("weights".to_owned()).expect("fixture artifact is valid");
    let first = ShardSpec::new(
        ShardId::new("first".to_owned()).expect("fixture shard is valid"),
        artifact.clone(),
        LayerRange::new(0, 11).expect("fixture range is valid"),
        minimum_replicas,
    )
    .expect("fixture shard is valid");
    let second = ShardSpec::new(
        ShardId::new("second".to_owned()).expect("fixture shard is valid"),
        artifact.clone(),
        LayerRange::new(11, 22).expect("fixture range is valid"),
        minimum_replicas,
    )
    .expect("fixture shard is valid");
    ModelManifest {
        reference,
        schema_version: LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION,
        model_id: "tinyllama".to_owned(),
        model_version: "fixture-v1".to_owned(),
        format: ModelFormat::Gguf,
        architecture: "llama".to_owned(),
        quantization: "Q5_K_M".to_owned(),
        tokenizer: TokenizerDeclaration {
            kind: TokenizerKind::Embedded,
            model: "llama".to_owned(),
        },
        artifacts: vec![ArtifactDescriptor {
            id: artifact,
            uri: "https://fixtures.example/weights.gguf".to_owned(),
            content_sha256: format!("sha256:{}", "b".repeat(64)),
            size_bytes: 1,
        }],
        publisher_key_id: "ed25519:fixture".to_owned(),
        license: "MIT".to_owned(),
        provenance: "fixture".to_owned(),
        execution_plan: Some(
            ShardPlan::new(ExecutionStrategy::layer_range(), vec![first, second])
                .expect("fixture plan is valid"),
        ),
        runtime_profile: Some(LOOM_RUNTIME_PROFILE.to_owned()),
    }
}

pub(super) fn directory(manifest: &ModelManifest) -> Arc<InMemoryPeerDirectory> {
    let plan = manifest
        .execution_plan
        .as_ref()
        .expect("fixture manifest has a plan");
    let availability = |index: usize| ShardAvailability {
        model: manifest.reference.clone(),
        shard: plan.shards[index].id().clone(),
    };
    let worker = |id: &str, health: WorkerHealth, shards: Vec<ShardAvailability>| {
        WorkerCapability::new(
            WorkerId::new(id.to_owned()).expect("fixture worker is valid"),
            health,
            vec![ExecutionStrategy::layer_range()],
            shards,
        )
        .expect("fixture worker capability is valid")
    };
    Arc::new(InMemoryPeerDirectory::new(vec![
        worker("loopback-a", WorkerHealth::Healthy, vec![availability(0)]),
        worker(
            "loopback-b",
            WorkerHealth::Healthy,
            vec![availability(0), availability(1)],
        ),
        worker("loopback-c", WorkerHealth::Healthy, vec![availability(1)]),
        worker(
            "loopback-z",
            WorkerHealth::Unavailable,
            vec![availability(0)],
        ),
    ]))
}
