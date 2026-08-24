//! Versioned, infrastructure-independent distributed-execution contracts.

mod codec;
mod frame;
mod session;
mod shard;
mod strategy;
mod tensor;

pub use codec::{DecodedFrame, FrameCodec, FrameCompression, SafeTraceId};
pub use frame::{
    CheckpointRef, FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence,
    FrameTarget, InFlightFrameLimit, RemainingDeadline, SessionId, StreamId,
    FRAME_PROTOCOL_VERSION, MAX_IN_FLIGHT_FRAMES,
};
pub use session::{RetryBudget, SessionState};
pub use shard::{LayerRange, ShardId, ShardPlan, ShardSpec};
pub use strategy::ExecutionStrategy;
pub use tensor::{TensorDescriptor, TensorDtype, MAX_TENSOR_BYTES, MAX_TENSOR_RANK};
