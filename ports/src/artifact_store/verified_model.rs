use std::path::{Path, PathBuf};

use synapseflow_domain::{DomainError, DomainResult, ModelManifest};

/// Represents verified artifacts leased to a backend without exposing a cache path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedModel {
    pub manifest: ModelManifest,
    artifact_paths: Vec<PathBuf>,
}

impl VerifiedModel {
    /// Creates a verified model without local artifacts, suitable for non-runtime test adapters.
    pub fn without_cached_artifacts(manifest: ModelManifest) -> Self {
        Self {
            manifest,
            artifact_paths: Vec::new(),
        }
    }

    /// Associates verified local cache objects with the manifest in declaration order.
    pub fn with_cached_artifacts(
        manifest: ModelManifest,
        artifact_paths: Vec<PathBuf>,
    ) -> DomainResult<Self> {
        if manifest.artifacts.len() != artifact_paths.len() {
            return Err(DomainError::CacheFailure);
        }

        Ok(Self {
            manifest,
            artifact_paths,
        })
    }

    /// Returns the cached GGUF path only to a model-runtime adapter.
    pub fn primary_artifact_path(&self) -> DomainResult<&Path> {
        self.artifact_paths
            .first()
            .map(PathBuf::as_path)
            .ok_or(DomainError::ArtifactUnavailable)
    }
}
