use synapseflow_domain::execution::FrameMessageType;
use synapseflow_domain::{DecodedFrame, DomainError, DomainResult};

use super::ShardExecutionRequest;

/// The next validated activation boundary or the terminal logits frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShardExecutionOutput {
    Boundary(DecodedFrame),
    FinalLogits(DecodedFrame),
}

impl ShardExecutionOutput {
    pub fn validate_for(&self, request: &ShardExecutionRequest) -> DomainResult<()> {
        let frame = match self {
            Self::Boundary(frame) | Self::FinalLogits(frame) => frame,
        };
        if frame.envelope.message_type != FrameMessageType::Data
            || frame.envelope.target != request.target
        {
            return Err(DomainError::FrameInvalid);
        }
        Ok(())
    }
}
