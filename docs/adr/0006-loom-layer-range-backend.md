# ADR 0006: Loom layer-range backend

**Status:** Accepted  
**Supersedes:** the native-bridge and llama.cpp-baseline portions of [ADR
0005](0005-loopback-layer-range-execution.md) for loopback sharding.

## Context

[ADR 0005](0005-loopback-layer-range-execution.md) correctly identified that
the public `llama-cpp-2` API cannot execute a declared contiguous layer range or
accept a residual activation boundary. Its selected remedy—a private C++ bridge
over llama.cpp internals—would make a model-specific private ABI and a vendored
native source revision part of the loopback-sharding critical path.

Loopback sharding instead needs a backend whose layer boundaries, tensor ownership,
and per-range KV state are directly observable and testable. The project accepts
an independent pure-Rust numerical baseline. The sharded path
will be compared to an unsharded execution of the *same pinned pure-Rust
implementation*, not to the verified-local-inference llama.cpp output. This proves partition
correctness; it does not alone establish equivalence to llama.cpp.

## Decision

- Add **Loom**, a Llama-specific, CPU-first, pure-Rust runtime inside
  `synapseflow-adapter-layer-range`. It implements only `layer_range_v1` behind
  the existing strategy-neutral `ShardExecutionBackend` port.
- Use `candle-core =0.11.0` and `candle-nn =0.11.0` for bounded tensor,
  quantized matrix, and CPU operations. Adapt the necessary Llama/GGUF
  implementation structure from Candle 0.11.0 into repository-owned focused
  modules; retain the required MIT/Apache-2.0 attribution and licence notices.
  Do not depend on Candle's whole-model `ModelWeights` facade because it
  intentionally hides layers and always executes the entire stack.
- Change the schema-v2 execution `runtime_profile` to the explicit
  `synapseflow-loom-llama-v1` value. This preserves the schema shape while
  requiring a newly signed immutable manifest and canonical-vector migration;
  the former `llama-layer-range-v1` profile is rejected by the pure-Rust adapter.
- The adapter owns its GGUF layout inspector and loads only the tensors needed
  by its declared role: embedding/prefix plus the first range, an intermediate
  range, or a final range plus output norm/head. The immutable verified manifest
  remains the sole source of artifact identity, range, architecture,
  quantization, and runtime-profile compatibility.
- Each worker owns KV entries only for its loaded layers. A residual boundary,
  token IDs, positions, and final logits cross workers only through the bounded
  SynapseFlow activation-frame codec. Runtime state, Candle tensors, and KV
  storage never cross a port or wire boundary.
- Provide a contiguous full-model execution mode using the same pure-Rust
  loader, layer functions, tokenizer behavior, sampling policy, and pinned
  dependency graph as the sharded route. Record its token IDs and logits as the
  loopback baseline, then require the two-range route to match it under a
  predeclared tolerance.
- Retain a separately provisioned llama.cpp fixture vector only as a
  non-authoritative compatibility smoke test. A mismatch is investigated and
  recorded; it does not relax or replace the pure-Rust sharding acceptance
  comparison.

## Module and validation boundary

The adapter will separate GGUF metadata/layout inspection, verified-range
loading, tokenizer handling, quantized layer operations, KV ownership,
execution, output conversion, and backend composition. Its public surface
contains only intended adapter types; Candle types remain implementation
details. Domain, ports, and application remain runtime-independent.

The implementation must reject unsupported GGUF metadata, non-Llama
architecture, non-`Q5_K_M` quantization, missing/incorrect declared tensors,
out-of-range layers, non-finite boundary/logit values, exceeded memory bounds,
cancellation, and elapsed deadlines with stable safe domain errors. It must
test range isolation with licence-cleared generated fixtures before an external
GGUF fixture is used.

## Consequences

- Layer partitioning, activation boundaries, and KV ownership become
  repository-owned Rust code rather than undocumented llama.cpp internals.
- The `llama-cpp-2` adapter and its validation profile remain valid
  for verified local inference. They are not the loopback-sharding
  baseline, and this ADR does not silently replace the delivered local CLI/API
  runtime.
- Candle is a direct adapter dependency. Its exact versions, locked checksums,
  selected CPU features, copied-source provenance, licences, and Tier-1
  performance/resource evidence require supply-chain and runtime/model review.
- The initial pure-Rust implementation remains Llama-specific. Another model
  family or sharding strategy receives its own compatibility profile, loader,
  execution modules, fixtures, and ADR.

## Alternatives considered

### Private llama.cpp C++ bridge

Rejected for loopback sharding. It could reuse the current numerical baseline, but
would bind loopback sharding to an unstable internal graph API and native source
maintenance burden.

### Use Candle's public quantized Llama model unchanged

Rejected. Its public facade loads and runs the complete model, so it cannot
prove range isolation or give one worker ownership of only its KV state.

### Implement raw tensor kernels without a Rust tensor foundation

Rejected. Reimplementing quantized matrix operations and CPU kernels would
increase correctness and performance risk without improving the sharding
contract.

## Superseding conditions

Supersede this ADR if a reviewed pure-Rust runtime no longer satisfies the
declared compatibility/performance bounds, if a future model family needs a
different execution design, or if a public maintained range API satisfies the
same portable frame and ownership contracts with lower verified risk.
