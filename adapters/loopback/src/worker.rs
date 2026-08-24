use std::sync::Arc;

use synapseflow_domain::execution::FrameEnvelope;
use synapseflow_domain::{
    DecodedFrame, DomainResult, ErrorCode, FrameCodec, FrameCompression, SafeTraceId,
};
use synapseflow_ports::{Transport, TransportReceipt, WorkerId};

use crate::LoopbackTransport;

/// A codec-validated frame received by a named local worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceivedWorkerFrame {
    pub source: WorkerId,
    pub frame: DecodedFrame,
}

/// Independently addressable worker endpoint for loopback integration tests.
#[derive(Clone)]
pub struct LoopbackWorker {
    id: WorkerId,
    transport: Arc<LoopbackTransport>,
}

impl LoopbackWorker {
    pub(crate) fn new(id: WorkerId, transport: Arc<LoopbackTransport>) -> Self {
        Self { id, transport }
    }

    pub fn id(&self) -> &WorkerId {
        &self.id
    }

    /// Encodes every control or data frame before it crosses the worker boundary.
    pub fn send(
        &self,
        destination: &WorkerId,
        envelope: &FrameEnvelope,
        payload: &[u8],
        trace_id: Option<&SafeTraceId>,
    ) -> DomainResult<()> {
        let bytes = FrameCodec::encode(envelope, payload, FrameCompression::None, trace_id)?;
        self.transport.send(&self.id, destination, bytes)
    }

    /// Forwards a decoded data frame without discarding its validated extensions.
    pub fn send_frame(&self, destination: &WorkerId, frame: &DecodedFrame) -> DomainResult<()> {
        let bytes = FrameCodec::encode_with_extensions(
            &frame.envelope,
            &frame.payload,
            frame.compression,
            frame.trace_id.as_ref(),
            frame.extensions(),
        )?;
        self.transport.send(&self.id, destination, bytes)
    }

    pub fn receive(&self) -> DomainResult<Option<ReceivedWorkerFrame>> {
        self.transport
            .receive(&self.id)?
            .map_or(Ok(None), |received| {
                Ok(Some(ReceivedWorkerFrame {
                    source: received.source,
                    frame: FrameCodec::decode(&received.bytes)?,
                }))
            })
    }

    pub fn acknowledge(
        &self,
        destination: &WorkerId,
        envelope: &FrameEnvelope,
    ) -> DomainResult<()> {
        self.transport.acknowledge(
            &self.id,
            destination,
            &TransportReceipt::from_envelope(envelope),
        )
    }

    pub fn reject(
        &self,
        destination: &WorkerId,
        envelope: &FrameEnvelope,
        reason: ErrorCode,
    ) -> DomainResult<()> {
        self.transport.reject(
            &self.id,
            destination,
            &TransportReceipt::from_envelope(envelope),
            reason,
        )
    }

    pub fn cancel(&self, destination: &WorkerId, envelope: &FrameEnvelope) -> DomainResult<()> {
        self.transport.cancel(
            &self.id,
            destination,
            &TransportReceipt::from_envelope(envelope),
        )
    }

    pub fn shutdown(&self) -> DomainResult<()> {
        self.transport.shutdown(&self.id)
    }
}
