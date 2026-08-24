use crate::{ArtifactDescriptor, DomainResult, ModelReference, ShardPlan};

use super::{manifest_parser, TokenizerDeclaration, TrustStore};

/// The only manifest schema accepted by the verified-local-inference milestone.
pub const MANIFEST_SCHEMA_VERSION: u16 = 1;
/// Schema version that introduces the first loopback-sharding declaration.
pub const LOOPBACK_SHARDING_MANIFEST_SCHEMA_VERSION: u16 = 2;

/// Maximum accepted serialized manifest size before parsing or allocating.
pub const MAX_MANIFEST_BYTES: usize = 64 * 1024;

/// The sole weight format supported by the verified-local-inference milestone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelFormat {
    Gguf,
}

/// Manifest information the application can use without registry or backend dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelManifest {
    pub reference: ModelReference,
    pub schema_version: u16,
    pub model_id: String,
    pub model_version: String,
    pub format: ModelFormat,
    pub architecture: String,
    pub quantization: String,
    pub tokenizer: TokenizerDeclaration,
    pub artifacts: Vec<ArtifactDescriptor>,
    pub publisher_key_id: String,
    pub license: String,
    pub provenance: String,
    pub execution_plan: Option<ShardPlan>,
    pub runtime_profile: Option<String>,
}

impl ModelManifest {
    /// Parses, bounds-checks, validates, and verifies a versioned signed manifest.
    pub fn parse_and_verify(
        reference: ModelReference,
        document: &[u8],
        trust_store: &TrustStore,
    ) -> DomainResult<Self> {
        manifest_parser::parse_and_verify(reference, document, trust_store)
    }

    pub fn supports_verified_local_inference(&self) -> bool {
        self.schema_version == MANIFEST_SCHEMA_VERSION
            && self.format == ModelFormat::Gguf
            && self.architecture == "llama"
            && self.quantization == "Q5_K_M"
            && self.tokenizer.is_embedded_llama()
            && self.artifacts.len() == 1
            && self.execution_plan.is_none()
    }
}
