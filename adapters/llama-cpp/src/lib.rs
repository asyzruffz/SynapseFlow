//! CPU-only GGUF/Llama backend isolated behind the SynapseFlow `ModelBackend` port.

#[cfg(any(feature = "runtime", test))]
mod compatibility;
#[cfg(feature = "runtime")]
mod runtime;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime")]
pub use runtime::LlamaCppBackend;
#[cfg(not(feature = "runtime"))]
pub use unavailable::LlamaCppBackend;

#[cfg(not(feature = "runtime"))]
mod unavailable {
    use synapseflow_domain::{DomainError, DomainResult, GenerationRequest, GenerationTerminal};
    use synapseflow_ports::{
        ExecutionCancellation, GeneratedTokenSink, ModelBackend, VerifiedModel,
    };

    /// Placeholder returned by builds that intentionally omit native llama.cpp support.
    pub struct LlamaCppBackend;

    impl LlamaCppBackend {
        pub fn new() -> DomainResult<Self> {
            Err(DomainError::BackendUnavailable)
        }
    }

    impl ModelBackend for LlamaCppBackend {
        fn generate(
            &self,
            _: &VerifiedModel,
            _: &GenerationRequest,
            _: &dyn ExecutionCancellation,
            _: &mut dyn GeneratedTokenSink,
        ) -> DomainResult<GenerationTerminal> {
            Err(DomainError::BackendUnavailable)
        }
    }
}
