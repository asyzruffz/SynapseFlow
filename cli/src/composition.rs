use std::sync::Arc;

use synapseflow_adapter_in_memory::{
    InMemoryArtifactStore, InMemoryAuditSink, InMemoryModelBackend, InMemoryModelRegistry,
};
use synapseflow_application::GenerationService;

/// Temporary deterministic composition root until verified local adapters land.
pub fn in_memory_generation_service() -> GenerationService {
    GenerationService::new(
        Arc::new(InMemoryModelRegistry::default()),
        Arc::new(InMemoryArtifactStore),
        Arc::new(InMemoryModelBackend::default()),
        Arc::new(InMemoryAuditSink::default()),
    )
}
