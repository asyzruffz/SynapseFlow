# ADR 0005: Loopback layer-range execution backend

**Status:** Proposed  
**Date:** 2026-08-24

## Context

[ADR 0003](0003-initial-model-backend-scope.md) and [ADR
0004](0004-verified-local-inference-contract.md) intentionally establish one
verified, whole-model GGUF/Llama inference tuple. Roadmap Milestone 3 requires
a different capability: two local workers must execute ordered model shards,
exchange validated activation frames, and recover from an induced worker failure
within bounded retry/deadline policy.

The current `llama-cpp-2 =0.1.154` adapter wraps llama.cpp's public whole-model
load/context/decode interface. It neither accepts an intermediate residual
activation as decode input nor restricts execution to a declared range of model
layers. llama.cpp's model graph construction is model-specific internal code,
not a suitable stable public layer-range API. Its public per-sequence state API
can serialize a KV/cache state, but that opaque native representation is tied to
the pinned runtime and must not become a SynapseFlow frame or checkpoint format.

GGUF file splitting and llama.cpp RPC are also not substitutes: split GGUF files
are loaded together as one model, while llama.cpp RPC owns its own runtime/tensor
transport rather than SynapseFlow's versioned activation-frame protocol.

## Decision

Milestone 3 introduces a dedicated, Llama-specific native execution adapter for
the first sharding capability, `layer_range_v1`. It is the first implementation
of a strategy-neutral sharding port; it does not define the shape of future
tensor-parallel, block-chunk, or MoE execution.

### Native backend boundary

- Build a small, repository-owned native extension against one reviewed and
  pinned llama.cpp source revision. It may use the model-specific internal graph
  builder needed to evaluate a contiguous range, but exposes only a narrow,
  versioned C ABI owned by SynapseFlow.
- Keep the exact upstream revision, local patch hash, build options, licences,
  supported architecture, quantization, and platform evidence in the adapter's
  compatibility record. Updating any of them requires a compatibility review,
  reference-output comparison, and a new record.
- Keep the native C ABI private to the adapter. Domain, ports, and application
  types contain no C++, ggml, llama.cpp, runtime pointer, or native-state type.
- Retain `llama-cpp-2 =0.1.154` as the Milestone 2 whole-model baseline. The
  layer-range adapter is evaluated against that baseline; it is not a silent
  replacement for the delivered local workflow.

### Layer-range execution contract

- `layer_range_v1` is the only sharding strategy accepted in Milestone 3.
  Planning and worker capabilities identify the strategy/version explicitly.
- A Llama layout inspector reads immutable verified GGUF metadata and maps the
  supported architecture's embedding/prefix, `blk.<N>.*` block interval, and
  output norm/head into ordered execution segments. It validates complete,
  non-overlapping coverage before loading a shard.
- The first worker consumes token IDs and positions, owns embedding/prefix plus
  its declared block interval, and emits an `f32` residual activation frame.
  The final worker consumes that residual plus positions, owns its declared
  block interval and output suffix, and emits `f32` logits. Each worker owns
  KV state only for the layers it executes.
- Activation and logit frames use SynapseFlow's versioned, bounded, hashed
  binary frame codec. Native state is never a wire payload, audit event, log,
  or committed test artifact.

### Checkpoints, cancellation, and replica recovery

- A checkpoint occurs after a validated stage boundary for each accepted prompt
  or decode-token position. It consists of the bounded token transcript and the
  bounded, in-memory-only validated activation-frame history needed to replay a
  downstream stage. It is not a serialized native KV snapshot.
- A replacement worker rebuilds its local KV state by deterministic replay from
  the most recent retained boundary history. The session manager accounts for
  replay time and memory against the same remaining deadline and retry budget.
  If either bound is exceeded, recovery fails with a stable safe error.
- Checkpoint data is owned by the session manager, is unavailable after the
  session reaches a terminal state, and is cleared during terminal cleanup. It
  is never written to source control, normal logs, audit records, or a cache.
- Cancellation is idempotent. It stops queued work, signals active workers,
  prevents further retry/replay, and completes terminal cleanup.

### Correctness boundary

- The provisioned fixture's whole-model `llama-cpp-2` run is the baseline.
  The two-worker run must reproduce the fixed token-ID/text stream exactly.
- The adapter also records and compares per-token logits using a declared
  maximum absolute and relative tolerance. The tolerance is selected before
  acceptance from reproducible reference measurements; it must not be relaxed
  to accept a failing candidate.
- Baseline and sharded comparisons record manifest hash, shard plan, native and
  Rust adapter versions, protocol/codec version, platform, CPU architecture,
  policy, seed, context/input shape, and measurement method.

### Extensibility boundary

- The generic plan/port/frame/session contracts share only immutable identity,
  bounded work, frame I/O, deadlines/cancellation, checkpoint ownership, and
  safe outcomes.
- A later sharding strategy receives a distinct identifier/version, ADR,
  manifest requirements, capability validation, adapter, and acceptance suite.
  It may introduce strategy-specific operations such as collectives or expert
  routing; those requirements are not inserted into `layer_range_v1`.
- A later model architecture similarly receives its own layout inspector,
  executor, compatibility profile, fixture, and reference evidence. Llama
  tensor names or KV rules do not enter generic contracts.

## Consequences

- Milestone 3 gains real two-stage activation transfer rather than a simulated
  pair of whole-model invocations.
- The project owns a small native integration surface and must maintain it
  against a pinned upstream revision. This raises the review and release burden
  but keeps the distributed protocol independent of a third-party RPC runtime.
- Recovery is correct and bounded for the loopback fixture, but replay can add
  latency and memory proportional to retained context. Measurements determine
  whether later block-chunking is justified.
- The generic sharding architecture stays open to additional architectures and
  strategies without prematurely designing their incompatible execution models.

## Alternatives considered

### Keep the existing whole-model `llama-cpp-2` adapter

Rejected. Its public model/context interface cannot execute a layer interval or
accept a residual activation boundary, so it cannot meet Milestone 3's
two-shard-equivalence criterion.

### Use llama.cpp RPC or its built-in split modes

Rejected. They do not exercise SynapseFlow's frame codec, control semantics,
checkpoint ownership, or transport port, and therefore do not provide the
required loopback-sharding evidence.

### Treat split GGUF files as executable layer shards

Rejected. GGUF file splitting partitions serialized tensors for loading; it does
not establish independently executable model stages or activation boundaries.

### Serialize and send native KV snapshots to a replica

Rejected for this milestone. Native state is runtime-internal, large, opaque,
and unsuitable as a stable interoperable frame contract. Bounded deterministic
replay gives the loopback baseline an inspectable recovery model.

### Implement tensor parallelism, MoE routing, or multiple architectures now

Rejected. Each has different synchronization, boundary, scheduling, and failure
properties. Adding them would dilute the layer-wise correctness baseline.

## Superseding conditions

Supersede this ADR if llama.cpp provides a maintained public layer-range ABI that
meets SynapseFlow's frame, correctness, and recovery contracts; if a reviewed
runtime provides an equivalent stable boundary; or if measurements justify a
different sharding strategy for the supported compatibility profile.
