# SynapseFlow documentation

SynapseFlow is planned to be a Rust-based distributed LLM inference system. It will execute immutable model shards across authenticated workers, passes bounded activation frames through a versioned protocol, and coordinates execution under explicit deadlines, cancellation, observability, and security policies.

SynapseFlow currently delivers verified local inference for one signed GGUF/Llama model tuple on a local CPU runtime. The distributed architecture, frame protocol, authenticated workers, and sharding documents describe the intended later design; they are not delivered capabilities in Milestone 2.

## Design documentation

| Document | Purpose |
|---|---|
| [Architecture](architecture.md) | System boundaries, components, dependency direction, and sharding strategy. |
| [Active workspace architecture](workspace-architecture.md) | Current crate map, dependency-direction rules, and legacy-crate migration boundary. |
| [Protocol](protocol.md) | Versioned manifests, activation frames, control semantics, and session lifecycle. |
| [Model management](model-management.md) | Remote acquisition, verification, local caching, provenance, and development artifacts. |
| [Verified local inference contract](verified-local-inference.md) | Initial GGUF/Llama compatibility tuple, fixture, error codes, and acceptance-vector procedure. |
| [Local acceptance records](acceptance/verified-local-inference-2026-08-22.md) | Provisioned fixture measurements and retained vector-hash evidence. |
| [CLI](cli.md) | User-facing command design and operational behavior. |
| [Development](development.md) | Quality gates, test strategy, benchmarking, security, and contribution rules. |
| [Contributor onboarding](onboarding.md) | Clean-clone setup, local validation, fixtures, and review preparation. |
| [Operational runbooks](operations-runbooks.md) | Dependency, artifact, CI, security, and release-rollback response. |
| [Dependency management](dependency-management.md) | Locked builds, source and licence policy, audits, and reproducible Cargo caches. |
| [Code review](code-review-policy.md) | Required review, sensitive-change, and merge criteria. |
| [Compatibility](compatibility.md) | Compatibility commitments for crates, CLI, APIs, protocols, manifests, and configuration. |
| [Release process](release-process.md) | Release verification, SBOM/provenance, publishing, and withdrawal process. |
| [Roadmap](roadmap.md) | Ordered delivery milestones and acceptance criteria. |
| [Architecture decisions](adr/README.md) | Accepted platform, compatibility, and model/backend decisions. |

## Implementation tracking

[Milestone 2 tracker](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md) is the active delivery record. [Implementation gap](implementation-gap.md) compares the desired architecture to the repository only after the tracker’s final update step.

## Documentation conventions

The design documents describe the intended stable product, not temporary implementation details. Protocol examples are normative only after they are backed by a versioned schema and compatibility tests. Architecture decisions with material trade-offs should be recorded as ADRs.

Repository-wide contributor, security, licence, and changelog documents are available at the repository root: [CONTRIBUTING.md](../CONTRIBUTING.md), [SECURITY.md](../SECURITY.md), [LICENSE](../LICENSE), and [CHANGELOG.md](../CHANGELOG.md).
