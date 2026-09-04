use axum::http::{header, HeaderMap};
use synapseflow_domain::{AuthenticatedPrincipal, DomainError};
use synapseflow_ports::BearerCredential;

use super::{state::ApiState, ApiError};

/// Verifies a bearer credential off the async request executor.
pub(super) async fn authenticate(
    state: &ApiState,
    headers: &HeaderMap,
) -> Result<AuthenticatedPrincipal, ApiError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .ok_or(DomainError::AuthenticationInvalid)
        .and_then(|value| {
            value
                .to_str()
                .map_err(|_| DomainError::AuthenticationInvalid)
        })
        .map_err(ApiError::from_domain)?;
    let (scheme, token) = value
        .split_once(' ')
        .ok_or(DomainError::AuthenticationInvalid)
        .map_err(ApiError::from_domain)?;
    if !scheme.eq_ignore_ascii_case("bearer")
        || token.is_empty()
        || token.contains(char::is_whitespace)
    {
        return Err(ApiError::from_domain(DomainError::AuthenticationInvalid));
    }
    let verifier = state.dependencies.identity.clone();
    let token = token.to_owned();
    tokio::task::spawn_blocking(move || verifier.verify(BearerCredential::new(&token)?))
        .await
        .map_err(|_| ApiError::from_domain(DomainError::AuthenticationInvalid))?
        .map_err(ApiError::from_domain)
}
