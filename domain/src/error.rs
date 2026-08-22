use std::fmt;

/// Stable machine-readable code for public SynapseFlow failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    InvalidReference,
    DisallowedSource,
    ManifestInvalid,
    ManifestUnavailable,
    ManifestUnsupported,
    PublisherUntrusted,
    SignatureInvalid,
    ArtifactUnavailable,
    ArtifactIntegrity,
    CacheFailure,
    BackendUnavailable,
    BackendIncompatible,
    TokenizerFailure,
    GenerationPolicyInvalid,
    DeadlineExceeded,
    GenerationFailed,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::InvalidReference => "SYN-MODEL-001",
            Self::DisallowedSource => "SYN-MODEL-002",
            Self::ManifestInvalid => "SYN-MODEL-003",
            Self::ManifestUnavailable => "SYN-MODEL-004",
            Self::ManifestUnsupported => "SYN-MODEL-005",
            Self::PublisherUntrusted => "SYN-MODEL-006",
            Self::SignatureInvalid => "SYN-MODEL-007",
            Self::ArtifactUnavailable => "SYN-MODEL-008",
            Self::ArtifactIntegrity => "SYN-MODEL-009",
            Self::CacheFailure => "SYN-MODEL-010",
            Self::BackendUnavailable => "SYN-INFER-001",
            Self::BackendIncompatible => "SYN-INFER-002",
            Self::TokenizerFailure => "SYN-INFER-003",
            Self::GenerationPolicyInvalid => "SYN-INFER-004",
            Self::DeadlineExceeded => "SYN-INFER-005",
            Self::GenerationFailed => "SYN-INFER-006",
        };
        formatter.write_str(value)
    }
}

/// Typed domain failures. Diagnostics must remain free of sensitive payloads.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("invalid manifest reference")]
    InvalidReference,
    #[error("model source is not allowed")]
    DisallowedSource,
    #[error("model manifest is invalid")]
    ManifestInvalid,
    #[error("model manifest is unavailable")]
    ManifestUnavailable,
    #[error("model manifest is unsupported")]
    ManifestUnsupported,
    #[error("publisher is not trusted")]
    PublisherUntrusted,
    #[error("model manifest signature is invalid")]
    SignatureInvalid,
    #[error("model artifact is unavailable")]
    ArtifactUnavailable,
    #[error("model artifact failed integrity verification")]
    ArtifactIntegrity,
    #[error("model cache operation failed")]
    CacheFailure,
    #[error("model backend is unavailable")]
    BackendUnavailable,
    #[error("model backend is incompatible with the verified model")]
    BackendIncompatible,
    #[error("tokenizer operation failed")]
    TokenizerFailure,
    #[error("generation policy is invalid")]
    GenerationPolicyInvalid,
    #[error("generation deadline expired")]
    DeadlineExceeded,
    #[error("generation failed")]
    GenerationFailed,
}

impl DomainError {
    /// Returns the stable public error code.
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidReference => ErrorCode::InvalidReference,
            Self::DisallowedSource => ErrorCode::DisallowedSource,
            Self::ManifestInvalid => ErrorCode::ManifestInvalid,
            Self::ManifestUnavailable => ErrorCode::ManifestUnavailable,
            Self::ManifestUnsupported => ErrorCode::ManifestUnsupported,
            Self::PublisherUntrusted => ErrorCode::PublisherUntrusted,
            Self::SignatureInvalid => ErrorCode::SignatureInvalid,
            Self::ArtifactUnavailable => ErrorCode::ArtifactUnavailable,
            Self::ArtifactIntegrity => ErrorCode::ArtifactIntegrity,
            Self::CacheFailure => ErrorCode::CacheFailure,
            Self::BackendUnavailable => ErrorCode::BackendUnavailable,
            Self::BackendIncompatible => ErrorCode::BackendIncompatible,
            Self::TokenizerFailure => ErrorCode::TokenizerFailure,
            Self::GenerationPolicyInvalid => ErrorCode::GenerationPolicyInvalid,
            Self::DeadlineExceeded => ErrorCode::DeadlineExceeded,
            Self::GenerationFailed => ErrorCode::GenerationFailed,
        }
    }
}

/// Result type for all domain contracts.
pub type DomainResult<T> = std::result::Result<T, DomainError>;
