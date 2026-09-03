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
    ExecutionStrategyUnsupported,
    ShardPlanInvalid,
    FrameInvalid,
    SessionStateInvalid,
    RetryExhausted,
    SessionCancelled,
    ReplicaRecoveryFailed,
    ProtocolUnsupported,
    ModelVersionMismatch,
    FrameBoundsExceeded,
    FrameIntegrity,
    FrameDtypeUnsupported,
    FrameSequenceInvalid,
    WorkerUnavailable,
    DuplicateWork,
    PrincipalInvalid,
    ScopeInvalid,
    PublicSessionInvalid,
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
            Self::ExecutionStrategyUnsupported => "SYN-SHARD-001",
            Self::ShardPlanInvalid => "SYN-SHARD-002",
            Self::FrameInvalid => "SYN-SHARD-003",
            Self::SessionStateInvalid => "SYN-SHARD-004",
            Self::RetryExhausted => "SYN-SHARD-005",
            Self::SessionCancelled => "SYN-SHARD-006",
            Self::ReplicaRecoveryFailed => "SYN-SHARD-007",
            Self::ProtocolUnsupported => "SYN-FRAME-001",
            Self::ModelVersionMismatch => "SYN-FRAME-002",
            Self::FrameBoundsExceeded => "SYN-FRAME-003",
            Self::FrameIntegrity => "SYN-FRAME-004",
            Self::FrameDtypeUnsupported => "SYN-FRAME-005",
            Self::FrameSequenceInvalid => "SYN-FRAME-006",
            Self::WorkerUnavailable => "SYN-SHARD-008",
            Self::DuplicateWork => "SYN-SHARD-009",
            Self::PrincipalInvalid => "SYN-NODE-001",
            Self::ScopeInvalid => "SYN-NODE-002",
            Self::PublicSessionInvalid => "SYN-NODE-003",
        };
        formatter.write_str(value)
    }
}

/// Typed domain failures. Diagnostics must remain free of sensitive payloads.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
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
    #[error("execution strategy is unsupported")]
    ExecutionStrategyUnsupported,
    #[error("shard execution plan is invalid")]
    ShardPlanInvalid,
    #[error("activation frame is invalid")]
    FrameInvalid,
    #[error("session state transition is invalid")]
    SessionStateInvalid,
    #[error("retry budget is exhausted")]
    RetryExhausted,
    #[error("session is cancelled")]
    SessionCancelled,
    #[error("replica recovery failed")]
    ReplicaRecoveryFailed,
    #[error("frame protocol version is unsupported")]
    ProtocolUnsupported,
    #[error("frame model version does not match the execution target")]
    ModelVersionMismatch,
    #[error("frame exceeds a configured resource bound")]
    FrameBoundsExceeded,
    #[error("frame payload integrity validation failed")]
    FrameIntegrity,
    #[error("frame tensor dtype is unsupported")]
    FrameDtypeUnsupported,
    #[error("frame sequence is invalid")]
    FrameSequenceInvalid,
    #[error("worker is unavailable")]
    WorkerUnavailable,
    #[error("work with this idempotency key is already active")]
    DuplicateWork,
    #[error("authenticated principal is invalid")]
    PrincipalInvalid,
    #[error("granted scope is invalid")]
    ScopeInvalid,
    #[error("public session identifier is invalid")]
    PublicSessionInvalid,
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
            Self::ExecutionStrategyUnsupported => ErrorCode::ExecutionStrategyUnsupported,
            Self::ShardPlanInvalid => ErrorCode::ShardPlanInvalid,
            Self::FrameInvalid => ErrorCode::FrameInvalid,
            Self::SessionStateInvalid => ErrorCode::SessionStateInvalid,
            Self::RetryExhausted => ErrorCode::RetryExhausted,
            Self::SessionCancelled => ErrorCode::SessionCancelled,
            Self::ReplicaRecoveryFailed => ErrorCode::ReplicaRecoveryFailed,
            Self::ProtocolUnsupported => ErrorCode::ProtocolUnsupported,
            Self::ModelVersionMismatch => ErrorCode::ModelVersionMismatch,
            Self::FrameBoundsExceeded => ErrorCode::FrameBoundsExceeded,
            Self::FrameIntegrity => ErrorCode::FrameIntegrity,
            Self::FrameDtypeUnsupported => ErrorCode::FrameDtypeUnsupported,
            Self::FrameSequenceInvalid => ErrorCode::FrameSequenceInvalid,
            Self::WorkerUnavailable => ErrorCode::WorkerUnavailable,
            Self::DuplicateWork => ErrorCode::DuplicateWork,
            Self::PrincipalInvalid => ErrorCode::PrincipalInvalid,
            Self::ScopeInvalid => ErrorCode::ScopeInvalid,
            Self::PublicSessionInvalid => ErrorCode::PublicSessionInvalid,
        }
    }
}

/// Result type for all domain contracts.
pub type DomainResult<T> = std::result::Result<T, DomainError>;
