//! Stable, framework-independent SynapseFlow contracts.

mod error;
pub mod generation;
pub mod model;

pub use error::{DomainError, DomainResult, ErrorCode};
pub use generation::{GeneratedToken, GenerationOutput, GenerationPolicy, GenerationRequest};
pub use model::{ArtifactDescriptor, ArtifactId, ModelFormat, ModelManifest, ModelReference};
