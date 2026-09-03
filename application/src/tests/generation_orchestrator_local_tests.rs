use std::{sync::Arc, time::Duration};

use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DomainError, DomainResult, GeneratedToken, GenerationEvent,
    GenerationPolicy, GenerationRequest, ModelFormat, ModelManifest, ModelReference,
    TokenizerDeclaration, TokenizerKind, MANIFEST_SCHEMA_VERSION,
};
use synapseflow_ports::{
    ArtifactStore, AuditEvent, AuditSink, CacheEntryState, CachedArtifactInspection,
    ExecutionCancellation, GeneratedTokenSink, GenerationEventSink, ModelBackend,
    ModelCacheInspection, ModelRegistry, NeverCancelled, VerifiedModel,
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
    fn generate(
        &self,
        _: &VerifiedModel,
        _: &GenerationRequest,
        cancellation: &dyn ExecutionCancellation,
        tokens: &mut dyn GeneratedTokenSink,
    ) -> DomainResult<synapseflow_domain::GenerationTerminal> {
        if cancellation.is_cancelled() {
            return Ok(synapseflow_domain::GenerationTerminal::Cancelled);
        }
        tokens.emit_token(GeneratedToken {
            id: 1,
            text: "test".to_owned(),
        })?;
        Ok(synapseflow_domain::GenerationTerminal::Completed { token_count: 1 })
    }
}

struct Audit;

impl AuditSink for Audit {
    fn record(&self, _: AuditEvent) -> DomainResult<()> {
        Ok(())
    }
}

#[derive(Default)]
struct RecordingEvents(Vec<GenerationEvent>);

impl GenerationEventSink for RecordingEvents {
    fn emit(&mut self, event: GenerationEvent) -> DomainResult<()> {
        self.0.push(event);
        Ok(())
    }
}

struct Cancelled;

impl ExecutionCancellation for Cancelled {
    fn is_cancelled(&self) -> bool {
        true
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
fn orchestrator_emits_ordered_tokens_and_one_terminal_event() {
    let request = request();
    let orchestrator = GenerationOrchestrator::new(
        Arc::new(Registry(manifest(request.model.clone()))),
        Arc::new(Store),
        Arc::new(Backend),
        None,
        Arc::new(Audit),
    );
    let mut events = RecordingEvents::default();

    orchestrator
        .generate_live(request, &NeverCancelled, &mut events)
        .expect("streamed generation should succeed");

    assert_eq!(
        events.0,
        vec![
            GenerationEvent::Token(GeneratedToken {
                id: 1,
                text: "test".to_owned(),
            }),
            GenerationEvent::Completed { token_count: 1 },
        ]
    );
}

#[test]
fn already_cancelled_work_emits_only_a_cancelled_terminal_event() {
    let request = request();
    let orchestrator = GenerationOrchestrator::new(
        Arc::new(Registry(manifest(request.model.clone()))),
        Arc::new(Store),
        Arc::new(Backend),
        None,
        Arc::new(Audit),
    );
    let mut events = RecordingEvents::default();

    orchestrator
        .generate_live(request, &Cancelled, &mut events)
        .expect("pre-cancelled generation should resolve safely");

    assert_eq!(events.0, vec![GenerationEvent::Cancelled]);
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
