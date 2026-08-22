use std::time::Duration;

use serde::{Deserialize, Serialize};
use synapseflow_domain::{
    GeneratedToken, GenerationOutput, GenerationPolicy, GenerationRequest, ModelReference,
};

use super::error::ApiError;
use crate::LocalGeneration;

pub(super) const MAX_REQUEST_BYTES: usize = 16 * 1024;

#[derive(Deserialize)]
pub(super) struct GenerateRequest {
    model: String,
    prompt: String,
    max_tokens: u16,
    temperature: f32,
    top_p: f32,
    seed: u64,
    deadline_ms: Option<u64>,
}

impl GenerateRequest {
    pub(super) fn into_domain(self) -> Result<GenerationRequest, ApiError> {
        let reference = ModelReference::parse(self.model).map_err(ApiError::from)?;
        let policy =
            GenerationPolicy::new(self.max_tokens, self.temperature, self.top_p, self.seed)
                .map_err(ApiError::from)?;
        let request =
            GenerationRequest::new(reference, self.prompt, policy).map_err(ApiError::from)?;
        match self.deadline_ms.map(Duration::from_millis) {
            Some(deadline) => request
                .with_deadline_after(deadline)
                .map_err(ApiError::from),
            None => Ok(request),
        }
    }
}

#[derive(Serialize)]
pub(super) struct ErrorResponse {
    pub(super) code: String,
    pub(super) message: &'static str,
}

#[derive(Serialize)]
pub(super) struct GenerateResponse {
    session_id: String,
    output: OutputResponse,
}

impl From<LocalGeneration> for GenerateResponse {
    fn from(generation: LocalGeneration) -> Self {
        Self {
            session_id: generation.session_id.to_string(),
            output: OutputResponse::from(generation.output),
        }
    }
}

#[derive(Serialize)]
struct OutputResponse {
    text: String,
    tokens: Vec<TokenResponse>,
}

impl From<GenerationOutput> for OutputResponse {
    fn from(output: GenerationOutput) -> Self {
        Self {
            text: output.text,
            tokens: output.tokens.into_iter().map(TokenResponse::from).collect(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct TokenResponse {
    id: u32,
    text: String,
}

impl From<GeneratedToken> for TokenResponse {
    fn from(token: GeneratedToken) -> Self {
        Self {
            id: token.id,
            text: token.text,
        }
    }
}

#[derive(Serialize)]
pub(super) struct CompletionResponse {
    session_id: String,
}

impl CompletionResponse {
    pub(super) fn new(session_id: String) -> Self {
        Self { session_id }
    }
}
