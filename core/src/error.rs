use std::io;
use std::path::PathBuf;

/// Errors returned by SynapseFlow core contracts.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The model directory could not be read.
    #[error("failed to read model directory {path}: {source}")]
    ReadModelDirectory {
        /// Directory that could not be read.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },

    /// The requested model source does not contain supported weight files.
    #[error("no .safetensors files found in {path}")]
    MissingSafetensors {
        /// Directory searched for model weights.
        path: PathBuf,
    },
}

/// Result type used by SynapseFlow core public APIs.
pub type Result<T> = std::result::Result<T, CoreError>;
