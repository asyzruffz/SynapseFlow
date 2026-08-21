//! SynapseFlow Core Library
//!
//! This module handles:
//! - Model manifest parsing (GGUF/ONNX/HF formats)
//! - Manifest signature verification
//! - Local shard metadata storage via sled

pub mod error;
pub mod models;
mod shard_index;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::models::source::ModelSource;

    #[test]
    fn local_model_source_preserves_path() {
        let path = PathBuf::from("test-model");
        let source = ModelSource::from(path.clone());

        match source {
            ModelSource::LocalPath(actual_path) => assert_eq!(actual_path, path),
        }
    }
}
