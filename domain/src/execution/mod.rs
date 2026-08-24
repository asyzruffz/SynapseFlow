//! Versioned, infrastructure-independent distributed-execution contracts.

mod frame;
mod session;
mod shard;
mod strategy;
mod tensor;

pub use frame::{
    CheckpointRef, FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameTarget, SessionId,
    StreamId, FRAME_PROTOCOL_VERSION,
};
pub use session::{RetryBudget, SessionState};
pub use shard::{LayerRange, ShardId, ShardPlan, ShardSpec};
pub use strategy::ExecutionStrategy;
pub use tensor::{TensorDescriptor, TensorDtype, MAX_TENSOR_BYTES, MAX_TENSOR_RANK};
