use std::path::PathBuf;

use synapseflow_domain::{DomainError, DomainResult};

/// Explicit, local-only inputs needed to compose the verified runtime.
#[derive(Clone, Debug)]
pub(crate) struct VerifiedLocalRuntimeConfig {
    pub(crate) manifest_path: PathBuf,
    pub(crate) artifact_path: PathBuf,
    pub(crate) cache_directory: PathBuf,
    pub(crate) publisher_public_key: String,
}

impl VerifiedLocalRuntimeConfig {
    pub(crate) fn validate(&self) -> DomainResult<()> {
        if self.publisher_public_key.is_empty() {
            return Err(DomainError::PublisherUntrusted);
        }
        Ok(())
    }
}
