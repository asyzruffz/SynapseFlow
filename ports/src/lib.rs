//! Object-safe boundaries between SynapseFlow application logic and adapters.

mod artifact_store;
mod audit;
mod backend;
mod clock;
mod peer_directory;
mod registry;
mod shard_execution;
mod transport;
mod worker;

pub use artifact_store::{
    ArtifactStore, CacheEntryState, CachedArtifactInspection, ModelCacheInspection, VerifiedModel,
};
pub use audit::{AuditEvent, AuditSink, ShardSessionOutcome};
pub use backend::ModelBackend;
pub use clock::Clock;
pub use peer_directory::PeerDirectory;
pub use registry::ModelRegistry;
pub use shard_execution::{
    ShardExecutionBackend, ShardExecutionOutput, ShardExecutionRequest, ShardExecutionRequirements,
};
pub use transport::{ReceivedFrame, Transport, TransportReceipt};
pub use worker::{ShardAvailability, WorkerCapability, WorkerHealth, WorkerId};
