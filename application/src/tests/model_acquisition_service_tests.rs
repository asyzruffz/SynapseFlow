use std::sync::Arc;

use synapseflow_adapter_in_memory::{
    InMemoryArtifactStore, InMemoryAuditSink, InMemoryModelRegistry,
};
use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, ModelFormat, ModelManifest, ModelReference,
    TokenizerDeclaration, TokenizerKind, MANIFEST_SCHEMA_VERSION,
};
use synapseflow_ports::AuditEvent;

use crate::ModelAcquisitionService;

fn manifest() -> ModelManifest {
    let reference = ModelReference::parse(format!(
        "registry://fixtures/acquisition@sha256:{}",
        "a".repeat(64)
    ))
    .expect("test reference should be valid");
    ModelManifest {
        reference,
        schema_version: MANIFEST_SCHEMA_VERSION,
        model_id: "tinyllama".to_owned(),
        model_version: "test".to_owned(),
        format: ModelFormat::Gguf,
        architecture: "llama".to_owned(),
        quantization: "Q5_K_M".to_owned(),
        tokenizer: TokenizerDeclaration {
            kind: TokenizerKind::Embedded,
            model: "llama".to_owned(),
        },
        artifacts: vec![ArtifactDescriptor {
            id: ArtifactId::new("weights".to_owned()).expect("test ID should be valid"),
            uri: "https://fixtures.example/weights.gguf".to_owned(),
            content_sha256: format!("sha256:{}", "b".repeat(64)),
            size_bytes: 1,
        }],
        publisher_key_id: "ed25519:test".to_owned(),
        license: "Apache-2.0".to_owned(),
        provenance: "fixture:test".to_owned(),
    }
}

#[test]
fn acquisition_returns_safe_cache_and_provenance_information() {
    let manifest = manifest();
    let audit = Arc::new(InMemoryAuditSink::default());
    let service = ModelAcquisitionService::new(
        Arc::new(InMemoryModelRegistry::with_manifest(manifest.clone())),
        Arc::new(InMemoryArtifactStore),
        audit.clone(),
    );

    let inspection = service
        .acquire_and_inspect(manifest.reference.clone())
        .expect("in-memory acquisition should succeed");

    assert_eq!(inspection.reference, manifest.reference);
    assert_eq!(inspection.publisher_key_id, "ed25519:test");
    assert_eq!(inspection.artifacts.len(), 1);
    assert!(matches!(
        audit
            .events()
            .expect("audit events should be readable")
            .as_slice(),
        [
            AuditEvent::ManifestVerified { .. },
            AuditEvent::ArtifactsCached { .. }
        ]
    ));
}
