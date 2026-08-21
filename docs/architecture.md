# Architecture

## System purpose

SynapseFlow executes a language model whose immutable weight shards are placed on one or more workers. A client submits a generation request to a node. The node authenticates and limits the request, obtains a verified model manifest, creates a route through eligible shard workers, and owns the request session until it completes, fails, or is cancelled.

Workers execute ordered layer groups and transfer activation frames. The final stage returns logits to the node, which applies the requested sampling policy and streams tokens to the client. Every action is traceable to a model version, shard hash, worker identity, and session identifier.

```text
client
  │ request, identity, generation policy
  ▼
node/API ── authorization, quotas, streaming response
  ▼
application service ── tokenization, planning, deadlines, session ownership
  ▼
worker A ── verified shard ── activation frame ──► worker B ──► final worker
  ▲                                                               │ logits
  └──────────── audit events, checkpoints, retry/fallback ◄──────┘
```

## Dependency direction

The workspace is organized around stable contracts, not infrastructure choices.

```text
applications (CLI, node/API)
        │
application services (generation, planning, policy, sessions)
        │
domain (model manifest, shard, tensor, frame, session state)
        │
ports (backend, transport, shard store, peer directory, audit sink)
        │
adapters (model runtime, remote registry, local cache, QUIC, database, telemetry)
```

Domain types and port traits must not depend on Tokio, a particular model runtime, QUIC, a database, or an HTTP framework. Application services depend only on domain types and ports. Adapters own external dependencies, allowing deterministic in-memory tests of planning and session behavior.

## Components

| Component | Responsibility |
|---|---|
| Model registry | Resolves immutable model versions and signed manifests from allowed remote sources. |
| Shard store | Downloads, verifies, caches, opens, evicts, and serves authorized shards. |
| Backend | Tokenizes, executes a whole model or declared layer range, and samples logits. |
| Planner | Chooses a route using model compatibility, worker capability, health, replication, deadline, and policy. |
| Session manager | Owns request state, cancellation, deadlines, checkpoints, retries, result delivery, and cleanup. |
| Transport | Moves validated frames with peer authentication, bounded queues, flow control, and backpressure. |
| Peer directory | Stores enrolled worker identity, capabilities, health, location, and shard availability. |
| Audit and telemetry | Emits privacy-safe audit events, traces, logs, and service metrics. |

## Sharding strategy

Start with layer-wise execution because it has the simplest correctness model. Group adjacent layers into blocks once measurements justify fewer network hops. Tensor parallelism and distributed MoE require synchronization/routing designs with distinct failure and bandwidth properties and are separate architectural decisions.

| Strategy | Appropriate use |
|---|---|
| Layer-wise | Baseline correctness, simple routing, and clear checkpoint boundaries. |
| Block-chunking | Lower hop count and improved compute/network balance. |
| Tensor parallel | Specialized high-bandwidth clusters after collective-communication support exists. |
| MoE distribution | Expert models after routing, scheduling, and fairness controls exist. |

## Security and reliability

Workers use mutually authenticated TLS over QUIC. Manifests and shards are authenticated by publisher signature and content hash. Authorization controls who may fetch or execute a model. The protocol bounds input before decoding or allocating, propagates deadlines, and treats cancellation as idempotent.

Replicas, checkpoints, circuit breakers, and a retry budget make failure handling explicit. Hash validation detects corruption; it does not by itself establish trust or prevent activation leakage. Optional TEEs, differential privacy, MPC, and homomorphic encryption require a documented threat model and performance justification before adoption.

## Operational properties

The system exposes request/token latency, activation bytes, compression ratio, queue depth, worker health, retry/fallback count, model-cache state, and error rate. Every request carries trace and session identifiers. Logs and audit events exclude prompt text, raw activations, weights, credentials, and other sensitive payloads unless an explicit, reviewed policy allows them.
