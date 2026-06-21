# Local Single-Machine Prototype Plan

This document outlines the concrete implementation plan for **Step 1: Local single-machine prototype** of SynapseFlow. The goal is to validate core mechanics (sharding, activation streaming, checkpointing) on a loopback network before moving to multi-machine testing.

---

## Objectives

- Load and split a small model into independent shards
- Simulate inter-peer communication via local IPC or QUIC over loopback
- Execute subgraphs across shuffled shard orderings (to emulate P2P routing)
- Measure activation sizes, latencies, checksums, failure recovery overhead
- Demonstrate fault tolerance with replica fallback

---

## Target Model Selection

| Criterion | Choice | Rationale |
| --- | --- | --- |
| **Model size** | 100M – 500M params (e.g., TinyLlama subset or Phi-vision distilled) | Fits in memory easily; fast enough to iterate quickly |
| **Format** | GGUF via `llama-rs` (candle-backend compatible) | Well-supported, easy sharding on CPU-only test box |
| **Layers** | 8–16 transformer layers total | Enough subgraphs for meaningful testing but small overhead |

### Example: TinyLlama-20M

*   Total parameters ~75 million (~300 MB in f16)
*   Split into `s-0`, `s-1` (e.g., 4 layers each)
*   Can be further subdivided for more complex sharding tests

---

## Shard Layout and Communication Model

### Sharded Architecture Example (Layer-wise, 2 peers)

```text
Model: [Embedding] → L[0..3],L[4..7] → [Norm + Proj] → Output Head

Shard s-0: Embedding → Layer Blocks [0..3] → Partial Norm  
Shard s-1: Partial Norm continuation → Layer Blocks [4..7] → Final Norm/Proj
```

### Simulation Approach (Local Single Machine)

**Option A — Local IPC via Unix sockets or named pipes:**

*   Fast, simple; avoids QUIC protocol overhead during prototyping.
*   Allows testing activation shapes and chunking without network jitter.
*   Later replaced by actual `quinn` over loopback.

**Option B — Loopback QUIC (preferred for realism):**

*   Start a dedicated local peer process (`localhost:80xx`).
*   Run the first shard locally as "peer 1" and second on a separate Tokio runtime or subprocess =Peer2".
*   Connect via `quinn::Connection` over `tcp://127.0.0.1:...`.

> **Recommendation for MVP:** Use Option A (local IPC) to demonstrate mechanics, then swap in QUIC as an exercise before moving outside the machine.

---

## Implementation Outline

### Step 1 — Project Skeleton and Dependencies

Create a new crate `synapseflow-cli` into the project structure:

```text
├── core/src/shards/                 
│   └── mod.rs              // Shard struct + manifest parsing code  
├── runtime/src/
│   ├── executor.rs         // Subgraph runner trait and impls per shard  
│   └── kernels/mod.rs      // Quantized matmul/gelu/reload, etc.
├── network/src/            # Communication layer (IPC or QUIC)
│   └── transport/          // Send activation frames; ACK/NACK handling
│       └── mod.rs          // Send activation frames; ACK/NACK handling
│       └── frame.rs        // Frame struct: header + payload + checksum  
├── coord/src/
│   └── planner.rs          // Build execution plan given shard graph  
|
├── cli/                    // Newly created crate into the project
│   └── src/
│      ├── main.rs          // CLI entrypoint for testing
│      └── [dependencies]   # candle, tokio, quinn (or socket2), zstd, ed25519-dalek  
└── Cargo.toml              # Main manifest with all dependencies  
```

### Step 2 — Shard Splitting

* Write a script in Rust or Python to:
  * Load the model via `llama-rs` or candle's GGUF parser.
  * Slice weight tensors for chosen layers into individual `.bin` files (or compressed `.zst`).
  *   Compute checksums (`sha256`) per shard and store in a local JSON manifest stub:
```json
{
  "shard_id" : "s-0",  
  "layers": [0,1,2,3],  
  "checksum" : "e7f9..."
}  
``` 
* Optionally sign the manifest locally for security testing.

### Step 3 — Subgraph Execution and Activation Streaming

Implement a minimal `Executor` that:
- Loads weights from shard files on disk (lazy I/O).
- Takes compressed activations as input frames, decompresses with zstd.
- Runs forward pass through its sublayers; buffers partial-norm output.
- Compresses & ACKs resulting tensor chunk to next executor via IPC/QUIC.

Include:
*   **Checkpointing**: After each layer group (e.g., after shard `s-0`), compute a small hash of the activation state and store for potential rollback/re-execute later.  
*   **Re-execution logic** on mismatch or timeout: planner requests replay from replica, if present.

### Step 4 — Fault Tolerance Test Harness

Write unit tests that simulate failures:
1. Shard `s-0` hangs → detect via ACK/timeout; fallback to a local replica copy of shard weights and re-run.  
2. Activation frame checksum mismatch on peer `s-1` → NACK, resend from origin or retry with corrected version.  

Measure recovery time vs canonical latency (single-machine baseline).

### Step 5 — Metrics Collection

Instrument code to print:
*   Latency per activation transfer (`tokio::time::Instant`).  
*   Compression ratio before/after `zstd` at various levels.  
*   Activation tensor sizes; memory footprint per peer subgraph.  

Export metrics as Prometheus-style JSON if desired, for later plotting or dashboarding.

---

## Test Scenarios and Expected Metrics

| Scenario | Setup | What to Measure |
| --- | --- | --- |
| **Basic correctness** | 2-layer shards → loopback IPC | End-to-end latency per prompt; final token matches single-node run (within tolerance). |
| **Compression overhead** | Toggle `zstd` levels {1,3,6} | Bandwidth saved vs CPU cost added. Target: >50% reduction at level 3+. |
| **Failure & fallback** | Shard replica on same disk; inject hang in one peer | Recovery time; ensure re-exec from last checkpoint doesn't exceed SLA much (>2x normal is acceptable for prototype). |

---

## Timeline and Milestones (1–2 weeks)

### Week 1: Core Mechanics
*   [Day 3] Model load + shard splitting script working.  
*   [Day 5] Subgraph executor compiles & runs forward pass locally with IPC transport stubs.  

### Week 2: Integration Testing
*   [Day 7-8] Add QUIC or socket-based frame exchange; demonstrate activation streaming between two local processes (or threadpools).  
*   [Day 10] Verify fault tolerance scenarios pass unit tests, logging all metrics to JSON/CSV.  

### Deliverable by Week End:
A working prototype that loads a small model split into shards and performs cross-shard inference using simulated peer communication over loopback, with checksums, replication fallback, and basic performance instrumentation ready for multi-machine migration later.

--- 

## Notes on Determinism & Numerics (Even Locally)

*   Disable non-deterministic BLAS flags; use `candle`'s deterministic math mode if available.  
*   Quantize weights/int8 activations when testing to avoid FP drift issues that might compound in multi-machine tests later.
