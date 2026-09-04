use std::path::PathBuf;

use clap::Args;
use synapseflow_domain::{DomainResult, ModelConfig};

/// CLI parsing for the explicit verified-local runtime configuration.
#[derive(Args)]
pub struct VerifiedLocalRuntimeArgs {
    /// Provisioned, signed manifest document for the selected immutable reference.
    #[arg(long)]
    pub manifest: PathBuf,
    /// Provisioned GGUF source matching the signed manifest artifact declaration.
    #[arg(long)]
    pub artifact: PathBuf,
    /// Directory for the verified content-addressed local cache.
    #[arg(long)]
    pub cache_dir: PathBuf,
    /// Base64url Ed25519 public key for the configured fixture publisher.
    #[arg(long)]
    pub publisher_public_key: String,
}

impl VerifiedLocalRuntimeArgs {
    pub fn into_config(self) -> DomainResult<ModelConfig> {
        let config = ModelConfig {
            manifest_path: self.manifest,
            artifact_path: self.artifact,
            cache_directory: self.cache_dir,
            publisher_public_key: self.publisher_public_key,
        };
        config.validate()?;
        Ok(config)
    }
}
