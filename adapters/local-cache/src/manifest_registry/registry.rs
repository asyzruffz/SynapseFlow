use std::collections::BTreeMap;

use synapseflow_domain::{DomainError, DomainResult, ModelManifest, ModelReference, TrustStore};
use synapseflow_ports::ModelRegistry;

/// A registry that accepts only pre-provisioned documents keyed by immutable references.
pub struct ProvisionedManifestRegistry {
    documents: BTreeMap<ModelReference, Vec<u8>>,
    trust_store: TrustStore,
}

impl ProvisionedManifestRegistry {
    pub fn new(
        trust_store: TrustStore,
        documents: impl IntoIterator<Item = (ModelReference, Vec<u8>)>,
    ) -> DomainResult<Self> {
        let mut configured = BTreeMap::new();
        for (reference, document) in documents {
            if configured.insert(reference, document).is_some() {
                return Err(DomainError::DisallowedSource);
            }
        }
        Ok(Self {
            documents: configured,
            trust_store,
        })
    }
}

impl ModelRegistry for ProvisionedManifestRegistry {
    fn resolve(&self, reference: &ModelReference) -> DomainResult<ModelManifest> {
        let document = self
            .documents
            .get(reference)
            .ok_or(DomainError::ManifestUnavailable)?;
        ModelManifest::parse_and_verify(reference.clone(), document, &self.trust_store)
    }
}
