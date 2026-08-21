use crate::backends::BackendType;

/// Errors returned by the public inference API.
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    /// A model source could not be discovered or validated.
    #[error(transparent)]
    Core(#[from] synapseflow_core::error::CoreError),

    /// The requested backend is not part of the supported runtime.
    #[error("backend {backend:?} is not available")]
    BackendUnavailable {
        /// Backend requested by the caller.
        backend: BackendType,
    },

    /// Model initialization failed after a source was resolved.
    #[error("model initialization failed: {message}")]
    Initialization {
        /// Safe diagnostic message for the caller.
        message: String,
    },

    /// Token generation failed after initialization.
    #[error("generation failed: {message}")]
    Generation {
        /// Safe diagnostic message for the caller.
        message: String,
    },
}

/// Result type used by public inference APIs.
pub type Result<T> = std::result::Result<T, InferenceError>;
