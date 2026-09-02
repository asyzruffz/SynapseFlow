use std::cmp::Reverse;

use synapseflow_domain::{DomainError, DomainResult, GeneratedToken};
use synapseflow_ports::{ModelTokenizer, VerifiedModel};

use crate::loom::LoomArchive;

/// Tokenizer adapter for the embedded Llama GGUF vocabulary selected by a
/// verified manifest.
#[derive(Default)]
pub struct LoomTokenizer;

impl LoomTokenizer {
    pub fn new() -> Self {
        Self
    }

    fn tokens(&self, model: &VerifiedModel) -> DomainResult<Vec<String>> {
        let artifact = model
            .manifest
            .artifacts
            .first()
            .ok_or(DomainError::TokenizerFailure)?;
        LoomArchive::open(model.artifact_path(&artifact.id)?)?.tokenizer_tokens()
    }
}

impl ModelTokenizer for LoomTokenizer {
    fn encode(&self, model: &VerifiedModel, text: &str) -> DomainResult<Vec<u32>> {
        let mut entries = self
            .tokens(model)?
            .into_iter()
            .enumerate()
            .filter_map(|(id, token)| {
                let bytes = token_bytes(&token)?;
                (!bytes.is_empty()).then_some((u32::try_from(id).ok()?, bytes))
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|(_, bytes)| Reverse(bytes.len()));

        let mut remaining = text.as_bytes();
        let mut token_ids = Vec::new();
        while !remaining.is_empty() {
            let Some((id, bytes)) = entries
                .iter()
                .find(|(_, bytes)| remaining.starts_with(bytes.as_slice()))
            else {
                return Err(DomainError::TokenizerFailure);
            };
            token_ids.push(*id);
            remaining = &remaining[bytes.len()..];
        }
        (!token_ids.is_empty())
            .then_some(token_ids)
            .ok_or(DomainError::TokenizerFailure)
    }

    fn decode(&self, model: &VerifiedModel, token_id: u32) -> DomainResult<GeneratedToken> {
        let token = self
            .tokens(model)?
            .get(usize::try_from(token_id).map_err(|_| DomainError::TokenizerFailure)?)
            .cloned()
            .ok_or(DomainError::TokenizerFailure)?;
        let bytes = token_bytes(&token).ok_or(DomainError::TokenizerFailure)?;
        let text = String::from_utf8(bytes).map_err(|_| DomainError::TokenizerFailure)?;
        Ok(GeneratedToken { id: token_id, text })
    }
}

fn token_bytes(token: &str) -> Option<Vec<u8>> {
    if let Some(hex) = token
        .strip_prefix("<0x")
        .and_then(|value| value.strip_suffix('>'))
    {
        return u8::from_str_radix(hex, 16).ok().map(|byte| vec![byte]);
    }
    Some(token.replace('▁', " ").into_bytes())
}
