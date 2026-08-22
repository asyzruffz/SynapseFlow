//! Immutable model manifest and artifact contracts.

mod artifact;
mod manifest;
mod manifest_parser;
mod reference;
mod tokenizer;
mod trust;

pub use artifact::{ArtifactDescriptor, ArtifactId};
pub use manifest::{ModelFormat, ModelManifest, MANIFEST_SCHEMA_VERSION, MAX_MANIFEST_BYTES};
pub use reference::ModelReference;
pub use tokenizer::{TokenizerDeclaration, TokenizerKind};
pub use trust::{TrustStore, TrustedPublisher};
