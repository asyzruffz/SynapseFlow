# Changelog

All notable changes to SynapseFlow are documented here. The project follows [Semantic Versioning](docs/adr/0002-release-and-compatibility-policy.md) and uses the Keep a Changelog categories below.

## 0.2.0-dev — 2026-08-25

### Added

- Schema-v2 signed shard manifests, `layer_range_v1` planning/session contracts, and activation-frame protocol v1 with canonical manifest and frame test vectors.
- Loom, the Llama range-execution adapter, with declared-range GGUF loading, per-range KV ownership, contiguous baseline execution, and bounded local loopback workers.
- Deterministic two-range comparison to the Loom baseline, typed corruption/cancellation/timeout handling, and checkpointed replica recovery.
- Windows acceptance evidence for the provisioned TinyLlama fixture, covering activation bytes, latency, process working set, queue depth, and recovery timing.

### Changed

- The schema-v2 runtime profile is `synapseflow-loom-llama-v1`; the former `llama-layer-range-v1` profile is rejected rather than silently migrated.
- The implementation-gap record now distinguishes delivered local loopback sharding from future remote-worker and operable-node work.

### Deprecated

### Removed

### Fixed

### Security

- Activation frames validate declared bounds, hashes, dtypes, ordering, deadlines, and compression before allocation or execution. Safe logs and acceptance records exclude prompts, raw activations, weights, credentials, and cache paths.

### Compatibility and scope

- This snapshot preserves the Milestone 2 verified-local runtime as a separate profile; its llama.cpp acceptance record is not the Loom sharding baseline.
- Milestone 3 acceptance is Windows-scoped. It does not claim remote-worker operation, QUIC, authentication/authorization, public distributed APIs, or cross-platform release validation.
- No user-facing CLI or loopback API migration is required.

## Unreleased (0.1.0-dev)

### Added

- Verified local GGUF/Llama inference through the `synapseflow run` CLI and loopback-only `/v1/generate` and `/v1/generate/stream` API.
- Signed-manifest verification, content-addressed local cache, explicit fixture provisioning, stable errors, seeded generation, deadlines, and provisioned acceptance evidence.

### Changed

- Advanced the shared workspace development version to `0.2.0-dev` for the next roadmap milestone.

### Deprecated

### Removed

- Obsolete pre-milestone `core`, `coord`, `inference`, `runtime`, `network`, `security`, `utils`, and `incentive` crates, including the retired Candle/safetensors path.

### Fixed

### Security

## Release entry requirements

Each release entry includes the release date, version, user-visible changes, upgrade/migration instructions for breaking changes, compatibility implications, and links to security advisories where appropriate. Do not publish an empty release entry; include a concise statement when a release only updates dependencies or build artifacts.
