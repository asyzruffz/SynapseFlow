use synapseflow_domain::{ArtifactId, DomainResult, ModelManifest, ModelReference};

/// Safe cache state exposed without revealing host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheEntryState {
    Cached,
    Missing,
}

/// Inspected state of one artifact declared by a verified manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedArtifactInspection {
    pub artifact_id: ArtifactId,
    pub content_sha256: String,
    pub size_bytes: u64,
    pub state: CacheEntryState,
}

/// Safe model provenance and cache state for application/API inspection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelCacheInspection {
    pub reference: ModelReference,
    pub publisher_key_id: String,
    pub license: String,
    pub provenance: String,
    pub artifacts: Vec<CachedArtifactInspection>,
}

/// Represents verified artifacts leased to a backend without exposing a cache path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedModel {
    pub manifest: ModelManifest,
}

/// Verifies, caches, leases, and releases immutable model artifacts.
pub trait ArtifactStore: Send + Sync {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel>;

    /// Inspects only verified metadata and cache state; it never returns a cache path.
    fn inspect(&self, manifest: &ModelManifest) -> DomainResult<ModelCacheInspection>;
}
