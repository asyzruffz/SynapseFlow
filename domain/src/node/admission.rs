/// Result of bounded application admission before execution begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionDecision {
    Admitted,
    Rejected(AdmissionRejection),
}

/// Privacy-safe reason for rejecting work before execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionRejection {
    RateLimited,
    PrincipalCapacity,
    NodeCapacity,
    AuditUnavailable,
}
