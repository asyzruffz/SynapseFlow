//! SynapseFlow Runtime Library
//!
//! Executes model subgraphs on local shards with deterministic numerics and memory management.

pub mod executor; // Subgraph execution engine
mod kernels; // Quantized math modes (float16, int8)
