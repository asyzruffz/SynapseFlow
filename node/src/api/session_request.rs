use std::time::Duration;

use axum::http::HeaderMap;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use synapseflow_application::SessionStartRequest;
use synapseflow_domain::{
    AuthenticatedPrincipal, DomainError, GenerationPolicy, GenerationRequest, IdempotencyKey,
    ModelReference,
};
use synapseflow_ports::RequestFingerprint;

/// Public JSON contract for creating one application-owned generation session.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateSessionBody {
    model: String,
    prompt: String,
    max_tokens: u16,
    temperature: f32,
    top_p: f32,
    seed: u64,
    deadline_ms: Option<u64>,
}

/// Parses the bounded optional idempotency key from the HTTP request.
pub(super) fn idempotency_key(headers: &HeaderMap) -> Result<Option<IdempotencyKey>, DomainError> {
    headers
        .get("idempotency-key")
        .map(|value| {
            value
                .to_str()
                .map_err(|_| DomainError::IdempotencyKeyInvalid)
        })
        .transpose()?
        .map(|value| IdempotencyKey::new(value.to_owned()))
        .transpose()
}

/// Builds the transient execution request and payload-free durable start record.
pub(super) fn into_requests(
    principal: AuthenticatedPrincipal,
    body: CreateSessionBody,
    idempotency_key: Option<IdempotencyKey>,
) -> Result<(GenerationRequest, SessionStartRequest), DomainError> {
    let model = ModelReference::parse(body.model)?;
    let policy = GenerationPolicy::new(body.max_tokens, body.temperature, body.top_p, body.seed)?;
    let mut generation = GenerationRequest::new(model.clone(), body.prompt, policy.clone())?;
    if let Some(deadline_ms) = body.deadline_ms {
        generation = generation.with_deadline_after(Duration::from_millis(deadline_ms))?;
    }
    let fingerprint = RequestFingerprint::new(canonical_fingerprint(&generation, body.deadline_ms));
    let start = SessionStartRequest {
        principal,
        model,
        reserved_output_tokens: policy.max_tokens,
        idempotency_key,
        request_fingerprint: Some(fingerprint),
        trace_id: None,
    };
    Ok((generation, start))
}

fn canonical_fingerprint(request: &GenerationRequest, deadline_ms: Option<u64>) -> [u8; 32] {
    let mut digest = Sha256::new();
    append_field(&mut digest, request.model.as_str().as_bytes());
    append_field(&mut digest, request.prompt.as_bytes());
    digest.update(request.policy.max_tokens.to_be_bytes());
    digest.update(request.policy.temperature.to_bits().to_be_bytes());
    digest.update(request.policy.top_p.to_bits().to_be_bytes());
    digest.update(request.policy.seed.to_be_bytes());
    digest.update(deadline_ms.unwrap_or_default().to_be_bytes());
    digest.update([u8::from(deadline_ms.is_some())]);
    digest.finalize().into()
}

fn append_field(digest: &mut Sha256, value: &[u8]) {
    digest.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
mod tests {
    use synapseflow_domain::{
        AuthenticatedPrincipal, GrantedScope, GrantedScopes, PrincipalPseudonym,
    };

    use super::{canonical_fingerprint, into_requests, CreateSessionBody};

    fn principal() -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::new(
            PrincipalPseudonym::new("principal_0001".to_owned()).expect("fixture principal"),
            GrantedScopes::new([GrantedScope::Generate]),
        )
    }

    fn request(prompt: &str) -> CreateSessionBody {
        CreateSessionBody {
            model: format!("registry://fixtures/model@sha256:{}", "a".repeat(64)),
            prompt: prompt.to_owned(),
            max_tokens: 16,
            temperature: 0.7,
            top_p: 0.9,
            seed: 42,
            deadline_ms: Some(1_000),
        }
    }

    #[test]
    fn fingerprints_the_complete_canonical_request_without_retaining_its_prompt() {
        let (first, _) =
            into_requests(principal(), request("first prompt"), None).expect("fixture request");
        let (same, _) =
            into_requests(principal(), request("first prompt"), None).expect("fixture request");
        let (different, _) =
            into_requests(principal(), request("second prompt"), None).expect("fixture request");

        assert_eq!(
            canonical_fingerprint(&first, Some(1_000)),
            canonical_fingerprint(&same, Some(1_000))
        );
        assert_ne!(
            canonical_fingerprint(&first, Some(1_000)),
            canonical_fingerprint(&different, Some(1_000))
        );
    }

    #[test]
    fn rejects_invalid_sampling_before_session_creation() {
        let mut invalid = request("fixture prompt");
        invalid.max_tokens = 0;
        assert!(into_requests(principal(), invalid, None).is_err());
    }
}
