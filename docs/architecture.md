# Architecture

> This is the target architecture. Milestone 2 delivers the verified-local
> inference slice, and Milestone 3 delivers its bounded two-worker loopback
> sharding slice: schema-v2 manifests, activation frames, planning/session
> contracts, Loom layer-range execution, and bounded replica recovery. QUIC,
> remote worker enrolment, authentication/authorization, public node operation,
> and production observability remain future milestones.

## System purpose

SynapseFlow executes a language model whose immutable weight shards are placed on one or more workers. A client submits a generation request to the kernel. The kernel owns the client-visible workflow state and describes work as Crux managed effects. A platform shell executes each effect: authenticates and limits the request, obtains a verified model manifest, creates a route through eligible shard workers, and owns the request session until it completes, fails, or is cancelled. Today `synapseflow-cli` validates the request boundary, invokes the configured generation runtime, and resolves the outcome back into the kernel. Future native and web clients use the same kernel events, effects, and view model.

Workers execute ordered layer groups and transfer activation frames. The final stage returns logits to the node, which applies the requested sampling policy and streams tokens to the client. Every action is traceable to a model version, shard hash, worker identity, and session identifier.

```text
client
  │ Events (request, identity, generation policy)
  ▼
kernel ── state machine, Commands, view model
  │ Effects (authorization, quotas, generation, render)
  ▼
application service ── model acquisition, tokenization, execution, auditing, planning, deadlines, session ownership
  ▼
worker A ── verified shard ── activation frame ──► worker B ──► final worker
  ▲                                                               │ logits
  └──────────── audit events, checkpoints, retry/fallback ◄──────┘
```

## Dependency direction

The workspace is organized around stable contracts, a portable interaction core and concrete shells, not infrastructure choices. Kernel owns the event/effect loop; shells own runtime integration and presentation.

```text
applications (CLI, native or web UI)
        │ drive and resolve
kernel (Crux App, Events, Effects, ViewModel)
        │ resolves initialization, then sends SubmitGeneration
kernel generation effect
        │ composes
application services (generation, planning, policy, sessions)
        │
domain (model manifest, shard, tensor, frame, session state)
        │
ports (backend, transport, shard store, peer directory, audit sink)
        │
adapters (model runtime, remote registry, local cache, QUIC, database, telemetry)
```

`synapseflow-kernel` has no runtime, transport, HTTP, filesystem, or backend dependency. It owns the client workflow model, uses Crux commands to request typed effects, and is tested by driving effects and resolutions in memory. `synapseflow-cli` is the only shell today and creates one kernel instance per independent invocation. It fulfills that initialization effect by creating and retaining the generation service, resolves it, and only then submits generation.

Domain types and port traits must not depend on Tokio, a particular model runtime, QUIC, a database, or an HTTP framework. Application services depend only on domain types and ports. Adapters own external dependencies, allowing deterministic in-memory tests of planning and session behavior. They all remain useful implementation seams, but their former dependency direction is no longer the top-level architecture rule.

## Components

| Component | Responsibility |
|---|---|
| SynapseFlow kernel | Client-facing state machine. Accepts events, emits typed effects, and exposes a presentation-safe workflow view. |
| Client shell | Drives the kernel core, executes its effects, resolves results, and renders the view. `synapseflow-cli` is the sole current implementation. |
| Model registry | Resolves immutable model versions and signed manifests from allowed remote sources. |
| Shard store | Downloads, verifies, caches, opens, evicts, and serves authorized shards. |
| Backend | Tokenizes, executes a whole model or declared layer range, and samples logits. |
| Planner | Chooses a route using model compatibility, worker capability, health, replication, deadline, and policy. |
| Session manager | Owns request state, cancellation, deadlines, checkpoints, retries, result delivery, and cleanup. |
| Transport | Moves validated frames with peer authentication, bounded queues, flow control, and backpressure. |
| Peer directory | Stores enrolled worker identity, capabilities, health, location, and shard availability. |
| Audit and telemetry | Emits privacy-safe audit events, traces, logs, and service metrics. |

## Sharding strategy

For Milestone 3, Loom is the Llama `layer_range_v1` adapter built
on pinned Candle tensor dependencies. It owns GGUF layout/tokenizer handling,
declared-range tensor loading, per-range KV state, and residual/logit
conversion. It accepts only the explicit
`synapseflow-loom-llama-v1` manifest runtime profile. Its contiguous
full-model execution is the sharded-path baseline;
the existing llama.cpp adapter remains the separate Milestone 2 local-inference
profile. Candle/runtime types remain inside the adapter.

Start with layer-wise execution because it has the simplest correctness model. Group adjacent layers into blocks once measurements justify fewer network hops. Tensor parallelism and distributed MoE require synchronization/routing designs with distinct failure and bandwidth properties and are separate architectural decisions.

| Strategy | Appropriate use |
|---|---|
| Layer-wise | Baseline correctness, simple routing, and clear checkpoint boundaries. |
| Block-chunking | Lower hop count and improved compute/network balance. |
| Tensor parallel | Specialized high-bandwidth clusters after collective-communication support exists. |
| MoE distribution | Expert models after routing, scheduling, and fairness controls exist. |

## Security and reliability

Future remote workers use mutually authenticated TLS over QUIC. Milestone 3
workers are independently addressable local loopback workers; they exercise the
production frame codec and bounded transport semantics but do not establish a
remote authenticated transport. Manifests and shards are authenticated by
publisher signature and content hash. Authorization controls who may fetch or
execute a model once the operable-node milestone adds it. The protocol bounds
input before decoding or allocating, propagates deadlines, and treats
cancellation as idempotent.

Replicas, checkpoints, circuit breakers, and a retry budget make failure handling explicit. Hash validation detects corruption; it does not by itself establish trust or prevent activation leakage. Optional TEEs, differential privacy, MPC, and homomorphic encryption require a documented threat model and performance justification before adoption.

## Operational properties

The system exposes request/token latency, activation bytes, compression ratio, queue depth, worker health, retry/fallback count, model-cache state, and error rate. Every request carries trace and session identifiers. Logs and audit events exclude prompt text, raw activations, weights, credentials, and other sensitive payloads unless an explicit, reviewed policy allows them.
