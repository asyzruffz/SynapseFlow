use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use axum::response::sse::Event;
use synapseflow_domain::{GenerationEvent, PublicSessionId};
use tokio::sync::mpsc;
use tokio_stream::Stream;

/// Bounded live-only SSE translation for one subscribed session.
pub(super) struct SessionEventStream {
    session_id: PublicSessionId,
    receiver: mpsc::Receiver<GenerationEvent>,
    started: bool,
}

impl SessionEventStream {
    pub(super) fn new(
        session_id: PublicSessionId,
        receiver: mpsc::Receiver<GenerationEvent>,
    ) -> Self {
        Self {
            session_id,
            receiver,
            started: false,
        }
    }
}

impl Stream for SessionEventStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.started {
            self.started = true;
            return Poll::Ready(Some(Ok(started_event(&self.session_id))));
        }
        Pin::new(&mut self.receiver)
            .poll_recv(context)
            .map(|event| event.map(|event| Ok(generation_event(&self.session_id, event))))
    }
}

fn started_event(session_id: &PublicSessionId) -> Event {
    Event::default()
        .event("started")
        .data(serde_json::json!({ "session_id": session_id.as_str() }).to_string())
}

fn generation_event(session_id: &PublicSessionId, event: GenerationEvent) -> Event {
    match event {
        GenerationEvent::Token(token) => Event::default().event("token").data(
            serde_json::json!({
                "session_id": session_id.as_str(),
                "token_id": token.id,
                "text": token.text,
            })
            .to_string(),
        ),
        GenerationEvent::Completed { token_count } => Event::default().event("completed").data(
            serde_json::json!({
                "session_id": session_id.as_str(),
                "token_count": token_count,
            })
            .to_string(),
        ),
        GenerationEvent::Cancelled => Event::default()
            .event("cancelled")
            .data(serde_json::json!({ "session_id": session_id.as_str() }).to_string()),
        GenerationEvent::Failed { code } => Event::default().event("failed").data(
            serde_json::json!({
                "session_id": session_id.as_str(),
                "code": code.to_string(),
            })
            .to_string(),
        ),
    }
}
