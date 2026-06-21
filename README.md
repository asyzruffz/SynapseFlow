# **SynapseFlow** - Distributed LLM Inference System

A torrent‑style, peer-to-peer distributed inference system. This directory documents the complete module structure and implementation details for each crate.

---

## Project Structure Overview

See [Project Structure Overview](docs/Project Structure Overview.md)

---

## Workspace Cargo Structure

### **Top-level `Cargo.toml`** (`SynapseFlow/Cargo.toml`)

Defines workspace with 8 member crates:

| Crate | Description |
|-------|-------------|
| `synapseflow-core` | Model loading, manifest handling, shard indexing (sled storage) |
| `synapseflow-runtime` | Subgraph execution engine on local peers |
| `synapseflow-network` | QUIC transport + libp2p P2P discovery overlay |
| `synapseflow-coord` | Orchestrator: planner and session manager |
| `synapseflow-security` | Crypto verification, attestation, auditing (checksums) |
| `synapseflow-api` | REST endpoints for prompts and admin operations |
| `synapseflow-utils` | Serialization helpers, storage utilities |
| `synapseflow-incentive` *(stub)* | External credit/reputation integration point |

---

## Testing & Benchmarking Guide

Each crate includes unit tests comparing outputs of each shard against canonical single-machine run to verify determinism. The plan documents specify:

| Metric | Target Threshold |
|--------|------------------|
| End-to-end latency per token  | < 150ms (small model) |
| Network bandwidth/MB          | ~40+ MB/s over QUIC |
| Compute utilization           | > 70% GPU when available |
| Recovery time                 | < 2s on peer failure |
| Output divergence             | NumDiff(ours, ref) < epsilon |

### Microbenchmark commands:

```bash
# Measure activation sizes for common layers (1k params each):
cargo bench --package synapseflow-runtime -- subgraph_execute

# Profile QUIC throughput under packet loss conditions  
tracing-subscriber::FmtLayer; quinn_endpoint.bench()

# Validate re-execution overhead on replica failure: 
integration_tests/replay_from_checkpoint.rs   
```

---

## License (placeholder for future)
MIT + Apache 2.0 dual license preferred; ensure model licenses permit distributed execution under chosen terms.
