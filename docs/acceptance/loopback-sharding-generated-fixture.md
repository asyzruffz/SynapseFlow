# Loopback-sharding generated-fixture validation profile

> **Source of truth.** This profile defines generated-fixture loopback
> validation. Change it only through an explicit validation redesign; the
> harness must conform to its stated boundaries.

This profile defines validation for the hermetic Loom loopback harness. It
captures correctness and overhead for a generated fixture, not performance for
a released model or a Tier-1 acceptance profile.

| Field | Value |
|---|---|
| Platform | Microsoft Windows 11 Pro 10.0.26100, 64-bit |
| Processor | 12th Gen Intel Core i7-12700; 12 cores, 20 logical processors |
| Rust toolchain | `rustc 1.89.0` |
| Execution profile | Cargo `test` profile, unoptimized with debug information; no CPU affinity or thermal policy was set |
| Runtime | `synapseflow-loom-llama-v1`; `synapseflow-adapter-loom 0.2.0-dev`; `candle-core = 0.11.0`; `candle-nn = 0.11.0` |
| Protocol | Activation-frame protocol v1; canonical `none` compression; loopback queue limit 4 |
| Fixture | Hermetic generated Llama/GGUF structure: 2 layers, width 256, vocabulary 256, `Q5_K_M` matrices, embedded tokenizer metadata. Constant generated tensors have no random seed; temporary artifacts are deleted after each run. |
| Manifest identity | Test-only schema-v2 manifest reference and declarations in `adapters/layer-range/src/tests.rs`; no signed external manifest or persistent artifact hash is claimed. |
| Request | Two token IDs (`[11, 12]`), position 0, one stream, one final-range primary failure after the sequence-1 checkpoint, one allowed retry and replica fallback. |
| Measurement method | Five warm, focused invocations of `cargo test -p synapseflow-adapter-loom --lib tests::loopback_harness_matches_contiguous_loom_and_recovers_the_final_range_from_checkpoint --locked -- --exact --nocapture`; the harness uses `Instant` around contiguous baseline, each range, recovery, and its complete setup/execution path. |

## Reference safe aggregates

| Metric | Minimum | Median | Maximum |
|---|---:|---:|---:|
| Contiguous-baseline latency | 15.083 ms | 16.672 ms | 28.758 ms |
| First-range latency | 6.769 ms | 7.458 ms | 8.996 ms |
| Final-range replica latency | 6.867 ms | 7.515 ms | 8.629 ms |
| Checkpoint-to-replica recovery latency | 7.120 ms | 7.711 ms | 8.846 ms |
| Harness end-to-end latency | 93.693 ms | 97.754 ms | 109.405 ms |

The median harness rate is 10.23 two-token fixture requests per
second, or 20.46 input-token IDs per second. This is not generated-token
throughput: the harness stops at final logits and deliberately does not add a
sampler or streaming API.

| Resource/transport metric | Reference value |
|---|---:|
| Initial canonical input-frame bytes | 226 bytes |
| Intermediate activation payload bytes | 2,048 bytes |
| Canonical intermediate boundary-frame bytes | 2,271 bytes |
| Maximum observed destination queue depth | 1 frame |
| Retry count / fallback count | 1 / 1 |
| Compression ratio | 1.00 |
| Compression CPU cost | Not applicable: protocol v1 admits only `none` and selects no compressor. A compressed/uncompressed comparison requires a separately versioned protocol capability. |

The metric output deliberately excludes token values, activation contents,
weights, cache paths, and runtime diagnostics.

## Scope boundary

This generated-fixture profile does not substitute for a provisioned external
GGUF fixture with an immutable signed manifest/artifact hash, release-mode run,
process working-set sampling, Tier-1 platform validation, or a versioned
compression comparison. The provisioned profile samples process working set at
a documented interval and reports p50/p95 latency and peak memory for each
Tier-1 platform.
