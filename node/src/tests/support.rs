use std::sync::Arc;

use synapseflow_adapter_in_memory::{
    InMemoryArtifactStore, InMemoryAuditSink, InMemoryModelBackend, InMemoryModelRegistry,
};
use synapseflow_application::GenerationService;
use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, GeneratedToken, GenerationOutput, ModelFormat, ModelManifest,
    ModelReference, TokenizerDeclaration, TokenizerKind, MANIFEST_SCHEMA_VERSION,
};

use crate::LocalNode;

pub(super) fn reference() -> ModelReference {
    ModelReference::parse(format!(
        "registry://fixtures/test@sha256:{}",
        "a".repeat(64)
    ))
    .expect("test model reference should be valid")
}

pub(super) fn node() -> LocalNode {
    let reference = reference();
    let manifest = ModelManifest {
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
            id: ArtifactId::new("weights".to_owned()).expect("test artifact ID should be valid"),
            uri: "https://fixtures.example/weights.gguf".to_owned(),
            content_sha256: format!("sha256:{}", "b".repeat(64)),
            size_bytes: 1,
        }],
        publisher_key_id: "ed25519:test".to_owned(),
        license: "Apache-2.0".to_owned(),
        provenance: "test".to_owned(),
    };
    let output = GenerationOutput::from_tokens(vec![
        GeneratedToken {
            id: 10,
            text: "hello".to_owned(),
        },
        GeneratedToken {
            id: 11,
            text: " world".to_owned(),
        },
    ]);
    LocalNode::new(GenerationService::new(
        Arc::new(InMemoryModelRegistry::with_manifest(manifest)),
        Arc::new(InMemoryArtifactStore),
        Arc::new(InMemoryModelBackend::with_output(output)),
        Arc::new(InMemoryAuditSink::default()),
    ))
}
