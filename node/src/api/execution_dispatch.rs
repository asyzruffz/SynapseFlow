use std::sync::Arc;

use synapseflow_application::{SessionExecutionService, SessionStartResult};
use synapseflow_domain::{DomainError, GenerationEvent, GenerationRequest};
use synapseflow_ports::GenerationEventSink;

/// Moves blocking model execution off the HTTP request executor after acceptance.
pub(super) fn start(
    execution: Arc<SessionExecutionService>,
    started: SessionStartResult,
    request: GenerationRequest,
) {
    std::mem::drop(tokio::task::spawn_blocking(move || {
        let mut events = UndeliveredEvents;
        let _ = execution.execute_started(started, request, &mut events);
    }));
}

/// Temporary event target until the live SSE bridge is installed.
///
/// The application persists the terminal transition and audit before invoking
/// this sink, so discarding live delivery cannot weaken durable cleanup.
struct UndeliveredEvents;

impl GenerationEventSink for UndeliveredEvents {
    fn emit(&mut self, _: GenerationEvent) -> Result<(), DomainError> {
        Ok(())
    }
}
