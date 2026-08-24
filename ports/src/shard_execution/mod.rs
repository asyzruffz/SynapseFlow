//! Strategy-neutral contracts for execution of one verified model shard.

mod backend;
mod output;
mod request;
mod requirements;

pub use backend::ShardExecutionBackend;
pub use output::ShardExecutionOutput;
pub use request::ShardExecutionRequest;
pub use requirements::ShardExecutionRequirements;
