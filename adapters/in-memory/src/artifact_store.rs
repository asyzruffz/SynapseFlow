use synapseflow_domain::{DomainError, DomainResult, ModelManifest};
use synapseflow_ports::{ArtifactStore, VerifiedModel};

/// An artifact store that treats an already registered manifest as verified.
pub struct InMemoryArtifactStore;

impl ArtifactStore for InMemoryArtifactStore {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel> {
        if !manifest.supports_verified_local_inference() {
            return Err(DomainError::ManifestUnsupported);
        }
        Ok(VerifiedModel {
            manifest: manifest.clone(),
        })
    }
}
