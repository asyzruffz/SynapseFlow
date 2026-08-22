use std::sync::Arc;

use synapseflow_domain::{DomainResult, ModelReference};
use synapseflow_ports::{
    ArtifactStore, AuditEvent, AuditSink, ModelCacheInspection, ModelRegistry,
};

/// Resolves, verifies, caches, and safely inspects one immutable local model.
pub struct ModelAcquisitionService {
    registry: Arc<dyn ModelRegistry>,
    artifacts: Arc<dyn ArtifactStore>,
    audit: Arc<dyn AuditSink>,
}

impl ModelAcquisitionService {
    pub fn new(
        registry: Arc<dyn ModelRegistry>,
        artifacts: Arc<dyn ArtifactStore>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            registry,
            artifacts,
            audit,
        }
    }

    /// Acquires a verified model then returns its safe provenance and cache state.
    pub fn acquire_and_inspect(
        &self,
        reference: ModelReference,
    ) -> DomainResult<ModelCacheInspection> {
        let result = self.acquire_and_inspect_inner(&reference);
        if result.is_err() {
            self.audit
                .record(AuditEvent::ModelAcquisitionFailed { model: reference })?;
        }
        result
    }

    fn acquire_and_inspect_inner(
        &self,
        reference: &ModelReference,
    ) -> DomainResult<ModelCacheInspection> {
        let manifest = self.registry.resolve(reference)?;
        self.audit.record(AuditEvent::ManifestVerified {
            model: manifest.reference.clone(),
            publisher_key_id: manifest.publisher_key_id.clone(),
        })?;
        let model = self.artifacts.acquire(&manifest)?;
        let inspection = self.artifacts.inspect(&model.manifest)?;
        self.audit.record(AuditEvent::ArtifactsCached {
            model: manifest.reference,
            artifact_count: inspection.artifacts.len(),
        })?;
        Ok(inspection)
    }
}
