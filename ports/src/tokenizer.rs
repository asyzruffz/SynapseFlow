use synapseflow_domain::{DomainResult, GeneratedToken};

use crate::VerifiedModel;

/// Encodes and decodes text through the tokenizer embedded in a verified model.
///
/// Implementations may use runtime-specific metadata, but must never resolve a
/// model or choose a generation policy.
pub trait ModelTokenizer: Send + Sync {
    fn encode(&self, model: &VerifiedModel, text: &str) -> DomainResult<Vec<u32>>;

    fn decode(&self, model: &VerifiedModel, token_id: u32) -> DomainResult<GeneratedToken>;
}
