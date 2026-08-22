use std::path::PathBuf;

use synapseflow_domain::{DomainError, DomainResult};

/// Explicit, local-only inputs needed to compose the verified runtime.
#[derive(Clone, Debug)]
pub struct VerifiedLocalRuntimeConfig {
    pub manifest_path: PathBuf,
    pub artifact_path: PathBuf,
    pub cache_directory: PathBuf,
    pub publisher_public_key: String,
}

impl VerifiedLocalRuntimeConfig {
    pub fn validate(&self) -> DomainResult<()> {
        if self.publisher_public_key.is_empty() {
            return Err(DomainError::PublisherUntrusted);
        }
        Ok(())
    }
}

/// CLI-only parsing for explicit local runtime configuration.
#[cfg(feature = "cli")]
#[derive(clap::Args)]
pub struct VerifiedLocalRuntimeArgs {
    /// Provisioned, signed manifest document for the selected immutable reference.
    #[arg(long)]
    manifest: PathBuf,
    /// Provisioned GGUF source matching the signed manifest artifact declaration.
    #[arg(long)]
    artifact: PathBuf,
    /// Directory for the verified content-addressed local cache.
    #[arg(long)]
    cache_dir: PathBuf,
    /// Base64url Ed25519 public key for the configured fixture publisher.
    #[arg(long)]
    publisher_public_key: String,
}

#[cfg(feature = "cli")]
impl VerifiedLocalRuntimeArgs {
    pub fn into_config(self) -> DomainResult<VerifiedLocalRuntimeConfig> {
        let config = VerifiedLocalRuntimeConfig {
            manifest_path: self.manifest,
            artifact_path: self.artifact,
            cache_directory: self.cache_dir,
            publisher_public_key: self.publisher_public_key,
        };
        config.validate()?;
        Ok(config)
    }
}
