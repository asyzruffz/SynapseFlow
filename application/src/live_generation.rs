use synapseflow_domain::{
    DomainError, DomainResult, GeneratedToken, GenerationEvent, GenerationOutput,
};
use synapseflow_ports::{GeneratedTokenSink, GenerationEventSink};

/// Forwards backend tokens into the application-owned public event stream.
pub(crate) struct TokenEventForwarder<'a> {
    events: &'a mut dyn GenerationEventSink,
}

impl<'a> TokenEventForwarder<'a> {
    pub(crate) fn new(events: &'a mut dyn GenerationEventSink) -> Self {
        Self { events }
    }
}

impl GeneratedTokenSink for TokenEventForwarder<'_> {
    fn emit_token(&mut self, token: GeneratedToken) -> DomainResult<()> {
        self.events.emit(GenerationEvent::Token(token))
    }
}

/// Compatibility collector for existing non-streaming shells.
pub(crate) struct OutputCollector {
    tokens: Vec<GeneratedToken>,
    terminal: Option<GenerationEvent>,
}

impl OutputCollector {
    pub(crate) const fn new() -> Self {
        Self {
            tokens: Vec::new(),
            terminal: None,
        }
    }

    pub(crate) fn into_output(self) -> DomainResult<GenerationOutput> {
        match self.terminal {
            Some(GenerationEvent::Completed { token_count })
                if token_count == self.tokens.len() =>
            {
                Ok(GenerationOutput::from_tokens(self.tokens))
            }
            Some(GenerationEvent::Cancelled) => Err(DomainError::SessionCancelled),
            Some(GenerationEvent::Failed { .. }) => Err(DomainError::GenerationFailed),
            _ => Err(DomainError::GenerationStreamInvalid),
        }
    }
}

impl GenerationEventSink for OutputCollector {
    fn emit(&mut self, event: GenerationEvent) -> DomainResult<()> {
        if self.terminal.is_some() {
            return Err(DomainError::GenerationStreamInvalid);
        }
        match event {
            GenerationEvent::Token(token) => self.tokens.push(token),
            terminal @ (GenerationEvent::Completed { .. }
            | GenerationEvent::Cancelled
            | GenerationEvent::Failed { .. }) => self.terminal = Some(terminal),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::OutputCollector;
    use synapseflow_domain::{DomainError, GeneratedToken, GenerationEvent};
    use synapseflow_ports::GenerationEventSink;

    #[test]
    fn collector_requires_one_matching_terminal_event() {
        let mut collector = OutputCollector::new();
        collector
            .emit(GenerationEvent::Token(GeneratedToken {
                id: 7,
                text: "seven".to_owned(),
            }))
            .expect("token should be accepted before terminal event");
        collector
            .emit(GenerationEvent::Completed { token_count: 1 })
            .expect("matching completed event should be accepted");

        assert_eq!(
            collector
                .into_output()
                .expect("completed event should collect output")
                .text,
            "seven"
        );
    }

    #[test]
    fn collector_rejects_events_after_the_terminal_outcome() {
        let mut collector = OutputCollector::new();
        collector
            .emit(GenerationEvent::Cancelled)
            .expect("first terminal event should be accepted");

        assert!(matches!(
            collector.emit(GenerationEvent::Cancelled),
            Err(DomainError::GenerationStreamInvalid)
        ));
    }
}
