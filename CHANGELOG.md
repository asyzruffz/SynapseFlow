# Changelog

All notable changes to SynapseFlow are documented here. The project follows [Semantic Versioning](docs/adr/0002-release-and-compatibility-policy.md) and uses the Keep a Changelog categories below.

## Unreleased (0.2.0-dev)

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
