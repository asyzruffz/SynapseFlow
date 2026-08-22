use synapseflow_domain::{DomainError, DomainResult, GenerationPolicy, ModelManifest};

pub(super) const MAX_CONTEXT_TOKENS: usize = 2_048;

pub(super) fn validate_manifest(manifest: &ModelManifest) -> DomainResult<()> {
    if manifest.supports_verified_local_inference() {
        Ok(())
    } else {
        Err(DomainError::BackendIncompatible)
    }
}

pub(super) fn validate_context(
    prompt_tokens: usize,
    policy: &GenerationPolicy,
    model_context_limit: usize,
) -> DomainResult<usize> {
    let limit = MAX_CONTEXT_TOKENS.min(model_context_limit);
    let requested = prompt_tokens
        .checked_add(usize::from(policy.max_tokens))
        .ok_or(DomainError::GenerationPolicyInvalid)?;
    if prompt_tokens == 0 || requested > limit {
        return Err(DomainError::GenerationPolicyInvalid);
    }
    Ok(limit)
}

/// Folds the public u64 seed into llama.cpp's stable u32 sampler seed without truncation bias.
pub(super) fn sampler_seed(seed: u64) -> u32 {
    (seed ^ (seed >> 32)) as u32
}
