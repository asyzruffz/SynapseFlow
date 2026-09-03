//! Stable, framework-independent SynapseFlow contracts.

mod error;
pub mod execution;
pub mod generation;
pub mod model;
pub mod node;

pub use error::{DomainError, DomainResult, ErrorCode};
pub use execution::{
    DecodedFrame, ExecutionStrategy, FrameCodec, FrameCompression, FrameExtension, LayerRange,
    SafeTraceId, ShardId, ShardPlan, ShardSpec, TensorDescriptor, TensorDtype,
};
pub use generation::{
    GeneratedToken, GenerationEvent, GenerationOutput, GenerationPolicy, GenerationRequest,
    GenerationTerminal,
};
pub use model::{
    ArtifactDescriptor, ArtifactId, ModelConfig, ModelFormat, ModelManifest, ModelReference,
    TokenizerDeclaration, TokenizerKind, TrustStore, TrustedPublisher, LOOM_RUNTIME_PROFILE,
    LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION, MANIFEST_SCHEMA_VERSION, MAX_MANIFEST_BYTES,
};
pub use node::{
    AdmissionDecision, AdmissionRejection, AuthenticatedPrincipal, AuthorizationDecision,
    CancellationResult, GrantedScope, GrantedScopes, IdempotencyKey, PrincipalPseudonym,
    PublicSessionId, PublicSessionState,
};
