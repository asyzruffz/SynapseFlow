use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use synapseflow_domain::{GeneratedToken, GenerationOutput, GenerationPolicy, ModelReference};

const ADAPTER_VERSION: &str = "llama-cpp-2=0.1.154";
const PROMPT: &str = "The capital of France is";
const MAX_TOKENS: u16 = 16;
const TEMPERATURE: f32 = 0.7;
const TOP_P: f32 = 0.9;
const SEED: u64 = 42;

/// Persisted, exact output and provenance for one approved fixture execution.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct ReferenceVector {
    fixture_id: String,
    manifest_reference: String,
    adapter_version: String,
    llama_cpp_revision: String,
    operating_system: String,
    cpu_architecture: String,
    request: RequestVector,
    tokens: Vec<TokenVector>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RequestVector {
    prompt: String,
    max_tokens: u16,
    temperature: f32,
    top_p: f32,
    seed: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TokenVector {
    id: u32,
    text: String,
}

impl ReferenceVector {
    pub(super) fn read(path: &Path) -> Result<Self, String> {
        let document = fs::read(path)
            .map_err(|error| format!("cannot read reference vector {}: {error}", path.display()))?;
        serde_json::from_slice(&document)
            .map_err(|error| format!("reference vector {} is invalid: {error}", path.display()))
    }

    pub(super) fn candidate(
        fixture_id: &str,
        reference: &ModelReference,
        llama_cpp_revision: String,
        output: &GenerationOutput,
    ) -> Self {
        Self {
            fixture_id: fixture_id.to_owned(),
            manifest_reference: reference.as_str().to_owned(),
            adapter_version: ADAPTER_VERSION.to_owned(),
            llama_cpp_revision,
            operating_system: std::env::consts::OS.to_owned(),
            cpu_architecture: std::env::consts::ARCH.to_owned(),
            request: RequestVector {
                prompt: PROMPT.to_owned(),
                max_tokens: MAX_TOKENS,
                temperature: TEMPERATURE,
                top_p: TOP_P,
                seed: SEED,
            },
            tokens: output.tokens.iter().map(TokenVector::from).collect(),
        }
    }

    pub(super) fn policy() -> Result<GenerationPolicy, String> {
        GenerationPolicy::new(MAX_TOKENS, TEMPERATURE, TOP_P, SEED)
            .map_err(|error| format!("the fixed acceptance policy is invalid: {error}"))
    }

    pub(super) fn prompt() -> &'static str {
        PROMPT
    }

    pub(super) fn assert_matches(
        &self,
        fixture_id: &str,
        reference: &ModelReference,
        llama_cpp_revision: &str,
        output: &GenerationOutput,
    ) -> Result<(), String> {
        if self.fixture_id != fixture_id
            || self.manifest_reference != reference.as_str()
            || self.adapter_version != ADAPTER_VERSION
            || self.llama_cpp_revision != llama_cpp_revision
            || self.operating_system != std::env::consts::OS
            || self.cpu_architecture != std::env::consts::ARCH
            || self.request.prompt != PROMPT
            || self.request.max_tokens != MAX_TOKENS
            || self.request.temperature != TEMPERATURE
            || self.request.top_p != TOP_P
            || self.request.seed != SEED
        {
            return Err(
                "reference vector metadata does not match the fixed acceptance contract".to_owned(),
            );
        }

        let actual = output
            .tokens
            .iter()
            .map(TokenVector::from)
            .collect::<Vec<_>>();
        if self.tokens != actual {
            return Err(format!(
                "generated token vector differs from the accepted vector; actual output: {}",
                serde_json::to_string(&actual)
                    .map_err(|error| format!("cannot render actual token vector: {error}"))?
            ));
        }
        Ok(())
    }

    pub(super) fn write_new(&self, path: &Path) -> Result<(), String> {
        let document = serde_json::to_vec_pretty(self)
            .map_err(|error| format!("cannot serialize candidate vector: {error}"))?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| {
                format!("cannot create candidate vector {}: {error}", path.display())
            })?;
        use std::io::Write as _;
        output.write_all(&document).map_err(|error| {
            format!("cannot write candidate vector {}: {error}", path.display())
        })?;
        output
            .sync_all()
            .map_err(|error| format!("cannot flush candidate vector {}: {error}", path.display()))
    }
}

impl From<&GeneratedToken> for TokenVector {
    fn from(token: &GeneratedToken) -> Self {
        Self {
            id: token.id,
            text: token.text.clone(),
        }
    }
}
