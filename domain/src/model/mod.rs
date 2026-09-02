//! Immutable model manifest and artifact contracts.

mod artifact;
mod config;
mod manifest;
mod manifest_parser;
mod reference;
mod tokenizer;
mod trust;

pub use artifact::{ArtifactDescriptor, ArtifactId};
pub use config::ModelConfig;
pub use manifest::{
    ModelFormat, ModelManifest, LOOM_RUNTIME_PROFILE, LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION,
    MANIFEST_SCHEMA_VERSION, MAX_MANIFEST_BYTES,
};
pub use reference::ModelReference;
pub use tokenizer::{TokenizerDeclaration, TokenizerKind};
pub use trust::{TrustStore, TrustedPublisher};
