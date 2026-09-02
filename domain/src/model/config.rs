use std::path::PathBuf;

use crate::{DomainError, DomainResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelConfig {
    pub manifest_path: PathBuf,
    pub artifact_path: PathBuf,
    pub cache_directory: PathBuf,
    pub publisher_public_key: String,
}

impl ModelConfig {
    /// Rejects configuration that is invalid before shell composition begins.
    pub fn validate(&self) -> DomainResult<()> {
        if self.publisher_public_key.is_empty() {
            return Err(DomainError::PublisherUntrusted);
        }
        Ok(())
    }
}
