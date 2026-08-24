//! Deterministic adapter implementations used by unit and integration tests.

mod artifact_store;
mod audit_sink;
mod backend;
mod peer_directory;
mod registry;
mod shard_execution;
mod transport;

pub use artifact_store::InMemoryArtifactStore;
pub use audit_sink::InMemoryAuditSink;
pub use backend::InMemoryModelBackend;
pub use peer_directory::InMemoryPeerDirectory;
pub use registry::InMemoryModelRegistry;
pub use shard_execution::InMemoryShardExecutionBackend;
pub use transport::InMemoryTransport;
