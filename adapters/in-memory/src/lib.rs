//! Deterministic adapter implementations used by unit and integration tests.

mod artifact_store;
mod audit_sink;
mod backend;
mod registry;

pub use artifact_store::InMemoryArtifactStore;
pub use audit_sink::InMemoryAuditSink;
pub use backend::InMemoryModelBackend;
pub use registry::InMemoryModelRegistry;
