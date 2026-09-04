use std::sync::Arc;

use synapseflow_domain::{
    DomainResult, GenerationEvent, GenerationRequest, GenerationTerminal, PublicSessionId,
};
use synapseflow_ports::GenerationEventSink;

use crate::{
    GenerationOrchestrator, GenerationSessionManager, SessionStartRequest, SessionStartResult,
    SessionTerminal,
};

/// Runs either selected execution profile through the shared durable session manager.
pub struct SessionExecutionService {
    sessions: Arc<GenerationSessionManager>,
    generation: Arc<GenerationOrchestrator>,
}

impl SessionExecutionService {
    pub fn new(
        sessions: Arc<GenerationSessionManager>,
        generation: Arc<GenerationOrchestrator>,
    ) -> Self {
        Self {
            sessions,
            generation,
        }
    }

    /// Begins, executes, persists, audits, and then delivers one terminal session outcome.
    pub fn execute(
        &self,
        session_request: SessionStartRequest,
        generation_request: GenerationRequest,
        events: &mut dyn GenerationEventSink,
    ) -> DomainResult<SessionStartResult> {
        let started = self.sessions.begin(session_request)?;
        self.execute_started(started, generation_request, events)
    }

    /// Executes a session already durably accepted by the caller.
    ///
    /// This lets a transport return its accepted session representation before
    /// work starts, while preserving the application-owned execution lifecycle.
    pub fn execute_started(
        &self,
        started: SessionStartResult,
        generation_request: GenerationRequest,
        events: &mut dyn GenerationEventSink,
    ) -> DomainResult<SessionStartResult> {
        if started.replayed {
            return Ok(started);
        }
        let session_id = started.session.id.clone();
        if let Err(error) = self.sessions.mark_running(&session_id) {
            if error != synapseflow_domain::DomainError::SessionStateInvalid {
                return Err(error);
            }
            let cancellation = self.sessions.cancelled_result(&session_id)?;
            if cancellation == synapseflow_domain::CancellationResult::Requested {
                self.sessions
                    .finish(&session_id, SessionTerminal::cancelled(cancellation))?;
                events.emit(GenerationEvent::Cancelled)?;
            }
            return Ok(started);
        }
        let cancellation = self.sessions.activate(&session_id)?;
        let result = self.generation.generate_until_terminal(
            generation_request,
            cancellation.as_ref(),
            events,
        );
        let finished = self.finish_and_deliver(&session_id, result, events);
        let deactivated = self.sessions.deactivate(&session_id);
        finished?;
        deactivated?;
        Ok(started)
    }

    fn finish_and_deliver(
        &self,
        session_id: &PublicSessionId,
        result: DomainResult<GenerationTerminal>,
        events: &mut dyn GenerationEventSink,
    ) -> DomainResult<()> {
        match result {
            Ok(GenerationTerminal::Completed { token_count }) => {
                self.sessions
                    .finish(session_id, SessionTerminal::completed(token_count))?;
                events.emit(GenerationEvent::Completed { token_count })
            }
            Ok(GenerationTerminal::Cancelled) => {
                let cancellation = self.sessions.cancelled_result(session_id)?;
                self.sessions
                    .finish(session_id, SessionTerminal::cancelled(cancellation))?;
                events.emit(GenerationEvent::Cancelled)
            }
            Err(error) => {
                self.sessions
                    .finish(session_id, SessionTerminal::failed(error.clone()))?;
                events.emit(GenerationEvent::Failed { code: error.code() })?;
                Err(error)
            }
        }
    }
}
