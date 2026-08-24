use synapseflow_domain::{
    DomainError, DomainResult, ExecutionStrategy, ModelFormat, ModelManifest, ShardSpec,
    LOOM_RUNTIME_PROFILE, LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION,
};

pub(crate) fn validate_model(manifest: &ModelManifest) -> DomainResult<()> {
    let valid = manifest.schema_version == LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION
        && manifest.format == ModelFormat::Gguf
        && manifest.architecture == "llama"
        && manifest.quantization == "Q5_K_M"
        && manifest.tokenizer.is_embedded_llama()
        && manifest.runtime_profile.as_deref() == Some(LOOM_RUNTIME_PROFILE)
        && manifest
            .execution_plan
            .as_ref()
            .is_some_and(|plan| plan.strategy == ExecutionStrategy::layer_range());
    if valid {
        Ok(())
    } else {
        Err(DomainError::BackendIncompatible)
    }
}

pub(crate) fn is_final_stage(manifest: &ModelManifest, shard: &ShardSpec) -> DomainResult<bool> {
    let plan = manifest
        .execution_plan
        .as_ref()
        .ok_or(DomainError::ShardPlanInvalid)?;
    let position = plan
        .shards
        .iter()
        .position(|candidate| candidate == shard)
        .ok_or(DomainError::ShardPlanInvalid)?;
    Ok(position + 1 == plan.shards.len())
}
