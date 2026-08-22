use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DomainError, GenerationPolicy, ModelFormat, ModelManifest,
    ModelReference, TokenizerDeclaration, TokenizerKind, MANIFEST_SCHEMA_VERSION,
};

use crate::compatibility::{sampler_seed, validate_context, validate_manifest, MAX_CONTEXT_TOKENS};

fn manifest() -> ModelManifest {
    ModelManifest {
        reference: ModelReference::parse(format!(
            "registry://fixtures/backend@sha256:{}",
            "a".repeat(64)
        ))
        .expect("test reference should be valid"),
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
fn accepts_only_the_verified_local_compatibility_tuple() {
    assert!(validate_manifest(&manifest()).is_ok());

    let mut unsupported = manifest();
    unsupported.quantization = "Q4_K_M".to_owned();
    assert!(matches!(
        validate_manifest(&unsupported),
        Err(DomainError::BackendIncompatible)
    ));
}

#[test]
fn rejects_context_overflow_without_silent_truncation() {
    let policy = GenerationPolicy::new(16, 0.7, 0.9, 42).expect("policy should be valid");

    assert_eq!(
        validate_context(10, &policy, MAX_CONTEXT_TOKENS).expect("context should fit"),
        MAX_CONTEXT_TOKENS
    );
    assert!(matches!(
        validate_context(MAX_CONTEXT_TOKENS - 15, &policy, MAX_CONTEXT_TOKENS),
        Err(DomainError::GenerationPolicyInvalid)
    ));
}

#[test]
fn folds_the_full_u64_seed_deterministically() {
    assert_eq!(sampler_seed(42), 42);
    assert_ne!(sampler_seed(0x0000_0001_0000_0000), sampler_seed(0));
    assert_eq!(sampler_seed(u64::MAX), u32::MAX ^ u32::MAX);
}
