//! Semantic validation and conversion from wire data to domain data.

use crate::{
    ArtifactDescriptor, ArtifactId, DomainError, DomainResult, ModelFormat, ModelManifest,
    ModelReference, TokenizerDeclaration, TokenizerKind, MANIFEST_SCHEMA_VERSION,
};

use super::schema::{ArtifactWire, ManifestWire};

pub(super) fn into_manifest(
    reference: ModelReference,
    wire: ManifestWire,
) -> DomainResult<ModelManifest> {
    validate_required_text(&wire.model_id)?;
    validate_required_text(&wire.model_version)?;
    validate_required_text(&wire.license)?;
    validate_required_text(&wire.provenance)?;

    if wire.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(DomainError::ManifestUnsupported);
    }
    if wire.format != "gguf"
        || wire.architecture != "llama"
        || wire.quantization != "Q5_K_M"
        || wire.tokenizer.kind != "embedded"
        || wire.tokenizer.model != "llama"
        || !wire.publisher_key_id.starts_with("ed25519:")
        || wire.artifacts.len() != 1
    {
        return Err(DomainError::ManifestUnsupported);
    }

    let artifacts = wire
        .artifacts
        .into_iter()
        .map(into_artifact)
        .collect::<DomainResult<Vec<_>>>()?;

    Ok(ModelManifest {
        reference,
        schema_version: wire.schema_version,
        model_id: wire.model_id,
        model_version: wire.model_version,
        format: ModelFormat::Gguf,
        architecture: wire.architecture,
        quantization: wire.quantization,
        tokenizer: TokenizerDeclaration {
            kind: TokenizerKind::Embedded,
            model: wire.tokenizer.model,
        },
        artifacts,
        publisher_key_id: wire.publisher_key_id,
        license: wire.license,
        provenance: wire.provenance,
    })
}

fn into_artifact(wire: ArtifactWire) -> DomainResult<ArtifactDescriptor> {
    if wire.size_bytes == 0
        || !wire.uri.starts_with("https://")
        || wire.uri.contains(char::is_whitespace)
        || !is_sha256(&wire.content_sha256)
    {
        return Err(DomainError::ManifestInvalid);
    }

    Ok(ArtifactDescriptor {
        id: ArtifactId::new(wire.artifact_id)?,
        uri: wire.uri,
        content_sha256: wire.content_sha256,
        size_bytes: wire.size_bytes,
    })
}

fn validate_required_text(value: &str) -> DomainResult<()> {
    if value.is_empty() || value.len() > 256 || value.contains(char::is_control) {
        return Err(DomainError::ManifestInvalid);
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hash| {
        hash.len() == 64
            && hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}
