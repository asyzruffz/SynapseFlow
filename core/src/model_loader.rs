//! Model loader and manifest handling module.
//!
//! Responsibilities:
//! - Parse GGUF/ONNX/HuggingFace model formats
//! - Verify signed manifests (ed25519 signatures)
//! - Load weight shards with integrity checks

use anyhow::{Context, Result};
//pub use ed25519_dalek;
