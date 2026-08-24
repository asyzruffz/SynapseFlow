/// Framework-independent cancellation observation for a bounded execution call.
pub trait ExecutionCancellation: Send + Sync {
    fn is_cancelled(&self) -> bool;
}

/// Cancellation probe for callers that have no active cancellation request.
#[derive(Default)]
pub struct NeverCancelled;

impl ExecutionCancellation for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}
