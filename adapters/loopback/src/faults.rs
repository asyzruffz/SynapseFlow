use synapseflow_domain::execution::FrameMessageType;
use synapseflow_domain::{DomainError, DomainResult};
use synapseflow_ports::WorkerId;

/// Deterministic fault controls for loopback transport and worker tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoopbackFault {
    DelayNext { destination: WorkerId, polls: u8 },
    TimeoutNext { destination: WorkerId },
    DropNextAck { destination: WorkerId },
    CorruptNextFrame { destination: WorkerId },
    Unavailable { worker: WorkerId, enabled: bool },
    FailNextSend { source: WorkerId },
}

pub(crate) struct DeliveryInstruction {
    pub(crate) delay_polls: u8,
    pub(crate) timeout: bool,
    pub(crate) drop: bool,
    pub(crate) corrupt: bool,
}

#[derive(Default)]
pub(crate) struct FaultState {
    delayed: Vec<(WorkerId, u8)>,
    timeouts: Vec<WorkerId>,
    dropped_acks: Vec<WorkerId>,
    corruptions: Vec<WorkerId>,
    unavailable: Vec<WorkerId>,
    failed_sends: Vec<WorkerId>,
}

impl FaultState {
    pub(crate) fn inject(&mut self, fault: LoopbackFault) -> DomainResult<()> {
        match fault {
            LoopbackFault::DelayNext { destination, polls } => {
                if polls == 0 {
                    return Err(DomainError::FrameInvalid);
                }
                self.delayed.push((destination, polls));
            }
            LoopbackFault::TimeoutNext { destination } => self.timeouts.push(destination),
            LoopbackFault::DropNextAck { destination } => self.dropped_acks.push(destination),
            LoopbackFault::CorruptNextFrame { destination } => self.corruptions.push(destination),
            LoopbackFault::Unavailable { worker, enabled } => {
                if enabled {
                    add_once(&mut self.unavailable, worker);
                } else {
                    remove_once(&mut self.unavailable, &worker);
                }
            }
            LoopbackFault::FailNextSend { source } => self.failed_sends.push(source),
        }
        Ok(())
    }

    pub(crate) fn is_unavailable(&self, worker: &WorkerId) -> bool {
        self.unavailable.contains(worker)
    }

    pub(crate) fn delivery_for(
        &mut self,
        source: &WorkerId,
        destination: &WorkerId,
        message_type: FrameMessageType,
    ) -> DomainResult<DeliveryInstruction> {
        if take(&mut self.failed_sends, source) {
            return Err(DomainError::WorkerUnavailable);
        }
        let delay_polls = take_pair(&mut self.delayed, destination).unwrap_or(0);
        Ok(DeliveryInstruction {
            delay_polls,
            timeout: take(&mut self.timeouts, destination),
            drop: message_type == FrameMessageType::Ack
                && take(&mut self.dropped_acks, destination),
            corrupt: take(&mut self.corruptions, destination),
        })
    }
}

fn add_once(values: &mut Vec<WorkerId>, value: WorkerId) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn remove_once(values: &mut Vec<WorkerId>, value: &WorkerId) {
    if let Some(index) = values.iter().position(|candidate| candidate == value) {
        values.remove(index);
    }
}

fn take(values: &mut Vec<WorkerId>, value: &WorkerId) -> bool {
    let Some(index) = values.iter().position(|candidate| candidate == value) else {
        return false;
    };
    values.remove(index);
    true
}

fn take_pair(values: &mut Vec<(WorkerId, u8)>, worker: &WorkerId) -> Option<u8> {
    values
        .iter()
        .position(|(candidate, _)| candidate == worker)
        .map(|index| values.remove(index).1)
}
