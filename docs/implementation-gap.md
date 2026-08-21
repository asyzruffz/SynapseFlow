# Implementation gap

This temporary migration tracker compares the documented design with the source tree. It was refreshed on 2026-08-21 after the Foundation milestone. Remove a row when its design contract, implementation, and automated evidence are complete; delete this document when no rows remain.

The Foundation baseline—toolchain, dependency policy, governance, CI, hermetic smoke tests, onboarding, operational runbooks, and clean-clone verification—is complete and is intentionally not repeated here. Its evidence is recorded in [the progress tracker](../PROGRESS.md).

## Remaining gaps

| Design area | Current code boundary | Required migration outcome |
|---|---|---|
| Model acquisition and verification | `ModelSource` accepts only a local path. `ModelLoader` discovers local safetensors, `config.json`, and `tokenizer.json`; it has no manifest, remote source, signature/hash validation, provenance, or cache. | Implement manifest-driven local and remote acquisition, publisher and content verification, compatibility checks, and a content-addressed cache as specified in [model management](model-management.md) and [the protocol](protocol.md). |
| Initial local inference | The available Candle Llama path is CPU/safetensors based and uses argmax generation. The advertised temperature and top-p fields are unused, and LlamaCpp returns `BackendUnavailable`. | Implement the ADR 0003 GGUF/Llama, llama.cpp-compatible adapter, deterministic seed mode, tokenizer/context behavior, and advertised sampling policy, with a verified fixture and reference-output tests. |
| Domain contracts and ports | There are no public, versioned manifest, shard index, execution plan, session, backend-port, transport-port, shard-store, peer-directory, or audit-sink contracts. The existing shard-index and session-manager modules are private placeholders. | Introduce framework-independent domain types and port traits before adding transport, planning, or scheduling infrastructure, following the dependency direction in [architecture](architecture.md). |
| Shard execution | The runtime executor and kernel modules contain no executable subgraph implementation. The incomplete shard type is not exported from core. | Implement and test deterministic layer-wise subgraph execution, checkpoint boundaries, and a loopback two-shard baseline. |
| Transport and discovery | The only frame type is a private serde-oriented `OutboundFrame`; encoder and decoder modules contain no implementation. QUIC transport, authentication, bounded queues, backpressure, retries, and discovery have no implementation. | Implement the versioned bounded protocol codec and loopback transport semantics before introducing QUIC. Add mutual TLS, enrollment, health, and remote-worker behavior only in the relevant roadmap milestones. |
| Node, API, security, storage, and incentives | The API, coordinator, security, storage, and incentive surfaces are module skeletons without operable endpoints, planning, sessions, authorization, manifest trust, persistence, or audit behavior. | Add only the components required by the roadmap, starting with an operable local node/API and verified manifest trust. |
| Milestone-level test coverage | The Foundation test baseline covers local model-file discovery, unsupported backend selection, and CLI help without model artifacts. It does not yet prove a verified model token stream, contracts, distributed execution, malformed frame handling, cancellation, recovery, fuzzing, performance, or security behavior. | Add the unit, contract, integration, property/fuzz, performance, and security evidence defined in [development](development.md) as each feature is delivered. |

## Evidence basis

- Foundation completion and clean-clone verification are recorded in [the progress tracker](../PROGRESS.md).
- The local loader’s artifact discovery and unsupported-backend behavior have hermetic automated tests; the CLI `--help` smoke test requires no model artifact or developer state.
- The CI workflow runs the required formatting, locked build, strict Clippy, tests, dependency-policy, audit, secret-scan, and SPDX SBOM checks on the supported CI platforms.
- The remaining rows above were verified from the public module boundaries and implementation files on 2026-08-21.

## Exit rule

Each migration change links to the relevant design contract, adds automated evidence at the appropriate test layer, and removes or narrows the corresponding row. Keep historical completion records in [the progress tracker](../PROGRESS.md), not in this document or the stable design documents.
