use std::fs;
use std::path::{Path, PathBuf};

use super::files::ModelFiles;
use super::source::ModelSource;
use crate::error::{CoreError, Result};

pub struct ModelLoader;

impl ModelLoader {
    pub fn load(source: ModelSource) -> Result<ModelFiles> {
        match source {
            ModelSource::LocalPath(path) => Self::load_from_path(&path),
        }
    }

    fn load_from_path(path: &Path) -> Result<ModelFiles> {
        let mut safetensors_files = Vec::<PathBuf>::new();

        let model_dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "safetensors" {
                    safetensors_files.push(path.to_path_buf());
                }
            }
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        };

        if safetensors_files.is_empty() {
            let entries =
                fs::read_dir(&model_dir).map_err(|source| CoreError::ReadModelDirectory {
                    path: model_dir.clone(),
                    source,
                })?;

            for entry in entries {
                let entry = entry.map_err(|source| CoreError::ReadModelDirectory {
                    path: model_dir.clone(),
                    source,
                })?;
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("safetensors") {
                    safetensors_files.push(p);
                }
            }
        }

        if safetensors_files.is_empty() {
            return Err(CoreError::MissingSafetensors { path: model_dir });
        }

        let config_path = model_dir.join("config.json");
        let config_path = if config_path.exists() {
            Some(config_path)
        } else {
            None
        };

        let tokenizer_path = model_dir.join("config.json");
        let tokenizer_path = if tokenizer_path.exists() {
            Some(tokenizer_path)
        } else {
            None
        };

        Ok(ModelFiles {
            dir: model_dir,
            safetensors: safetensors_files,
            config: config_path,
            tokenizer: tokenizer_path,
        })
    }
}
