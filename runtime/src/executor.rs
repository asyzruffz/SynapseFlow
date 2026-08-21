//! Subgraph execution engine that processes model layers across distributed peers.
//!
//! Responsibilities:
//! - Receive activations from upstream peer (or initial input)
//! - Execute assigned subgraph locally using candle or llama-cpp backend
//! - Return compressed logits/activations to downstream peer
//! - Support deterministic computation modes for reproducibility
