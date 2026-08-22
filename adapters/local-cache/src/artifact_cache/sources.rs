use std::{collections::BTreeMap, path::PathBuf};

use synapseflow_domain::{ArtifactDescriptor, DomainError, DomainResult};

/// Explicit mappings from a manifest's approved URI to a locally provisioned source file.
#[derive(Default)]
pub(super) struct ProvisionedArtifactSources {
    paths: BTreeMap<String, PathBuf>,
}

impl ProvisionedArtifactSources {
    pub(super) fn insert(&mut self, uri: String, source: PathBuf) -> DomainResult<()> {
        if !uri.starts_with("https://") || uri.contains(char::is_whitespace) {
            return Err(DomainError::DisallowedSource);
        }
        if self.paths.insert(uri, source).is_some() {
            return Err(DomainError::DisallowedSource);
        }
        Ok(())
    }

    pub(super) fn get(&self, artifact: &ArtifactDescriptor) -> DomainResult<&PathBuf> {
        self.paths
            .get(&artifact.uri)
            .ok_or(DomainError::DisallowedSource)
    }
}
