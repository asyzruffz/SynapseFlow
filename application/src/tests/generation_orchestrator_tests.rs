use std::sync::{Arc, Mutex};

use synapseflow_adapter_in_memory::{
    InMemoryAuditSink, InMemoryModelBackend, InMemoryModelRegistry,
};
use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DomainError, DomainResult, GeneratedToken, GenerationOutput,
    GenerationPolicy, GenerationRequest, LayerRange, ModelFormat, ModelManifest, ModelReference,
    ShardId, ShardPlan, ShardSpec, TokenizerDeclaration, TokenizerKind, LOOM_RUNTIME_PROFILE,
};
use synapseflow_ports::{
    ArtifactStore, ModelCacheInspection, ShardedGenerationRuntime, VerifiedModel,
};

use crate::GenerationOrchestrator;

struct VerifiedArtifacts;

impl ArtifactStore for VerifiedArtifacts {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel> {
        Ok(VerifiedModel::without_cached_artifacts(manifest.clone()))
    }

    fn inspect(&self, _: &ModelManifest) -> DomainResult<ModelCacheInspection> {
        Err(DomainError::CacheFailure)
    }
}

struct ShardedRuntime {
    called: Mutex<bool>,
    output: GenerationOutput,
}

impl ShardedGenerationRuntime for ShardedRuntime {
    fn generate(&self, _: &VerifiedModel, _: &GenerationRequest) -> DomainResult<GenerationOutput> {
        *self.called.lock().map_err(|_| DomainError::CacheFailure)? = true;
        Ok(self.output.clone())
    }
}

fn reference() -> ModelReference {
    ModelReference::parse(format!(
        "registry://fixtures/model@sha256:{}",
        "a".repeat(64)
    ))
    .expect("fixture reference is valid")
}

fn request(reference: ModelReference) -> GenerationRequest {
    GenerationRequest::new(
        reference,
        "prompt".to_owned(),
        GenerationPolicy::new(1, 0.7, 0.9, 42).expect("fixture policy is valid"),
    )
    .expect("fixture request is valid")
}

fn local_manifest() -> ModelManifest {
    ModelManifest {
        reference: reference(),
        schema_version: 1,
        model_id: "fixture".to_owned(),
        model_version: "v1".to_owned(),
        format: ModelFormat::Gguf,
        architecture: "llama".to_owned(),
        quantization: "Q5_K_M".to_owned(),
        tokenizer: TokenizerDeclaration {
            kind: TokenizerKind::Embedded,
            model: "llama".to_owned(),
        },
        artifacts: vec![ArtifactDescriptor {
            id: ArtifactId::new("weights".to_owned()).expect("fixture artifact is valid"),
            uri: "https://fixtures.example/weights.gguf".to_owned(),
            content_sha256: format!("sha256:{}", "b".repeat(64)),
            size_bytes: 1,
        }],
        publisher_key_id: "ed25519:fixture".to_owned(),
        license: "MIT".to_owned(),
        provenance: "fixture".to_owned(),
        execution_plan: None,
        runtime_profile: None,
    }
}

fn sharded_manifest() -> ModelManifest {
    let mut manifest = local_manifest();
    let shard = ShardSpec::new(
        ShardId::new("range".to_owned()).expect("fixture shard is valid"),
        ArtifactId::new("weights".to_owned()).expect("fixture artifact is valid"),
        LayerRange::new(0, 1).expect("fixture range is valid"),
        1,
    )
    .expect("fixture shard is valid");
    manifest.schema_version = 2;
    manifest.execution_plan = Some(
        ShardPlan::new(
            synapseflow_domain::ExecutionStrategy::layer_range(),
            vec![shard],
        )
        .expect("fixture plan is valid"),
    );
    manifest.runtime_profile = Some(LOOM_RUNTIME_PROFILE.to_owned());
    manifest
}

#[test]
fn selects_the_runtime_only_from_the_verified_manifest_profile() {
    let manifest = sharded_manifest();
    let output = GenerationOutput::from_tokens(vec![GeneratedToken {
        id: 7,
        text: "seven".to_owned(),
    }]);
    let sharded = Arc::new(ShardedRuntime {
        called: Mutex::new(false),
        output: output.clone(),
    });
    let orchestrator = GenerationOrchestrator::new(
        Arc::new(InMemoryModelRegistry::with_manifest(manifest.clone())),
        Arc::new(VerifiedArtifacts),
        Arc::new(InMemoryModelBackend::default()),
        Some(sharded.clone()),
        Arc::new(InMemoryAuditSink::default()),
    );

    assert_eq!(
        orchestrator
            .generate(request(manifest.reference))
            .expect("sharded route succeeds"),
        output
    );
    assert!(*sharded
        .called
        .lock()
        .expect("fixture mutex should be available"));
}

#[test]
fn rejects_a_sharded_profile_when_its_runtime_is_not_composed() {
    let manifest = sharded_manifest();
    let orchestrator = GenerationOrchestrator::new(
        Arc::new(InMemoryModelRegistry::with_manifest(manifest.clone())),
        Arc::new(VerifiedArtifacts),
        Arc::new(InMemoryModelBackend::default()),
        None,
        Arc::new(InMemoryAuditSink::default()),
    );

    assert!(matches!(
        orchestrator.generate(request(manifest.reference)),
        Err(DomainError::BackendUnavailable)
    ));
}
