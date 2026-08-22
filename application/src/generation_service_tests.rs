use std::sync::Arc;

use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DomainError, DomainResult, GeneratedToken, GenerationOutput,
    GenerationPolicy, GenerationRequest, ModelFormat, ModelManifest, ModelReference,
};
use synapseflow_ports::{
    ArtifactStore, AuditEvent, AuditSink, ModelBackend, ModelRegistry, VerifiedModel,
};

use crate::GenerationService;

struct Registry(ModelManifest);

impl ModelRegistry for Registry {
    fn resolve(&self, _: &ModelReference) -> DomainResult<ModelManifest> {
        Ok(self.0.clone())
    }
}

struct Store;

impl ArtifactStore for Store {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel> {
        Ok(VerifiedModel {
            manifest: manifest.clone(),
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
        model_id: "test".to_owned(),
        model_version: "v1".to_owned(),
        format: ModelFormat::Gguf,
        architecture: "llama".to_owned(),
        artifacts: vec![ArtifactDescriptor {
            id: ArtifactId::new("weights".to_owned()).expect("artifact id should be valid"),
            content_sha256: format!("sha256:{}", "b".repeat(64)),
            size_bytes: 1,
        }],
    }
}

#[test]
fn service_uses_only_in_memory_port_implementations() {
    let request = request();
    let service = GenerationService::new(
        Arc::new(Registry(manifest(request.model.clone()))),
        Arc::new(Store),
        Arc::new(Backend),
        Arc::new(Audit),
    );

    let output = service
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
