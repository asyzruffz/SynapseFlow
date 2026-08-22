use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use synapseflow_domain::DomainError;

use super::models::ErrorResponse;

/// Sanitized transport error retaining the domain's stable public code.
pub(super) struct ApiError {
    status: StatusCode,
    code: String,
    message: &'static str,
}

impl ApiError {
    pub(super) fn invalid_request() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "SYN-API-001".to_owned(),
            message: "invalid API request",
        }
    }

    pub(super) fn execution_join_failed() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "SYN-API-002".to_owned(),
            message: "local generation task failed",
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        let status = match error {
            DomainError::InvalidReference | DomainError::GenerationPolicyInvalid => {
                StatusCode::BAD_REQUEST
            }
            DomainError::ManifestUnavailable | DomainError::ArtifactUnavailable => {
                StatusCode::NOT_FOUND
            }
            DomainError::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
            DomainError::BackendUnavailable | DomainError::CacheFailure => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        };
        Self {
            status,
            code: error.code().to_string(),
            message: safe_message(&error),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                code: self.code,
                message: self.message,
            }),
        )
            .into_response()
    }
}

fn safe_message(error: &DomainError) -> &'static str {
    match error {
        DomainError::InvalidReference => "invalid manifest reference",
        DomainError::DisallowedSource => "model source is not allowed",
        DomainError::ManifestInvalid => "model manifest is invalid",
        DomainError::ManifestUnavailable => "model manifest is unavailable",
        DomainError::ManifestUnsupported => "model manifest is unsupported",
        DomainError::PublisherUntrusted => "publisher is not trusted",
        DomainError::SignatureInvalid => "model manifest signature is invalid",
        DomainError::ArtifactUnavailable => "model artifact is unavailable",
        DomainError::ArtifactIntegrity => "model artifact failed integrity verification",
        DomainError::CacheFailure => "model cache operation failed",
        DomainError::BackendUnavailable => "model backend is unavailable",
        DomainError::BackendIncompatible => "model backend is incompatible",
        DomainError::TokenizerFailure => "tokenizer operation failed",
        DomainError::GenerationPolicyInvalid => "generation policy is invalid",
        DomainError::DeadlineExceeded => "generation deadline expired",
        DomainError::GenerationFailed => "generation failed",
    }
}
