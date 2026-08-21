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

The [CI workflow](../.github/workflows/ci.yml) runs the formatting, build, strict Clippy, and test commands on native Windows and Linux runners. It also validates the declared MSRV on Linux, enforces the `cargo deny` dependency and licence policy, audits Rust advisories, scans the full repository history for verified secrets, and publishes an SPDX SBOM workflow artifact. Cargo and Rustup homes are allocated beneath each runner's temporary directory; cache restoration is an optimization only and a cache miss performs a locked build from scratch.

Repository branch protection must require the `Quality (ubuntu-latest)`, `Quality (windows-latest)`, `MSRV (Rust 1.87.0)`, `Dependency policy and audit`, `Secret scan`, and `SBOM` checks before merging into a protected release branch.

The repository pins Rust 1.89.0 in [`rust-toolchain.toml`](../rust-toolchain.toml), with `rustfmt` and Clippy. It deliberately uses the standard Rust formatting rules, so no `rustfmt.toml` is required. Tier-1 CI runs natively on Windows and Linux; no cross-compilation target is required for local setup. The Tier-1 targets and Rust 1.87.0 MSRV are defined in [ADR 0001](adr/0001-supported-platforms-and-toolchain.md). CI validates both the pinned toolchain and the MSRV.

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

## Test fixtures and determinism

Tests are hermetic: they must not require a downloaded model, network access, credentials, a populated Cargo home, or developer-local state. Structural model-loader tests create their own empty temporary files with `.safetensors`, `config.json`, and `tokenizer.json` names; these are generated during the test, contain no third-party model data, and are deleted afterward. CLI smoke tests exercise `--help` only, so they require no model artifact.

When a test requires generated data, use a named fixed seed and record it in the test or fixture metadata. Versioned fixtures must be small, deterministic, licence-cleared, and accompanied by provenance, generator version, content hash, and expected behavior. Never version real model weights, credentials, prompts containing private data, or activation dumps. Tests that need a real verified model belong in an explicitly provisioned integration environment, not the default quality gate.

## Code rules

- Use typed `thiserror` errors in libraries; translate errors at application boundaries.
- Do not use `assert!` for user, network, or artifact input.
- Bound input before decoding, decompression, allocation, or buffering.
- Prefer explicit types for identifiers, versions, hashes, units, and deadlines.
- Keep domain and port crates independent of transport/backend/framework details.
- Document public concurrency, timeout, cancellation, and ownership behavior.

## Error and input boundary

Every public library operation returns that crate's typed error and result alias. Errors identify the operation category—such as source discovery, validation, backend availability, initialization, generation, transport, or policy—without leaking secrets or raw model/prompt data. Binaries and application services may add `anyhow` context when rendering a user-facing diagnostic, but they do not expose `anyhow` as a library contract.

User, peer, and artifact input is fallible data. Validate it before work begins; return a stable error rather than panicking. Frame and artifact handlers validate size and structure before decoding, decompression, allocation, or buffering. Callers must handle output and filesystem errors explicitly; an output failure must not become an `unwrap()` panic.

## Change management

Protocol, backend, cryptographic, authorization, and scheduler changes require review and an ADR when they create a durable trade-off. Releases follow [ADR 0002](adr/0002-release-and-compatibility-policy.md) and maintain a changelog, compatibility statement, SBOM, and migration notes. Dependencies follow the [dependency-management policy](dependency-management.md). Model artefacts follow the [model-management policy](model-management.md) and the initial runtime scope in [ADR 0003](adr/0003-initial-model-backend-scope.md).
