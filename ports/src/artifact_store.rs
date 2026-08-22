use synapseflow_domain::{DomainResult, ModelManifest};

/// Represents verified artifacts leased to a backend without exposing a cache path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedModel {
    pub manifest: ModelManifest,
}

/// Verifies, caches, leases, and releases immutable model artifacts.
pub trait ArtifactStore: Send + Sync {
    fn acquire(&self, manifest: &ModelManifest) -> DomainResult<VerifiedModel>;
}
