use std::time::Duration;

use crate::{DomainError, DomainResult, ModelReference};

use super::{ShardId, TensorDescriptor};

/// The first version of SynapseFlow's activation-frame envelope contract.
pub const FRAME_PROTOCOL_VERSION: u16 = 1;

/// Protocol version declared by every frame envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameProtocolVersion(u16);

impl FrameProtocolVersion {
    pub fn new(value: u16) -> DomainResult<Self> {
        if value == 0 {
            return Err(DomainError::FrameInvalid);
        }
        Ok(Self(value))
    }

    pub const fn current() -> Self {
        Self(FRAME_PROTOCOL_VERSION)
    }

    pub const fn value(&self) -> u16 {
        self.0
    }

    pub const fn is_supported(&self) -> bool {
        self.0 == FRAME_PROTOCOL_VERSION
    }
}

/// Opaque, bounded session correlation identifier.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(value: String) -> DomainResult<Self> {
        let valid = (16..=128).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
        if !valid {
            return Err(DomainError::FrameInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Monotonic stream identifier assigned within one session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamId(u64);

impl StreamId {
    pub fn new(value: u64) -> DomainResult<Self> {
        if value == 0 {
            return Err(DomainError::FrameInvalid);
        }
        Ok(Self(value))
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

/// Control or payload intent of one frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameMessageType {
    Data,
    Ack,
    Nack,
    Cancel,
    Heartbeat,
    Error,
}

/// Immutable execution target of a frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameTarget {
    pub model: ModelReference,
    pub shard: ShardId,
}

/// Reference to replayable session-owned checkpoint data, never native KV bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointRef {
    pub session_id: SessionId,
    pub stream_id: StreamId,
    pub sequence: u64,
}

/// Validated frame metadata before bytes are decoded or dispatched.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameEnvelope {
    pub protocol_version: FrameProtocolVersion,
    pub message_type: FrameMessageType,
    pub session_id: SessionId,
    pub stream_id: StreamId,
    pub sequence: u64,
    pub target: FrameTarget,
    pub tensor: Option<TensorDescriptor>,
    remaining_deadline: Duration,
}

impl FrameEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        protocol_version: FrameProtocolVersion,
        message_type: FrameMessageType,
        session_id: SessionId,
        stream_id: StreamId,
        sequence: u64,
        target: FrameTarget,
        tensor: Option<TensorDescriptor>,
        remaining_deadline: Duration,
    ) -> DomainResult<Self> {
        if remaining_deadline.is_zero()
            || matches!(message_type, FrameMessageType::Data) != tensor.is_some()
        {
            return Err(DomainError::FrameInvalid);
        }
        Ok(Self {
            protocol_version,
            message_type,
            session_id,
            stream_id,
            sequence,
            target,
            tensor,
            remaining_deadline,
        })
    }

    pub const fn remaining_deadline(&self) -> Duration {
        self.remaining_deadline
    }

    pub fn checkpoint_ref(&self) -> CheckpointRef {
        CheckpointRef {
            session_id: self.session_id.clone(),
            stream_id: self.stream_id,
            sequence: self.sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameTarget, SessionId, StreamId,
        FRAME_PROTOCOL_VERSION,
    };
    use crate::{DomainError, ModelReference, ShardId, TensorDescriptor, TensorDtype};

    fn target() -> FrameTarget {
        FrameTarget {
            model: ModelReference::parse(format!(
                "registry://fixtures/tinyllama@sha256:{}",
                "a".repeat(64)
            ))
            .expect("test model reference should be valid"),
            shard: ShardId::new("first".to_owned()).expect("test shard should be valid"),
        }
    }

    fn session_id() -> SessionId {
        SessionId::new("session-00000001".to_owned()).expect("test session identifier is valid")
    }

    #[test]
    fn accepts_a_bounded_data_frame_and_exposes_its_checkpoint_reference() {
        let frame = FrameEnvelope::new(
            FrameProtocolVersion::current(),
            FrameMessageType::Data,
            session_id(),
            StreamId::new(1).expect("test stream identifier is valid"),
            7,
            target(),
            Some(
                TensorDescriptor::new(TensorDtype::F32, vec![1, 2_048])
                    .expect("test tensor is valid"),
            ),
            Duration::from_millis(500),
        )
        .expect("data frame should be valid");

        assert_eq!(frame.checkpoint_ref().sequence, 7);
        assert!(frame.protocol_version.is_supported());
        assert_eq!(frame.protocol_version.value(), FRAME_PROTOCOL_VERSION);
    }

    #[test]
    fn rejects_payload_shape_mismatches_and_zero_deadlines() {
        assert!(matches!(
            FrameEnvelope::new(
                FrameProtocolVersion::current(),
                FrameMessageType::Data,
                session_id(),
                StreamId::new(1).expect("test stream identifier is valid"),
                0,
                target(),
                None,
                Duration::from_millis(1),
            ),
            Err(DomainError::FrameInvalid)
        ));
        assert!(matches!(
            FrameEnvelope::new(
                FrameProtocolVersion::current(),
                FrameMessageType::Ack,
                session_id(),
                StreamId::new(1).expect("test stream identifier is valid"),
                0,
                target(),
                None,
                Duration::ZERO,
            ),
            Err(DomainError::FrameInvalid)
        ));
    }
}
