use crate::{DomainError, DomainResult};

/// Caller-selected deterministic sampling policy.
#[derive(Clone, Debug, PartialEq)]
pub struct GenerationPolicy {
    pub max_tokens: u16,
    pub temperature: f32,
    pub top_p: f32,
    pub seed: u64,
}

impl GenerationPolicy {
    pub fn new(max_tokens: u16, temperature: f32, top_p: f32, seed: u64) -> DomainResult<Self> {
        if max_tokens == 0
            || max_tokens > 256
            || !temperature.is_finite()
            || temperature <= 0.0
            || temperature > 2.0
            || !top_p.is_finite()
            || top_p <= 0.0
            || top_p > 1.0
        {
            return Err(DomainError::GenerationPolicyInvalid);
        }

        Ok(Self {
            max_tokens,
            temperature,
            top_p,
            seed,
        })
    }
}
