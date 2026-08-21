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
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::error::CoreError;
    use crate::models::loader::ModelLoader;
    use crate::models::source::ModelSource;

    static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "synapseflow-core-test-{}-{sequence}",
                std::process::id()
            ));

            fs::create_dir(&path).expect("unique test directory should be created");
            Self(path)
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn local_model_source_preserves_path() {
        let path = PathBuf::from("test-model");
        let source = ModelSource::from(path.clone());

        match source {
            ModelSource::LocalPath(actual_path) => assert_eq!(actual_path, path),
        }
    }

    #[test]
    fn model_loader_rejects_an_empty_model_directory() {
        let directory = TestDirectory::new();

        match ModelLoader::load(ModelSource::from(directory.path().to_path_buf())) {
            Err(CoreError::MissingSafetensors { path }) => assert_eq!(path, directory.path()),
            Err(error) => panic!("expected missing safetensors error, got {error}"),
            Ok(_) => panic!("an empty model directory must not load"),
        }
    }

    #[test]
    fn model_loader_discovers_local_artifacts_by_their_expected_names() {
        let directory = TestDirectory::new();
        let weights = directory.path().join("weights.safetensors");
        let config = directory.path().join("config.json");
        let tokenizer = directory.path().join("tokenizer.json");

        fs::write(&weights, []).expect("test weights placeholder should be written");
        fs::write(&config, "{}").expect("test configuration should be written");
        fs::write(&tokenizer, "{}").expect("test tokenizer should be written");

        let files = match ModelLoader::load(ModelSource::from(directory.path().to_path_buf())) {
            Ok(files) => files,
            Err(error) => panic!("expected local artifacts to load, got {error}"),
        };

        assert_eq!(files.dir, directory.path());
        assert_eq!(files.safetensors, vec![weights]);
        assert_eq!(files.config, Some(config));
        assert_eq!(files.tokenizer, Some(tokenizer));
    }
}
