use synapseflow_domain::{DomainResult, GeneratedToken, GenerationEvent};

/// Receives ordered generated tokens without coupling a backend to a transport.
pub trait GeneratedTokenSink {
    fn emit_token(&mut self, token: GeneratedToken) -> DomainResult<()>;
}

/// Receives the application-owned public token and terminal event sequence.
pub trait GenerationEventSink {
    fn emit(&mut self, event: GenerationEvent) -> DomainResult<()>;
}
