use crate::{ArtifactDescriptor, ModelReference};

/// The sole weight format supported by the verified-local-inference milestone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelFormat {
    Gguf,
}

/// Manifest information the application can use without registry or backend dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelManifest {
    pub reference: ModelReference,
    pub model_id: String,
    pub model_version: String,
    pub format: ModelFormat,
    pub architecture: String,
    pub artifacts: Vec<ArtifactDescriptor>,
}

impl ModelManifest {
    pub fn supports_verified_local_inference(&self) -> bool {
        self.format == ModelFormat::Gguf
            && self.architecture == "llama"
            && self.artifacts.len() == 1
    }
}
