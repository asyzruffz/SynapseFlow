use synapseflow_domain::{ArtifactId, ModelReference};

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
