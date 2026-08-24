use std::collections::VecDeque;
use std::sync::Mutex;

use synapseflow_domain::execution::{
    FrameEnvelope, FrameExtension, FrameMessageType, FrameProtocolVersion, InFlightFrameLimit,
};
use synapseflow_domain::{DomainError, DomainResult, ErrorCode, FrameCodec, FrameCompression};
use synapseflow_ports::{ReceivedFrame, Transport, TransportReceipt, WorkerId};

use crate::faults::{FaultState, LoopbackFault};

/// Protocol-v1 additive extension tag carrying an ASCII stable error code on a NACK.
const NACK_REASON_EXTENSION_TAG: u8 = 1;

struct QueuedFrame {
    source: WorkerId,
    bytes: Vec<u8>,
    delay_polls: u8,
    timeout: bool,
}

/// Safe transport observation with no prompt, activation, or weight content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoopbackEvent {
    NackSent {
        source: WorkerId,
        destination: WorkerId,
        receipt: TransportReceipt,
        reason: ErrorCode,
    },
}

/// Production loopback adapter: it admits and delivers only codec-validated bytes.
pub struct LoopbackTransport {
    limit: InFlightFrameLimit,
    queues: Mutex<Vec<(WorkerId, VecDeque<QueuedFrame>)>>,
    faults: Mutex<FaultState>,
    events: Mutex<Vec<LoopbackEvent>>,
}

impl LoopbackTransport {
    pub(crate) fn new(limit: InFlightFrameLimit, workers: Vec<WorkerId>) -> DomainResult<Self> {
        if workers.len() < 2 || has_duplicates(&workers) {
            return Err(DomainError::WorkerUnavailable);
        }
        Ok(Self {
            limit,
            queues: Mutex::new(
                workers
                    .into_iter()
                    .map(|worker| (worker, VecDeque::new()))
                    .collect(),
            ),
            faults: Mutex::new(FaultState::default()),
            events: Mutex::new(Vec::new()),
        })
    }

    pub fn inject(&self, fault: LoopbackFault) -> DomainResult<()> {
        self.faults
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .inject(fault)
    }

    pub fn events(&self) -> DomainResult<Vec<LoopbackEvent>> {
        self.events
            .lock()
            .map(|events| events.clone())
            .map_err(|_| DomainError::CacheFailure)
    }

    fn send_control(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        receipt: &TransportReceipt,
        message_type: FrameMessageType,
        reason: Option<ErrorCode>,
    ) -> DomainResult<()> {
        let envelope = FrameEnvelope::new(
            FrameProtocolVersion::current(),
            message_type,
            receipt.session_id.clone(),
            receipt.stream_id,
            receipt.sequence,
            receipt.target.clone(),
            None,
            receipt.remaining_deadline,
        )?;
        let extensions = match reason {
            Some(reason) => vec![FrameExtension::new(
                NACK_REASON_EXTENSION_TAG,
                reason.to_string().into_bytes(),
            )?],
            None => Vec::new(),
        };
        let bytes = FrameCodec::encode_with_extensions(
            &envelope,
            &[],
            FrameCompression::None,
            None,
            &extensions,
        )?;
        self.send(source, destination, bytes)
    }

    fn purge_session(
        &self,
        destination: &WorkerId,
        receipt: &TransportReceipt,
    ) -> DomainResult<()> {
        let mut queues = self.queues.lock().map_err(|_| DomainError::CacheFailure)?;
        let queue = queue_for(&mut queues, destination)?;
        queue.retain(|queued| {
            FrameCodec::decode(&queued.bytes)
                .is_ok_and(|frame| frame.envelope.session_id != receipt.session_id)
        });
        Ok(())
    }
}

impl Transport for LoopbackTransport {
    fn queue_limit(&self) -> InFlightFrameLimit {
        self.limit
    }

    fn is_available(&self, worker: &WorkerId) -> DomainResult<bool> {
        let known = self
            .queues
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .iter()
            .any(|(candidate, _)| candidate == worker);
        let unavailable = self
            .faults
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .is_unavailable(worker);
        Ok(known && !unavailable)
    }

    fn send(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        canonical_frame: Vec<u8>,
    ) -> DomainResult<()> {
        if !self.is_available(source)? || !self.is_available(destination)? {
            return Err(DomainError::WorkerUnavailable);
        }
        let decoded = FrameCodec::decode(&canonical_frame)?;
        let instruction = self
            .faults
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .delivery_for(source, destination, decoded.envelope.message_type)?;
        if instruction.drop {
            return Ok(());
        }
        let mut bytes = canonical_frame;
        if instruction.corrupt {
            let last = bytes.last_mut().ok_or(DomainError::FrameInvalid)?;
            *last ^= 1;
        }
        let mut queues = self.queues.lock().map_err(|_| DomainError::CacheFailure)?;
        let queue = queue_for(&mut queues, destination)?;
        if queue.len() >= usize::from(self.limit.value()) {
            return Err(DomainError::FrameBoundsExceeded);
        }
        queue.push_back(QueuedFrame {
            source: source.clone(),
            bytes,
            delay_polls: instruction.delay_polls,
            timeout: instruction.timeout,
        });
        Ok(())
    }

    fn receive(&self, recipient: &WorkerId) -> DomainResult<Option<ReceivedFrame>> {
        if !self.is_available(recipient)? {
            return Err(DomainError::WorkerUnavailable);
        }
        let mut queues = self.queues.lock().map_err(|_| DomainError::CacheFailure)?;
        let queue = queue_for(&mut queues, recipient)?;
        let Some(front) = queue.front_mut() else {
            return Ok(None);
        };
        if front.delay_polls > 0 {
            front.delay_polls -= 1;
            return Ok(None);
        }
        let queued = queue.pop_front().ok_or(DomainError::FrameInvalid)?;
        if queued.timeout {
            return Err(DomainError::DeadlineExceeded);
        }
        FrameCodec::decode(&queued.bytes)?;
        Ok(Some(ReceivedFrame::new(queued.source, queued.bytes)?))
    }

    fn acknowledge(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        receipt: &TransportReceipt,
    ) -> DomainResult<()> {
        self.send_control(source, destination, receipt, FrameMessageType::Ack, None)
    }

    fn reject(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        receipt: &TransportReceipt,
        reason: ErrorCode,
    ) -> DomainResult<()> {
        self.events
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .push(LoopbackEvent::NackSent {
                source: source.clone(),
                destination: destination.clone(),
                receipt: receipt.clone(),
                reason,
            });
        self.send_control(
            source,
            destination,
            receipt,
            FrameMessageType::Nack,
            Some(reason),
        )
    }

    fn cancel(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        receipt: &TransportReceipt,
    ) -> DomainResult<()> {
        self.purge_session(destination, receipt)?;
        self.send_control(source, destination, receipt, FrameMessageType::Cancel, None)
    }

    fn shutdown(&self, worker: &WorkerId) -> DomainResult<()> {
        if !self
            .queues
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .iter()
            .any(|(candidate, _)| candidate == worker)
        {
            return Err(DomainError::WorkerUnavailable);
        }
        self.inject(LoopbackFault::Unavailable {
            worker: worker.clone(),
            enabled: true,
        })?;
        let mut queues = self.queues.lock().map_err(|_| DomainError::CacheFailure)?;
        queue_for(&mut queues, worker)?.clear();
        Ok(())
    }
}

fn queue_for<'a>(
    queues: &'a mut [(WorkerId, VecDeque<QueuedFrame>)],
    worker: &WorkerId,
) -> DomainResult<&'a mut VecDeque<QueuedFrame>> {
    queues
        .iter_mut()
        .find(|(candidate, _)| candidate == worker)
        .map(|(_, queue)| queue)
        .ok_or(DomainError::WorkerUnavailable)
}

fn has_duplicates(workers: &[WorkerId]) -> bool {
    workers
        .iter()
        .enumerate()
        .any(|(index, worker)| workers[index + 1..].contains(worker))
}
