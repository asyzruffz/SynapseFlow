use std::{sync::Arc, time::Duration};

use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DomainError, DomainResult, GeneratedToken, GenerationOutput,
    GenerationPolicy, GenerationRequest, ModelFormat, ModelManifest, ModelReference,
    TokenizerDeclaration, TokenizerKind, MANIFEST_SCHEMA_VERSION,
};
use synapseflow_ports::{
    ArtifactStore, AuditEvent, AuditSink, CacheEntryState, CachedArtifactInspection, ModelBackend,
    ModelCacheInspection, ModelRegistry, VerifiedModel,
};

use crate::GenerationOrchestrator;

struct Registry(ModelManifest);

impl ModelRegistry for Registry {
    fn resolve(&self, _: &ModelReference) -> DomainResult<ModelManifest> {
        Ok(self.0.clone())
    }
}

struct Store;

impl ArtifactStore for Store {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel> {
        Ok(VerifiedModel::without_cached_artifacts(manifest.clone()))
    }

    fn inspect(&self, manifest: &ModelManifest) -> DomainResult<ModelCacheInspection> {
        Ok(ModelCacheInspection {
            reference: manifest.reference.clone(),
            publisher_key_id: manifest.publisher_key_id.clone(),
            license: manifest.license.clone(),
            provenance: manifest.provenance.clone(),
            artifacts: manifest
                .artifacts
                .iter()
                .map(|artifact| CachedArtifactInspection {
                    artifact_id: artifact.id.clone(),
                    content_sha256: artifact.content_sha256.clone(),
                    size_bytes: artifact.size_bytes,
                    state: CacheEntryState::Cached,
                })
                .collect(),
        })
    }
}

struct Backend;

impl ModelBackend for Backend {
    fn generate(&self, _: &VerifiedModel, _: &GenerationRequest) -> DomainResult<GenerationOutput> {
        Ok(GenerationOutput::from_tokens(vec![GeneratedToken {
            id: 1,
            text: "test".to_owned(),
        }]))
    }
}

struct Audit;

impl AuditSink for Audit {
    fn record(&self, _: AuditEvent) -> DomainResult<()> {
        Ok(())
    }
}

fn request() -> GenerationRequest {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/test@sha256:{}",
        "a".repeat(64)
    ))
    .expect("reference should be valid");
    let policy = GenerationPolicy::new(1, 0.7, 0.9, 42).expect("policy should be valid");
    GenerationRequest::new(reference, "prompt".to_owned(), policy).expect("request should be valid")
}

fn manifest(reference: ModelReference) -> ModelManifest {
    ModelManifest {
        reference,
        schema_version: MANIFEST_SCHEMA_VERSION,
        model_id: "test".to_owned(),
        model_version: "v1".to_owned(),
        format: ModelFormat::Gguf,
        architecture: "llama".to_owned(),
        quantization: "Q5_K_M".to_owned(),
        tokenizer: TokenizerDeclaration {
            kind: TokenizerKind::Embedded,
            model: "llama".to_owned(),
        },
        artifacts: vec![ArtifactDescriptor {
            id: ArtifactId::new("weights".to_owned()).expect("artifact id should be valid"),
            uri: "https://fixtures.example/weights.gguf".to_owned(),
            content_sha256: format!("sha256:{}", "b".repeat(64)),
            size_bytes: 1,
        }],
        publisher_key_id: "ed25519:test".to_owned(),
        license: "Apache-2.0".to_owned(),
        provenance: "test".to_owned(),
        execution_plan: None,
        runtime_profile: None,
    }
}

#[test]
fn orchestrator_uses_only_in_memory_port_implementations() {
    let request = request();
    let orchestrator = GenerationOrchestrator::new(
        Arc::new(Registry(manifest(request.model.clone()))),
        Arc::new(Store),
        Arc::new(Backend),
        None,
        Arc::new(Audit),
    );

    let output = orchestrator
        .generate(request)
        .expect("generation should succeed");

    assert_eq!(output.text, "test");
}

#[test]
fn invalid_policy_is_a_typed_domain_error() {
    assert!(matches!(
        GenerationPolicy::new(0, 0.7, 0.9, 42),
        Err(DomainError::GenerationPolicyInvalid)
    ));
    assert!(matches!(
        GenerationPolicy::new(1, 0.0, 0.9, 42),
        Err(DomainError::GenerationPolicyInvalid)
    ));
}

#[test]
fn expired_deadline_is_rejected_before_the_workflow_uses_ports() {
    let request = request()
        .with_deadline_after(Duration::from_millis(1))
        .expect("test deadline should be valid");
    std::thread::sleep(Duration::from_millis(2));
    let orchestrator = GenerationOrchestrator::new(
        Arc::new(Registry(manifest(request.model.clone()))),
        Arc::new(Store),
        Arc::new(Backend),
        None,
        Arc::new(Audit),
    );

    assert!(matches!(
        orchestrator.generate(request),
        Err(DomainError::DeadlineExceeded)
    ));
}
