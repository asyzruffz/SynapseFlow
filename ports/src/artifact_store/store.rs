use synapseflow_domain::{DomainResult, ModelManifest};

use super::{ModelCacheInspection, VerifiedModel};

/// Verifies, caches, leases, and releases immutable model artifacts.
pub trait ArtifactStore: Send + Sync {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel>;

    /// Inspects only verified metadata and cache state; it never returns a cache path.
    fn inspect(&self, manifest: &ModelManifest) -> DomainResult<ModelCacheInspection>;
}
