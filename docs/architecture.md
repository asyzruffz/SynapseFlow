# Architecture

> **Source of truth.** This document defines SynapseFlow's intended
> architecture. Change it only through an explicit architecture redesign; the
> implementation must converge on this design rather than redefining it.

SynapseFlow provides verified local inference and bounded two-worker loopback
sharding through schema-v2 manifests, activation frames, planning/session
contracts, Loom layer-range execution, and replica recovery. QUIC, remote
worker enrolment, authentication/authorization, public node operation, and
production observability remain planned capabilities.

## System purpose

SynapseFlow executes immutable model artifacts through a single
application-owned generation lifecycle. A client surface drives the kernel's
workflow state and resolves its typed effects. The composition root validates
the outer request boundary, authenticates and limits the caller where required,
and invokes an application use case. The application use case resolves the
verified manifest, selects the declared execution profile, owns the session,
and returns one safe outcome to the kernel.

Workers execute declared ranges and transfer activation frames. The final range
returns logits to the application layer, which applies the public generation
policy and returns tokens to the client. Every action is attributable to an
immutable model reference, shard identity, worker identity, and session ID.

```text
client surface
  │ events and presentation
  ▼
kernel ── workflow state, effects, view model
  │ typed effect requests
  ▼
composition root ── CLI, node API, native or web shell
  │ composes adapters and invokes one use case
  ▼
application ── admission, manifest resolution, profile selection, planning,
               session ownership, retries, final sampling, audit
  │
  ├── control plane ── session, deadline, cancellation, checkpoint selection
  │
  └── data plane ── worker A ── activation frame ──► worker B ──► logits
```

## Dependency direction

The workspace is organized around stable contracts, a portable interaction core,
and concrete composition roots rather than infrastructure choices. The kernel
owns event/effect workflow state; application owns execution decisions; shells
own presentation and adapter wiring.

```text
client surfaces (CLI, node API, native or web UI)
        │ drive and resolve
kernel (Crux App, events, effects, view model)
        │ requests typed execution
composition roots
        │ wire adapters; invoke use cases
application services ─────────────► domain contracts
        │                            manifests, frames, plans, sessions, errors
        └──────────────────────────► ports
                                     registry, artifacts, execution, transport,
                                     peer directory, audit, clock, identity
adapters ──────────────────────────► ports and domain
  model runtimes, cache/registry, loopback/QUIC, persistence, telemetry
```

`synapseflow-kernel` has no runtime, transport, HTTP, filesystem, registry, or
backend dependency. It owns client workflow state and requests typed effects.
Composition roots create one kernel instance per client workflow, resolve its
effects, and never embed execution policy in presentation code.

Domain types and port traits must not depend on Tokio, a model runtime, QUIC, a
database, or an HTTP framework. Application services depend only on domain and
ports. Adapters implement ports and own infrastructure dependencies. Production
adapters must not depend on application services or client shells; composition
roots are the only place that may depend on both use cases and concrete
adapters.

## Components

| Component | Responsibility |
|---|---|
| SynapseFlow kernel | Client-facing state machine. Accepts events, emits typed effects, and exposes a presentation-safe workflow view. |
| Client surface | Drives the kernel, executes its effects through a composition root, and renders the view. CLI, node API, and future UI shells use the same lifecycle. |
| Generation orchestrator | Resolves a verified manifest, selects local or sharded execution from its declared profile, owns admission, route planning, session lifecycle, retry, final sampling, and cleanup. |
| Model registry | Resolves immutable model versions and signed manifests from allowed remote sources. |
| Shard store | Downloads, verifies, caches, opens, evicts, and serves authorized shards. |
| Execution backend | Tokenizes and executes a whole model or declared layer range. It returns bounded execution output and never owns public request policy or session state. |
| Planner | Chooses a route using model compatibility, worker capability, health, replication, deadline, and policy. |
| Session manager | The application-owned authority for request state, cancellation, deadlines, checkpoint references, retries, result delivery, and cleanup. Checkpoint payloads remain behind bounded storage/runtime boundaries. |
| Transport | Moves validated frames with peer authentication, bounded queues, flow control, and backpressure. |
| Peer directory | Stores enrolled worker identity, capabilities, health, location, and shard availability. |
| Identity and policy | Authenticates callers/workers and applies authorization, quota, and model-access policy before execution begins. |
| Audit and telemetry | Emits privacy-safe audit events, traces, logs, and service metrics without payload content. |

## Execution ownership

The verified-local and sharded paths are two profiles of one generation
orchestrator, not separate shell workflows. The orchestrator selects a path
only from a verified manifest's declared compatibility and strategy profile.
Client input cannot choose a backend, worker, cache entry, or transport.

The orchestrator owns the following decisions exactly once: admission,
manifest resolution, artifact acquisition, route selection, session creation,
deadline propagation, cancellation, checkpoint-reference selection, retry and
replica selection, final-logit sampling, terminal auditing, and cleanup. A
backend owns only validated execution for its declared capability. A transport
owns only bounded delivery of canonical frame bytes. A client surface owns only
request presentation and effect resolution.

## Control and data planes

The control plane consists of identity, authorization, quotas, manifest and
route selection, session transitions, cancellation, deadline budgets,
checkpoint references, retry decisions, auditing, and terminal cleanup. It is
owned by the application layer and must not be reconstructed independently by
workers.

The data plane consists of canonical activation-frame bytes, ACK/NACK, and
bounded transport flow control between declared workers. It carries no
credentials, authorization decisions, raw runtime state, or unbounded replay
history. Control messages carried by the frame protocol request transport-level
actions; they do not authorize work or alter application-owned policy.

## Sharding strategy

Loom is the Llama `layer_range_v1` adapter built
on pinned Candle tensor dependencies. It owns GGUF layout/tokenizer handling,
declared-range tensor loading, per-range KV state, and residual/logit
conversion. It accepts only the explicit
`synapseflow-loom-llama-v1` manifest runtime profile. Its contiguous
full-model execution is the sharded-path baseline;
the existing llama.cpp adapter remains the separate verified-local-inference
profile. Candle/runtime types remain inside the adapter.

Start with layer-wise execution because it has the simplest correctness model. Group adjacent layers into blocks once measurements justify fewer network hops. Tensor parallelism and distributed MoE require synchronization/routing designs with distinct failure and bandwidth properties and are separate architectural decisions.

| Strategy | Appropriate use |
|---|---|
| Layer-wise | Baseline correctness, simple routing, and clear checkpoint boundaries. |
| Block-chunking | Lower hop count and improved compute/network balance. |
| Tensor parallel | Specialized high-bandwidth clusters after collective-communication support exists. |
| MoE distribution | Expert models after routing, scheduling, and fairness controls exist. |

## Security and reliability

Future remote workers use mutually authenticated TLS over QUIC. Loopback
workers are independently addressable local workers; they exercise the
production frame codec and bounded transport semantics but do not establish a
remote authenticated transport. Manifests and shards are authenticated by
publisher signature and content hash. Authorization controls who may fetch or
execute a model once the operable-node capability is introduced. The protocol bounds
input before decoding or allocating, propagates deadlines, and treats
cancellation as idempotent.

Replicas, checkpoints, circuit breakers, and a retry budget make failure handling explicit. Hash validation detects corruption; it does not by itself establish trust or prevent activation leakage. Optional TEEs, differential privacy, MPC, and homomorphic encryption require a documented threat model and performance justification before adoption.

## Operational properties

The system exposes request/token latency, activation bytes, compression ratio, queue depth, worker health, retry/fallback count, model-cache state, and error rate. Every request carries trace and session identifiers. Logs and audit events exclude prompt text, raw activations, weights, credentials, and other sensitive payloads unless an explicit, reviewed policy allows them.
