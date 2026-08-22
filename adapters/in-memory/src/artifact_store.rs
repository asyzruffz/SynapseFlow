use synapseflow_domain::{DomainError, DomainResult, ModelManifest};
use synapseflow_ports::{
    ArtifactStore, CacheEntryState, CachedArtifactInspection, ModelCacheInspection, VerifiedModel,
};

/// An artifact store that treats an already registered manifest as verified.
pub struct InMemoryArtifactStore;

impl ArtifactStore for InMemoryArtifactStore {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel> {
        if !manifest.supports_verified_local_inference() {
            return Err(DomainError::ManifestUnsupported);
        }
        Ok(VerifiedModel::without_cached_artifacts(manifest.clone()))
    }

    fn inspect(&self, manifest: &ModelManifest) -> DomainResult<ModelCacheInspection> {
        if !manifest.supports_verified_local_inference() {
            return Err(DomainError::ManifestUnsupported);
        }
        Ok(ModelCacheInspection {
            reference: manifest.reference.clone(),
            publisher_key_id: manifest.publisher_key_id.clone(),
            license: manifest.license.clone(),
            provenance: manifest.provenance.clone(),
            artifacts: manifest
                .artifacts
                .iter()
                .map(|artifact| CachedArtifactInspection {
                    artifact_id: artifact.id.clone(),
                    content_sha256: artifact.content_sha256.clone(),
                    size_bytes: artifact.size_bytes,
                    state: CacheEntryState::Cached,
                })
                .collect(),
        })
    }
}
