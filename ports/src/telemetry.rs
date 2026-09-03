use synapseflow_domain::{AdmissionDecision, AuthorizationDecision, ErrorCode, PublicSessionId};

/// Bounded, payload-free operational signals emitted by application use cases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryEvent<'a> {
    IdentityVerified,
    AuthorizationEvaluated {
        decision: AuthorizationDecision,
    },
    AdmissionEvaluated {
        decision: AdmissionDecision,
    },
    SessionTerminal {
        session_id: &'a PublicSessionId,
        failure: Option<ErrorCode>,
    },
}

/// Emits non-authoritative operational telemetry without weakening audit behavior.
pub trait TelemetrySink: Send + Sync {
    fn record(&self, event: TelemetryEvent<'_>);
}
