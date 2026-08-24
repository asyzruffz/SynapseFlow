use synapseflow_domain::execution::{
    FrameEnvelope, FrameSequence, FrameTarget, InFlightFrameLimit, RemainingDeadline, SessionId,
    StreamId,
};
use synapseflow_domain::{DomainError, DomainResult, ErrorCode};

use crate::WorkerId;

/// Frame identity used for idempotent acknowledgement, rejection, and cancellation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportReceipt {
    pub session_id: SessionId,
    pub stream_id: StreamId,
    pub sequence: FrameSequence,
    pub target: FrameTarget,
    pub remaining_deadline: RemainingDeadline,
}

impl TransportReceipt {
    pub fn from_envelope(envelope: &FrameEnvelope) -> Self {
        Self {
            session_id: envelope.session_id.clone(),
            stream_id: envelope.stream_id,
            sequence: envelope.sequence,
            target: envelope.target.clone(),
            remaining_deadline: envelope.remaining_deadline(),
        }
    }
}

/// Canonical protocol bytes received from a named worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceivedFrame {
    pub source: WorkerId,
    pub bytes: Vec<u8>,
}

impl ReceivedFrame {
    pub fn new(source: WorkerId, bytes: Vec<u8>) -> DomainResult<Self> {
        if bytes.is_empty() {
            return Err(DomainError::FrameInvalid);
        }
        Ok(Self { source, bytes })
    }
}

/// Framework-independent transport operations over canonical protocol bytes.
///
/// Adapters must encode before `send` and decode after `receive`; this port does
/// not allow Rust frame objects to bypass SynapseFlow's protocol codec.
pub trait Transport: Send + Sync {
    fn queue_limit(&self) -> InFlightFrameLimit;

    fn is_available(&self, worker: &WorkerId) -> DomainResult<bool>;

    fn send(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        canonical_frame: Vec<u8>,
    ) -> DomainResult<()>;

    fn receive(&self, recipient: &WorkerId) -> DomainResult<Option<ReceivedFrame>>;

    fn acknowledge(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        receipt: &TransportReceipt,
    ) -> DomainResult<()>;

    fn reject(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        receipt: &TransportReceipt,
        reason: ErrorCode,
    ) -> DomainResult<()>;

    fn cancel(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        receipt: &TransportReceipt,
    ) -> DomainResult<()>;

    fn shutdown(&self, worker: &WorkerId) -> DomainResult<()>;
}

#[cfg(test)]
mod tests {
    use super::ReceivedFrame;
    use crate::WorkerId;
    use synapseflow_domain::DomainError;

    #[test]
    fn received_frames_reject_empty_protocol_bytes() {
        let worker = WorkerId::new("loopback-a".to_owned()).expect("worker identifier is valid");

        assert!(matches!(
            ReceivedFrame::new(worker, Vec::new()),
            Err(DomainError::FrameInvalid)
        ));
    }
}
