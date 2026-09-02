# ADR 0002: Release and compatibility policy

**Status:** Accepted

## Context

SynapseFlow has public crate, CLI, API, protocol, and manifest surfaces that evolve at different rates. A pre-1.0 project still needs a predictable compatibility policy so users and workers can upgrade safely.

## Decision

- SynapseFlow follows Semantic Versioning. Development snapshots use a valid prerelease identifier such as `0.1.0-dev` and are not stable releases.
- Before `1.0.0`, a patch release (`0.y.z` → `0.y.z+1`) is backward compatible and limited to fixes, documentation, and compatible additions. A minor release (`0.y.z` → `0.(y+1).0`) may contain a breaking public API, CLI, configuration, or protocol change and must provide migration notes.
- At `1.0.0` and later, public API/CLI/configuration breaking changes require a major version. Deprecations receive a documented replacement and removal release.
- Published workspace crates share the product release version until a later ADR introduces independently versioned crates.
- Protocol envelopes include an explicit protocol version. Additive, defaultable fields are compatible within a version; semantic or decoding changes require a new version. Nodes advertise supported versions and reject unsupported versions safely.
- Model manifests are immutable artifacts identified by schema version, model version, publisher identity, and content hash. A changed model, shard, tokenizer, signature, or semantic manifest field is a new manifest/model version, never an in-place update.
- Every release publishes release notes, compatibility/migration notes when required, an SBOM, and evidence from the Tier-1 quality gate.

## Consequences

- Early users can rely on patch-release stability while accepting that minor pre-1.0 releases may require migration.
- Protocol and manifest evolution needs schema compatibility tests before release.
- Release automation must package all workspace crates consistently and retain the exact source revision, lockfile, toolchain, and dependency evidence.

## Superseding conditions

Supersede this ADR when the project begins independently versioning crates, releases `1.0.0`, or adopts a compatibility window different from the defined protocol policy.
