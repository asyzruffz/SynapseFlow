use std::sync::Arc;

use synapseflow_domain::execution::InFlightFrameLimit;
use synapseflow_domain::{DomainError, DomainResult};
use synapseflow_ports::WorkerId;

use crate::{LoopbackFault, LoopbackTransport, LoopbackWorker};

/// Creates independently addressable workers backed by one bounded loopback transport.
pub struct LoopbackNetwork {
    workers: Vec<WorkerId>,
    transport: Arc<LoopbackTransport>,
}

impl LoopbackNetwork {
    pub fn new(limit: InFlightFrameLimit, workers: Vec<WorkerId>) -> DomainResult<Self> {
        let transport = Arc::new(LoopbackTransport::new(limit, workers.clone())?);
        Ok(Self { workers, transport })
    }

    pub fn worker(&self, id: &WorkerId) -> DomainResult<LoopbackWorker> {
        if !self.workers.contains(id) {
            return Err(DomainError::WorkerUnavailable);
        }
        Ok(LoopbackWorker::new(id.clone(), self.transport.clone()))
    }

    pub fn inject(&self, fault: LoopbackFault) -> DomainResult<()> {
        self.transport.inject(fault)
    }

    pub fn transport(&self) -> Arc<LoopbackTransport> {
        self.transport.clone()
    }
}
