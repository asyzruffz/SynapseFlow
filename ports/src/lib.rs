//! Object-safe boundaries between SynapseFlow application logic and adapters.

mod artifact_store;
mod audit;
mod backend;
mod clock;
mod peer_directory;
mod registry;
mod transport;

pub use artifact_store::{
    ArtifactStore, CacheEntryState, CachedArtifactInspection, ModelCacheInspection, VerifiedModel,
};
pub use audit::{AuditEvent, AuditSink};
pub use backend::ModelBackend;
pub use clock::Clock;
pub use peer_directory::PeerDirectory;
pub use registry::ModelRegistry;
pub use transport::Transport;
