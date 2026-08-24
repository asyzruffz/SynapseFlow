use std::collections::VecDeque;
use std::sync::Mutex;

use synapseflow_domain::execution::InFlightFrameLimit;
use synapseflow_domain::{DomainError, DomainResult, ErrorCode};
use synapseflow_ports::{ReceivedFrame, Transport, TransportReceipt, WorkerId};

/// Deterministic bounded queues for transport-port tests, not a worker transport.
pub struct InMemoryTransport {
    limit: InFlightFrameLimit,
    queues: Mutex<Vec<(WorkerId, VecDeque<ReceivedFrame>)>>,
    unavailable: Mutex<Vec<WorkerId>>,
}

impl InMemoryTransport {
    pub fn new(limit: InFlightFrameLimit, workers: Vec<WorkerId>) -> Self {
        Self {
            limit,
            queues: Mutex::new(
                workers
                    .into_iter()
                    .map(|worker| (worker, VecDeque::new()))
                    .collect(),
            ),
            unavailable: Mutex::new(Vec::new()),
        }
    }
}

impl Transport for InMemoryTransport {
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
            .unavailable
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .contains(worker);
        Ok(known && !unavailable)
    }

    fn send(
        &self,
        source: &WorkerId,
        destination: &WorkerId,
        canonical_frame: Vec<u8>,
    ) -> DomainResult<()> {
        if !self.is_available(destination)? {
            return Err(DomainError::WorkerUnavailable);
        }
        let mut queues = self.queues.lock().map_err(|_| DomainError::CacheFailure)?;
        let queue = queues
            .iter_mut()
            .find(|(worker, _)| worker == destination)
            .map(|(_, queue)| queue)
            .ok_or(DomainError::WorkerUnavailable)?;
        if queue.len() >= usize::from(self.limit.value()) {
            return Err(DomainError::FrameBoundsExceeded);
        }
        queue.push_back(ReceivedFrame::new(source.clone(), canonical_frame)?);
        Ok(())
    }

    fn receive(&self, recipient: &WorkerId) -> DomainResult<Option<ReceivedFrame>> {
        let mut queues = self.queues.lock().map_err(|_| DomainError::CacheFailure)?;
        queues
            .iter_mut()
            .find(|(worker, _)| worker == recipient)
            .map(|(_, queue)| queue.pop_front())
            .ok_or(DomainError::WorkerUnavailable)
    }

    fn acknowledge(
        &self,
        _: &WorkerId,
        destination: &WorkerId,
        _: &TransportReceipt,
    ) -> DomainResult<()> {
        self.available(destination)
    }

    fn reject(
        &self,
        _: &WorkerId,
        destination: &WorkerId,
        _: &TransportReceipt,
        _: ErrorCode,
    ) -> DomainResult<()> {
        self.available(destination)
    }

    fn cancel(
        &self,
        _: &WorkerId,
        destination: &WorkerId,
        _: &TransportReceipt,
    ) -> DomainResult<()> {
        self.available(destination)
    }

    fn shutdown(&self, worker: &WorkerId) -> DomainResult<()> {
        let mut unavailable = self
            .unavailable
            .lock()
            .map_err(|_| DomainError::CacheFailure)?;
        if !unavailable.contains(worker) {
            unavailable.push(worker.clone());
        }
        Ok(())
    }
}

impl InMemoryTransport {
    fn available(&self, worker: &WorkerId) -> DomainResult<()> {
        if self.is_available(worker)? {
            Ok(())
        } else {
            Err(DomainError::WorkerUnavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryTransport;
    use synapseflow_domain::execution::InFlightFrameLimit;
    use synapseflow_domain::DomainError;
    use synapseflow_ports::{Transport, WorkerId};

    #[test]
    fn provides_bounded_ordered_queues_and_shutdown_availability() {
        let source = WorkerId::new("loopback-a".to_owned()).expect("source is valid");
        let destination = WorkerId::new("loopback-b".to_owned()).expect("destination is valid");
        let transport = InMemoryTransport::new(
            InFlightFrameLimit::new(1).expect("limit is valid"),
            vec![source.clone(), destination.clone()],
        );

        transport
            .send(&source, &destination, vec![1])
            .expect("first frame should fit");
        assert!(matches!(
            transport.send(&source, &destination, vec![2]),
            Err(DomainError::FrameBoundsExceeded)
        ));
        let frame = transport
            .receive(&destination)
            .expect("receive should succeed")
            .expect("frame should be queued");
        assert_eq!(frame.source, source);
        assert_eq!(frame.bytes, vec![1]);

        transport
            .shutdown(&destination)
            .expect("shutdown should succeed");
        assert!(!transport
            .is_available(&destination)
            .expect("availability should succeed"));
    }
}
