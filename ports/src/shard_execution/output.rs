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
        match self {
            Self::Boundary(frame)
                if frame.envelope.message_type == FrameMessageType::Data
                    && Some(&frame.envelope.target) == request.next_target.as_ref() =>
            {
                Ok(())
            }
            Self::FinalLogits(frame)
                if frame.envelope.message_type == FrameMessageType::Data
                    && frame.envelope.target == request.target
                    && request.next_target.is_none() =>
            {
                Ok(())
            }
            _ => Err(DomainError::FrameInvalid),
        }
    }
}
