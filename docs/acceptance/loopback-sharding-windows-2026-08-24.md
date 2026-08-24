# Loopback-sharding Windows measurement — 2026-08-24

This record captures the provisioned TinyLlama GGUF run for Milestone 3 Step
10 on Windows. It contains no prompt text, token values, activation values,
weights, cache paths, or credentials.

| Field | Value |
|---|---|
| Platform | Microsoft Windows 11 Pro 10.0.26100, 64-bit |
| Processor | 12th Gen Intel Core i7-12700; 12 cores, 20 logical processors |
| Rust toolchain | `rustc 1.89.0 (29483883e 2025-08-04)` |
| Execution profile | Cargo `test --release` profile, optimized; no CPU affinity or thermal policy was set |
| Runtime | `synapseflow-loom-llama-v1`; `synapseflow-adapter-loom 0.2.0-dev`; `candle-core = 0.11.0`; `candle-nn = 0.11.0` |
| Protocol | Activation-frame protocol v1; canonical `none` compression; loopback queue limit 4 |
| Supplied manifest provenance | Signed Milestone 2 schema-v1 fixture reference `registry://fixtures/synapseflow-verified-local-tinyllama-q5km-v1@sha256:8c9c17c57eed25a84f908c6a72a24d90d37546713d4480aba2e3dadb5d0e29e5` |
| Loom declaration | In-memory schema-v2, two-range `[0, 11)` → `[11, 22)` declaration derived only for this measurement. It is **not** a signed Milestone 3 manifest. |
| Artifact | `tinyllama-1.1b-chat-v0.3.Q5_K_M.gguf`; 782,052,992 bytes; SHA-256 `7c255febbf29c97b5d6f57cdf62db2f2bc95c0e541dc72c0ca29786ca0fa5eed` |
| GGUF layout | 22 layers, activation width 2,048, embedded-tokenizer vocabulary 32,003 |
| Request | Two fixed token IDs at position 0; primary final-range worker becomes unavailable after the sequence-1 checkpoint; retry budget 1; remaining deadline 300 seconds |

## Measurement method

The ignored provisioned test
`provisioned_tinyllama_loopback_measurement_matches_contiguous_loom_and_recovers`
loads the full artifact once as the contiguous Loom baseline, then executes the
two declared ranges through canonical loopback frames. It compares final logits
byte-for-byte, makes the primary final-range worker unavailable, and replays the
checkpoint boundary to the declared replica.

Five focused warm release invocations supplied the timing samples. p95 below
uses the conservative nearest-rank value from those five observations. A
separate passing release run sampled the combined Cargo test process tree
working set every 25 ms.

| Metric | p50 | p95 |
|---|---:|---:|
| Contiguous Loom baseline | 721 ms | 728 ms |
| First range | 446 ms | 450 ms |
| Final-range replica | 451 ms | 456 ms |
| Checkpoint-to-replica recovery | 451 ms | 456 ms |
| Sharded end-to-end with induced recovery | 897 ms | 907 ms |

| Resource / transport metric | Observed value |
|---|---:|
| Recovered completed requests per second | 1.115 requests/s at p50 end-to-end latency |
| Input token-ID rate through the recovered path | 2.230 IDs/s at p50 end-to-end latency |
| Peak Cargo-test process-tree working set | 803,913,728 bytes (766.7 MiB), sampled every 25 ms |
| Intermediate activation payload | 16,384 bytes |
| Canonical intermediate boundary frame | 16,640 bytes |
| Maximum observed destination queue depth | 1 frame |
| Retry count / fallback count | 1 / 1 |
| Compression ratio | 1.00 |
| Compression CPU cost | Not applicable: protocol v1 admits only `none` and selects no compressor. |

The rates are not generated-token throughput: the harness intentionally stops
at final logits and does not add sampling or streaming work.

## Compatibility result

The artifact hash and size matched the supplied immutable fixture metadata.
Loom accepted the artifact after handling two valid GGUF details: its aggregate
`Q5_K_M` profile includes `Q6K` projection matrices, and its vocabulary is
declared by the embedded tokenizer array rather than an optional
`llama.vocab_size` metadata value. The contiguous and sharded outputs matched
exactly; the replica completed with exactly one retry and one fallback.

## Scope and remaining evidence

The project owner explicitly deferred Linux measurement for this milestone
step. This record is Windows-only and must not be used to claim cross-platform
performance evidence. It lacks a signed schema-v2 Loom fixture manifest; an
immutable signed schema-v2 declaration is required before making a production
performance claim. Compression comparison remains deferred until a separately
versioned compression capability is accepted.
