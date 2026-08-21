//! Model Source

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelSource {
    LocalPath(PathBuf),
    //HuggingFace { repo: String, file: String },
}

impl From<PathBuf> for ModelSource {
    fn from(path: PathBuf) -> Self {
        ModelSource::LocalPath(path)
    }
}
