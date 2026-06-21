### **SynapseFlow**

SynapseFlow, a torrent‑style, peer‑to‑peer LLM inference system is **an experimental research prototype** that still requires careful design across partitioning, networking, determinism, security, and incentives. This document lays out a concrete, implementable plan for a **lightweight Rust-based** distributed inference runtime: architecture, protocol, data formats, Rust module breakdown, security/privacy measures, reliability strategies, and a staged prototype and benchmark plan.

---

### **System Architecture**

| **Component** | **Purpose** | **Key Requirements** | **Rust crates / tech** |
| --- | --- | --- | --- |
| **Shard Manager** | Tracks which peer stores which weight shards | Signed manifests; versioning; replication metadata | ``sled``/``rocksdb`` for local index |
| **Runtime Worker** | Executes model subgraph on local shard | Deterministic numerics; quantized kernels; memory mgmt | ``candle`` or ``llama-rs``; ``ndarray`` |
| **Transport Layer** | Stream activations between peers | Low latency, backpressure, compression, retries | ``quinn`` (QUIC), ``tokio`` |
| **Discovery & Overlay** | Peer discovery and routing | NAT traversal, DHT or libp2p, peer health | ``libp2p`` |
| **Orchestrator** | Schedules inference across peers | Latency-aware scheduling, fallback, replication | Lightweight central or decentralized |
| **Verifier & Auditor** | Validate correctness of shards/outputs | Checksums, attestations, re-execution | ``ring``/``rustls``, signature libs |
| **Incentive Layer** | Encourage availability | Credits, micropayments, reputation | External service or token system |

#### **High-level flow**

1.  **Client** submits prompt to local coordinator.
2.  **Coordinator** consults Shard Manager to build an execution plan (ordered list of peers and subgraphs).
3.  **Coordinator** streams tokenized input to first peer.
4.  **Each peer** runs its assigned subgraph, streams activation chunks to the next peer.
5.  **Final peer** returns logits/tokens; local sampler produces output.
6.  **Auditing**: checksums and optional re-execution on replicas validate result.

---

### **Protocol and Data Formats**

#### **Shard manifest**

A **signed JSON manifest** per model version describing architecture and shard mapping.

```json
{
  "model_id" : "llama4-scout-1",
  "version" : "2026-06-18",
  "architecture" : "transformer",
  "shards" : [
    { "shard_id" : "s-0001", "layers" : [0,1,2], "checksum" : "sha256:...", "replicas" : 2 },
    { "shard_id" : "s-0002", "layers" : [3,4,5], "checksum" : "sha256:...", "replicas" : 2 }
  ],
  "signature" : "ed25519:..."
}
```

*   **Signed** by model publisher to prevent tampering.
*   Each peer stores its shard metadata and local checksum.

#### **Activation streaming protocol**

Use **chunked binary frames** over QUIC with protobuf or compact binary framing. Each frame:

*   **Header**: session\_id, frame\_index, total\_frames, tensor\_dtype, shape, checksum
*   **Payload**: compressed activation bytes (e.g., zstd)
*   **Control messages**: ACK, NACK, RETRY, CANCEL, HEARTBEAT

**Example control flow**:

*   Sender sends FRAME(0..N) with window\_size = 4.
*   Receiver ACKs frames; if checksum mismatch, sends NACK(frame\_index) and requests re-send or fallback to replica.

#### **Sampling and tokenization**

*   Tokenization is local; token IDs are small integers.
*   Final logits returned as compressed float16 or int8 logits; local sampler performs sampling to avoid extra round trips.

#### **Checkpointing and rollback**

*   After each layer group, include a **lightweight checkpoint**: a small hash of activation state and sequence number. Coordinator can request re-execution from last checkpoint on replica if mismatch or peer failure.

---

### **Sharding Strategies and Tradeoffs**

| **Strategy** | **Comm Pattern** | **Pros** | **Cons** |
| --- | --- | --- | --- |
| **Layer-wise** | Sequential activations between peers | Simple; minimal cross-layer synchronization | High latency; large activation transfers |
| **Tensor-parallel** | Parallel compute with allreduce-like comms | Lower per-peer memory; parallelism | Complex allreduce; heavy bandwidth |
| **MoE expert distribution** | Route tokens to experts on different peers | Natural fit for MoE models; sparse activation | Routing overhead; load imbalance |
| **Block-chunking (hybrid)** | Group several layers per shard | Balance latency and compute | Requires careful partitioning heuristics |

**Recommendation:** Start with **layer-wise** or **block-chunking** for prototype simplicity; evaluate MoE later.

---

### **Rust Implementation Plan**

#### **Top-level modules**

1.  **core**
    *   **model\_loader**: parse GGUF/ONNX/HF formats; verify manifest signatures.
    *   **shard\_index**: local shard metadata store.
2.  **runtime**
    *   **executor**: runs subgraph; exposes apply(activations) -> activations.
    *   **kernels**: quantized kernels; deterministic math modes.
3.  **network**
    *   **transport**: QUIC connection manager; frame encoder/decoder.
    *   **discovery**: libp2p DHT or bootstrap list.
4.  **coord**
    *   **planner**: builds execution plan; latency-aware.
    *   **session\_manager**: tracks in-flight inference sessions.
5.  **security**
    *   **crypto**: signature verification; TLS/DTLS; attestation.
    *   **audit**: checksum verification; logging.
6.  **api**
    *   **local\_api**: REST/gRPC for user prompts.
    *   **admin**: shard upload, health, metrics.
7.  **utils**
    *   **serde**: JSON/protobuf helpers.
    *   **storage**: local disk cache, eviction policy.

#### **Suggested crates and tools**

*   **Async runtime**: tokio
*   **QUIC**: quinn
*   **P2P**: libp2p (Rust implementation)
*   **Serialization**: serde, prost (protobuf)
*   **Crypto**: ring, ed25519-dalek
*   **Model runtimes**: candle (Hugging Face Rust ML), llama-rs for GGUF
*   **Compression**: zstd or snap crates
*   **Storage**: sled or rocksdb
*   **Metrics**: prometheus client crate

#### **Determinism and numerics**

*   Use **fixed quantization** (e.g., int8 or float16) and a single, well-tested kernel implementation (preferably in Rust) to avoid cross-device FP drift.
*   Provide a **deterministic math mode** flag that disables non-deterministic BLAS optimizations.
*   Include **unit tests** that compare outputs against a canonical reference for each shard.

### **Security, Privacy, and Trust**

#### **Threats**

*   **Model theft**: peers copying shards and redistributing.
*   **Malicious peers**: returning corrupted activations or poisoned outputs.
*   **Data leakage**: activations reveal prompt content.

#### **Protections**

*   **Signed manifests and shards**: shards are encrypted at rest and signed; peers verify signatures before use.
*   **Attestation**: optional TEE (Intel SGX, AMD SEV) attestation for trusted execution.
*   **Checksum + replication**: require at least k replicas and cross-verify outputs; re-execute on replica if mismatch.
*   **MPC / HE**: for high-sensitivity workloads, consider secure multi-party computation or homomorphic encryption for parts of the pipeline (high overhead).
*   **Differential privacy**: add noise to activations or outputs when appropriate.
*   **Access control**: shards only served to authenticated peers; rate limits and quotas.

#### **Incentives and governance**

*   **Reputation system**: peers earn reputation for uptime and correctness.
*   **Credit system**: clients pay credits to peers for compute; credits fund storage and availability.
*   **Legal**: ensure model license permits distributed storage and execution; enforce via signed manifests and usage policies.

---

### **Reliability, Fault Tolerance, and Orchestration**

#### **Fault tolerance patterns**

*   **Replication**: each shard has r replicas; planner prefers low-latency replicas.
*   **Erasure coding**: reduce storage overhead while enabling reconstruction from partial shards.
*   **Timeouts and retries**: per-frame timeouts; fallback to replica or local re-execution.
*   **Checkpointing**: intermediate activation checkpoints to limit rework on failure.

#### **Orchestration options**

*   **Decentralized**: libp2p DHT for discovery; coordinator runs on client side; good for censorship resistance.
*   **Hybrid**: small set of bootstrap orchestrators (trusted) for scheduling and reputation; peers remain P2P for data transfer.
*   **Centralized**: single scheduler for research prototype to simplify planning and measurement.

---

### **Testing, Prototype Plan, and Benchmarks**

#### **Prototype stages**

1.  **Local single-machine prototype**
    *   Use a small model (100M–1B) split into 2–4 shards.
    *   Implement transport over loopback; measure activation sizes and latency.
2.  **Two-machine prototype**
    *   Identical hardware; measure network latency and throughput; validate checksums and re-execution.
3.  **Heterogeneous cluster**
    *   Mix CPU-only and GPU machines; test numeric determinism and fallback strategies.
4.  **Public P2P pilot**
    *   Small set of volunteer peers; enable discovery and reputation; measure real-world churn.

#### **Benchmarks to collect**

*   **End-to-end latency** per token and per prompt.
*   **Bandwidth per peer** (MB/s) and total network traffic.
*   **Compute utilization** per peer.
*   **Failure recovery time** and re-execution overhead.
*   **Output divergence** vs canonical single-node run (numerical error).
*   **Cost per token** (if using paid compute or credits).

#### **Example microbenchmarks**

*   Measure activation tensor sizes for common models and batch sizes.
*   Measure compression ratio with zstd at different levels.
*   Measure QUIC throughput and per-frame latency under packet loss.

---

### **Risks, Open Problems, and Mitigations**

*   **High latency vs centralized GPU**: mitigate by grouping more layers per peer, batching, and preferring high-bandwidth peers.
*   **Numerical drift**: mitigate by enforcing single runtime and quantization; allow re-execution on replicas.
*   **Privacy leakage**: mitigate by restricting use to non-sensitive workloads or using TEEs/MPC.
*   **Incentive alignment**: prototype with volunteer peers; add credits later.
*   **Legal/licensing**: verify model license allows distributed storage and execution.

---

### **Concrete Next Steps (developer checklist)**

1.  **Select target model and format** (small model for prototype).
2.  **Implement manifest format and signing**; publish a test manifest.
3.  **Build minimal runtime** using llama-rs or candle to run a subgraph locally.
4.  **Implement QUIC-based frame transport** and a simple control protocol.
5.  **Prototype layer-wise shard execution** on two machines; collect latency/bandwidth metrics.
6.  **Add checksum verification and replica fallback**.
7.  **Iterate: add libp2p discovery, reputation, and basic incentive accounting**.
8.  **Run security threat model and decide on TEE/MPC tradeoffs**.

**Deliverables I can produce next (pick any and I will produce immediately):**

*   **Protocol spec**: protobuf definitions and frame-level state machine.
*   **Rust module skeleton**: cargo workspace layout and lib.rs stubs for each module.
*   **Minimal QUIC frame example**: Rust code showing chunked activation send/receive.
*   **Benchmark plan**: exact commands, metrics to collect, and expected thresholds.

I will proceed with any one of the deliverables above; I will not ask clarifying questions and will assume you want the protocol spec first.
