# Development

## Quality gate

Every pull request runs the following gate on Windows and a supported Linux target:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo deny check
cargo audit
```

The toolchain is pinned, dependencies are obtained from a reproducible writable cache, and CI is the source of truth for the supported Rust version and platforms.

## Test strategy

| Layer | What it proves |
|---|---|
| Unit | Manifest invariants, shard selection, sampling, session transitions, and error mapping. |
| Contract | Golden manifest/frame bytes remain compatible across supported versions. |
| Integration | Model acquisition, CLI/API generation, loopback sharding, cancellation, timeout, malformed frames, and replica fallback. |
| Property/fuzz | Manifest and frame decoders never panic, over-allocate, or enter invalid state. |
| Performance | Reproducible token latency, activation bytes, compression, throughput, memory, and recovery metrics. |
| Security | Dependency/SBOM/secret scanning; invalid signatures; authentication, authorization, and limit enforcement. |

Every defect in framing, planning, cache use, or model loading begins with a failing regression test. Benchmark records include model and backend version, hardware, input/batch size, seed, protocol settings, and measurement method.

## Code rules

- Use typed `thiserror` errors in libraries; translate errors at application boundaries.
- Do not use `assert!` for user, network, or artifact input.
- Bound input before decoding, decompression, allocation, or buffering.
- Prefer explicit types for identifiers, versions, hashes, units, and deadlines.
- Keep domain and port crates independent of transport/backend/framework details.
- Document public concurrency, timeout, cancellation, and ownership behavior.

## Change management

Protocol, backend, cryptographic, authorization, and scheduler changes require review and an ADR when they create a durable trade-off. Releases maintain a changelog, compatibility statement, SBOM, and migration notes. Model artefacts follow the [model-management policy](model-management.md).
