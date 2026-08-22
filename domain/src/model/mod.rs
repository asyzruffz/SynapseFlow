//! Immutable model manifest and artifact contracts.

mod artifact;
mod manifest;
mod reference;

pub use artifact::{ArtifactDescriptor, ArtifactId};
pub use manifest::{ModelFormat, ModelManifest};
pub use reference::ModelReference;
