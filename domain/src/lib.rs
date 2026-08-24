//! Stable, framework-independent SynapseFlow contracts.

mod error;
pub mod execution;
pub mod generation;
pub mod model;

pub use error::{DomainError, DomainResult, ErrorCode};
pub use execution::{
    DecodedFrame, ExecutionStrategy, FrameCodec, FrameCompression, LayerRange, SafeTraceId,
    ShardId, ShardPlan, ShardSpec, TensorDescriptor, TensorDtype,
};
pub use generation::{GeneratedToken, GenerationOutput, GenerationPolicy, GenerationRequest};
pub use model::{
    ArtifactDescriptor, ArtifactId, ModelFormat, ModelManifest, ModelReference,
    TokenizerDeclaration, TokenizerKind, TrustStore, TrustedPublisher,
    LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION, MANIFEST_SCHEMA_VERSION, MAX_MANIFEST_BYTES,
};
