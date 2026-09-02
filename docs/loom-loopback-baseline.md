# Loom loopback baseline harness

> **Source of truth.** This document defines the loopback baseline, comparison,
> and recovery contract. Change it only through an explicit validation redesign;
> the harness and runtime must conform to it.

This document defines a deterministic acceptance fixture for automated
integration coverage. It does not represent accuracy or performance on a
released model.

## Fixture and execution shape

- Model architecture: Llama, GGUF, `Q5_K_M` matrices, embedded tokenizer
  metadata.
- Layout: two transformer layers, activation width 256, vocabulary size 256.
- Input: token IDs `[11, 12]`, encoded as little-endian `u32`, stream `1`,
  sequence `0`, position start `0`.
- Baseline: one contiguous declared range `[0, 2)` over a generated whole-model
  artifact.
- Sharded route: declared ranges `[0, 1)` and `[1, 2)`, with `loopback-a` as
  the first-range primary, `loopback-b` as the final-range primary, and
  `loopback-c` as the final-range replica.
- Runtime: `synapseflow-loom-llama-v1`, `synapseflow-adapter-loom` workspace
  version `0.2.0-dev`, `candle-core = 0.11.0`, and `candle-nn = 0.11.0`.
- Wire contract: activation-frame protocol version `1`; the rank-2 f32
  boundary includes its required position extension and crosses the bounded
  loopback transport as canonical encoded bytes.

## Comparison method

The test executes the whole-model artifact once, then executes the two declared
range artifacts through separate `LoomBackend` instances. It compares the final
logit frame payloads byte-for-byte. For this deterministic CPU-generated
fixture, the accepted tolerance is exact equality: zero differing f32 elements,
zero absolute error, and zero relative error. A realistic external-model
fixture must declare its own numerical tolerance and comparison record before
being used as an acceptance baseline.

## Recovery method

After the first range produces the sequence-1 activation boundary, the session
manager records its payload-free checkpoint reference. The harness makes
`loopback-b` unavailable before forwarding that boundary, consumes exactly one
retry from the session's budget, and forwards the same validated frame to
`loopback-c`. The replica completes the final range under the original
five-second remaining deadline. The test requires recovered terminal audit
events with retry count `1` and fallback count `1`.

## Automated validation

`loopback_harness_matches_contiguous_loom_and_recovers_the_final_range_from_checkpoint`
in `adapters/layer-range/src/tests.rs` is the executable specification. It validates
the contiguous baseline, production frame forwarding, two-range execution,
checkpoint selection, primary failure, replica execution, exact final-logit
comparison, and payload-free recovered audits.

For generated-fixture transport and timing details, see the
[loopback validation profile](acceptance/loopback-sharding-generated-fixture.md).
The [Windows validation profile](acceptance/loopback-sharding-windows.md)
preserves the distinction between the supplied schema-v1 fixture manifest and
the in-memory schema-v2 declaration used only for Loom measurement.
