use std::sync::Arc;

use synapseflow_application::{SessionExecutionService, SessionStartResult};
use synapseflow_domain::{DomainError, GenerationEvent, GenerationRequest, PublicSessionId};
use synapseflow_ports::GenerationEventSink;

use crate::NodeWorkflowRegistry;

/// Moves blocking model execution off the HTTP request executor after acceptance.
pub(super) fn start(
    execution: Arc<SessionExecutionService>,
    started: SessionStartResult,
    request: GenerationRequest,
    workflows: Arc<NodeWorkflowRegistry>,
) {
    std::mem::drop(tokio::task::spawn_blocking(move || {
        let mut events = WorkflowEvents {
            workflows,
            session_id: started.session.id.clone(),
        };
        let _ = execution.execute_started(started, request, &mut events);
    }));
}

/// Delivers application-owned live events to the node workflow registry.
struct WorkflowEvents {
    workflows: Arc<NodeWorkflowRegistry>,
    session_id: PublicSessionId,
}

impl GenerationEventSink for WorkflowEvents {
    fn emit(&mut self, event: GenerationEvent) -> Result<(), DomainError> {
        self.workflows.deliver(&self.session_id, event)
    }
}
