//! Quantized kernels for deterministic inference operations across distributed peers.
//!
//! Module exports:
//! - `quantized_ops`: matmul, activation functions with fixed-point arithmetic
//! - `sampler`: local token sampling logic (no-network round trip)

pub mod quantized_ops; // Dense matrix mul + softmax variants
