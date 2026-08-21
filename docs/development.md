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

The repository pins Rust 1.89.0 in [`rust-toolchain.toml`](../rust-toolchain.toml), with `rustfmt` and Clippy. It deliberately uses the standard Rust formatting rules, so no `rustfmt.toml` is required. Tier-1 CI runs natively on Windows and Linux; no cross-compilation target is required for local setup. The Tier-1 targets and Rust 1.85.0 MSRV are defined in [ADR 0001](adr/0001-supported-platforms-and-toolchain.md). CI validates both the pinned toolchain and the MSRV.

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

Protocol, backend, cryptographic, authorization, and scheduler changes require review and an ADR when they create a durable trade-off. Releases follow [ADR 0002](adr/0002-release-and-compatibility-policy.md) and maintain a changelog, compatibility statement, SBOM, and migration notes. Model artefacts follow the [model-management policy](model-management.md) and the initial runtime scope in [ADR 0003](adr/0003-initial-model-backend-scope.md).
