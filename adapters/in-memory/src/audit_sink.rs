use std::sync::Mutex;

use synapseflow_domain::{DomainError, DomainResult};
use synapseflow_ports::{AuditEvent, AuditSink};

/// Captures safe audit events for deterministic assertions.
#[derive(Default)]
pub struct InMemoryAuditSink {
    events: Mutex<Vec<AuditEvent>>,
}

impl InMemoryAuditSink {
    pub fn events(&self) -> DomainResult<Vec<AuditEvent>> {
        self.events
            .lock()
            .map(|events| events.clone())
            .map_err(|_| DomainError::CacheFailure)
    }
}

impl AuditSink for InMemoryAuditSink {
    fn record(&self, event: AuditEvent) -> DomainResult<()> {
        self.events
            .lock()
            .map_err(|_| DomainError::CacheFailure)?
            .push(event);
        Ok(())
    }
}
