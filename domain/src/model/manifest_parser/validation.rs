//! Semantic validation and conversion from wire data to domain data.

use crate::{
    ArtifactDescriptor, ArtifactId, DomainError, DomainResult, ExecutionStrategy, LayerRange,
    ModelFormat, ModelManifest, ModelReference, ShardId, ShardPlan, ShardSpec,
    TokenizerDeclaration, TokenizerKind, LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION,
    MANIFEST_SCHEMA_VERSION,
};

use super::schema::{ArtifactWire, ManifestDocument, ManifestWire, ShardWire, ShardedManifestWire};

pub(super) fn into_manifest(
    reference: ModelReference,
    wire: ManifestDocument,
) -> DomainResult<ModelManifest> {
    match wire {
        ManifestDocument::Full(wire) => into_full(reference, wire),
        ManifestDocument::Sharded(wire) => into_sharded(reference, wire),
    }
}

fn into_full(reference: ModelReference, wire: ManifestWire) -> DomainResult<ModelManifest> {
    validate_common(
        &wire.model_id,
        &wire.model_version,
        &wire.license,
        &wire.provenance,
    )?;
    if wire.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(DomainError::ManifestUnsupported);
    }
    let artifacts = validate_compatibility_and_artifacts(
        &wire.format,
        &wire.architecture,
        &wire.quantization,
        &wire.tokenizer.kind,
        &wire.tokenizer.model,
        &wire.publisher_key_id,
        wire.artifacts,
        true,
    )?;
    Ok(model_manifest(
        reference,
        wire.schema_version,
        wire.model_id,
        wire.model_version,
        wire.architecture,
        wire.quantization,
        wire.tokenizer.model,
        artifacts,
        wire.publisher_key_id,
        wire.license,
        wire.provenance,
        None,
        None,
    ))
}

fn into_sharded(
    reference: ModelReference,
    wire: ShardedManifestWire,
) -> DomainResult<ModelManifest> {
    validate_common(
        &wire.model_id,
        &wire.model_version,
        &wire.license,
        &wire.provenance,
    )?;
    if wire.schema_version != LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION
        || wire.execution.runtime_profile != "llama-layer-range-v1"
    {
        return Err(DomainError::ManifestUnsupported);
    }
    validate_required_text(&wire.execution.runtime_profile)?;
    let artifacts = validate_compatibility_and_artifacts(
        &wire.format,
        &wire.architecture,
        &wire.quantization,
        &wire.tokenizer.kind,
        &wire.tokenizer.model,
        &wire.publisher_key_id,
        wire.artifacts,
        false,
    )?;
    let execution_plan = into_shard_plan(
        wire.execution.strategy,
        wire.execution.layer_count,
        wire.execution.shards,
        &artifacts,
    )?;
    Ok(model_manifest(
        reference,
        wire.schema_version,
        wire.model_id,
        wire.model_version,
        wire.architecture,
        wire.quantization,
        wire.tokenizer.model,
        artifacts,
        wire.publisher_key_id,
        wire.license,
        wire.provenance,
        Some(execution_plan),
        Some(wire.execution.runtime_profile),
    ))
}

#[allow(clippy::too_many_arguments)]
fn model_manifest(
    reference: ModelReference,
    schema_version: u16,
    model_id: String,
    model_version: String,
    architecture: String,
    quantization: String,
    tokenizer_model: String,
    artifacts: Vec<ArtifactDescriptor>,
    publisher_key_id: String,
    license: String,
    provenance: String,
    execution_plan: Option<ShardPlan>,
    runtime_profile: Option<String>,
) -> ModelManifest {
    ModelManifest {
        reference,
        schema_version,
        model_id,
        model_version,
        format: ModelFormat::Gguf,
        architecture,
        quantization,
        tokenizer: TokenizerDeclaration {
            kind: TokenizerKind::Embedded,
            model: tokenizer_model,
        },
        artifacts,
        publisher_key_id,
        license,
        provenance,
        execution_plan,
        runtime_profile,
    }
}

fn into_shard_plan(
    strategy: String,
    layer_count: u32,
    shards: Vec<ShardWire>,
    artifacts: &[ArtifactDescriptor],
) -> DomainResult<ShardPlan> {
    let strategy = ExecutionStrategy::new(strategy)?;
    let shards = shards
        .into_iter()
        .map(|shard| into_shard(shard, artifacts))
        .collect::<DomainResult<Vec<_>>>()?;
    let plan = ShardPlan::new(strategy, shards)?;
    if layer_count == 0 || plan.total_layers() != layer_count {
        return Err(DomainError::ShardPlanInvalid);
    }
    Ok(plan)
}

fn into_shard(wire: ShardWire, artifacts: &[ArtifactDescriptor]) -> DomainResult<ShardSpec> {
    let artifact_id = ArtifactId::new(wire.artifact_id)?;
    if !artifacts.iter().any(|artifact| artifact.id == artifact_id) {
        return Err(DomainError::ManifestInvalid);
    }
    ShardSpec::new(
        ShardId::new(wire.shard_id)?,
        artifact_id,
        LayerRange::new(wire.layer_start, wire.layer_end_exclusive)?,
        wire.minimum_replicas,
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_compatibility_and_artifacts(
    format: &str,
    architecture: &str,
    quantization: &str,
    tokenizer_kind: &str,
    tokenizer_model: &str,
    publisher_key_id: &str,
    artifacts: Vec<ArtifactWire>,
    require_single_artifact: bool,
) -> DomainResult<Vec<ArtifactDescriptor>> {
    if format != "gguf"
        || architecture != "llama"
        || quantization != "Q5_K_M"
        || tokenizer_kind != "embedded"
        || tokenizer_model != "llama"
        || !publisher_key_id.starts_with("ed25519:")
        || artifacts.is_empty()
        || (require_single_artifact && artifacts.len() != 1)
    {
        return Err(DomainError::ManifestUnsupported);
    }
    let artifacts = artifacts
        .into_iter()
        .map(into_artifact)
        .collect::<DomainResult<Vec<_>>>()?;
    let unique = artifacts.iter().enumerate().all(|(index, artifact)| {
        artifacts[..index]
            .iter()
            .all(|other| other.id != artifact.id)
    });
    if !unique {
        return Err(DomainError::ManifestInvalid);
    }
    Ok(artifacts)
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

fn validate_common(
    model_id: &str,
    model_version: &str,
    license: &str,
    provenance: &str,
) -> DomainResult<()> {
    validate_required_text(model_id)?;
    validate_required_text(model_version)?;
    validate_required_text(license)?;
    validate_required_text(provenance)
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
