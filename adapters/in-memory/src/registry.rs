use std::collections::BTreeMap;

use synapseflow_domain::{DomainError, DomainResult, ModelManifest, ModelReference};
use synapseflow_ports::ModelRegistry;

/// A registry populated directly by tests or application composition.
#[derive(Default)]
pub struct InMemoryModelRegistry {
    manifests: BTreeMap<ModelReference, ModelManifest>,
}

impl InMemoryModelRegistry {
    pub fn with_manifest(manifest: ModelManifest) -> Self {
        let mut manifests = BTreeMap::new();
        manifests.insert(manifest.reference.clone(), manifest);
        Self { manifests }
    }
}

impl ModelRegistry for InMemoryModelRegistry {
    fn resolve(&self, reference: &ModelReference) -> DomainResult<ModelManifest> {
        self.manifests
            .get(reference)
            .cloned()
            .ok_or(DomainError::ManifestUnavailable)
    }
}
