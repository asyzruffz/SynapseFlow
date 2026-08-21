//! Inference Config
//!

#[derive(Default, Clone)]
pub struct InferenceConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,
}
