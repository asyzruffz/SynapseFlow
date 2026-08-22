use synapseflow_domain::{DomainResult, ModelManifest, ModelReference};

/// Acquires a trusted immutable manifest from an allowed source.
pub trait ModelRegistry: Send + Sync {
    fn resolve(&self, reference: &ModelReference) -> DomainResult<ModelManifest>;
}
