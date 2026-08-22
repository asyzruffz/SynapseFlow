use synapseflow_domain::{
    ArtifactDescriptor, ArtifactId, DomainError, ModelFormat, ModelManifest, ModelReference,
    TokenizerDeclaration, TokenizerKind, MANIFEST_SCHEMA_VERSION,
};

use super::VerifiedModel;

fn manifest() -> ModelManifest {
    ModelManifest {
        reference: ModelReference::parse(format!(
            "registry://test/model@sha256:{}",
            "a".repeat(64)
        ))
        .expect("the fixture reference should be valid"),
        schema_version: MANIFEST_SCHEMA_VERSION,
        model_id: "test-model".to_owned(),
        model_version: "1.0.0".to_owned(),
        format: ModelFormat::Gguf,
        architecture: "llama".to_owned(),
        quantization: "Q5_K_M".to_owned(),
        tokenizer: TokenizerDeclaration {
            kind: TokenizerKind::Embedded,
            model: "llama".to_owned(),
        },
        artifacts: vec![ArtifactDescriptor {
            id: ArtifactId::new("weights.gguf".to_owned())
                .expect("the fixture artifact identifier should be valid"),
            uri: "https://fixtures.invalid/weights.gguf".to_owned(),
            content_sha256: "b".repeat(64),
            size_bytes: 1,
        }],
        publisher_key_id: "fixture-publisher".to_owned(),
        license: "Apache-2.0".to_owned(),
        provenance: "unit-test fixture".to_owned(),
    }
}

#[test]
fn rejects_cached_paths_that_do_not_match_manifest_artifacts() {
    let error = VerifiedModel::with_cached_artifacts(manifest(), Vec::new())
        .expect_err("cache paths must match declared artifacts");

    assert!(matches!(error, DomainError::CacheFailure));
}

#[test]
fn reports_missing_runtime_artifact_path() {
    let error = VerifiedModel::without_cached_artifacts(manifest())
        .primary_artifact_path()
        .expect_err("test models must not expose a runtime artifact path");

    assert!(matches!(error, DomainError::ArtifactUnavailable));
}
