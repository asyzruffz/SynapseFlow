//! Backend implementations for synapseflow-inference.
//!

pub mod candle;
pub mod llama_cpp;

#[derive(Debug, Clone)]
pub enum BackendType {
    Candle,
    LlamaCpp,
}
