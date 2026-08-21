use std::path::PathBuf;

pub struct ModelFiles {
    pub dir: PathBuf,
    pub safetensors: Vec<PathBuf>,
    pub config: Option<PathBuf>,
    pub tokenizer: Option<PathBuf>,
}
