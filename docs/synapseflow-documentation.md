# SynapseFlow documentation

> **Documentation map.** Normative documents identify themselves as sources of
> truth. Update those documents only through their stated redesign process;
> implementation must converge on their contracts.

SynapseFlow is planned to be a Rust-based distributed LLM inference system. It will execute immutable model shards across authenticated workers, passes bounded activation frames through a versioned protocol, and coordinates execution under explicit deadlines, cancellation, observability, and security policies.

SynapseFlow defines verified local inference for one signed GGUF/Llama model
tuple and bounded loopback sharding through schema-v2 shard declarations,
activation-frame protocol v1, Loom layer-range execution, and local worker
recovery. Remote operation and authenticated worker service capabilities remain
planned work.

## Design documentation

| Document | Purpose |
|---|---|
| [Architecture](architecture.md) | System boundaries, execution ownership, control/data planes, dependency direction, and sharding strategy. |
| [Workspace architecture](workspace-architecture.md) | Cargo boundaries, composition roots, and dependency-direction rules. |
| [Protocol](protocol.md) | Versioned manifests, data-plane frames, transport control semantics, and session lifecycle. |
| [Model management](model-management.md) | Remote acquisition, verification, local caching, provenance, and development artifacts. |
| [Verified local inference contract](verified-local-inference.md) | Initial GGUF/Llama compatibility tuple, fixture, error codes, and acceptance-vector procedure. |
| [Verified local inference validation](acceptance/verified-local-inference.md) | Provisioned fixture validation profile. |
| [Loopback-sharding validation](acceptance/loopback-sharding-windows.md) | Windows Loom baseline, two-range, and recovery validation profile. |
| [Loom loopback baseline](loom-loopback-baseline.md) | Contiguous-baseline comparison, frame route, tolerance, and recovery method. |
| [CLI](cli.md) | User-facing command design and operational behavior. |
| [Node API v1](node-api.md) | Authenticated public session and SSE contract. |
| [Operable node configuration](node-configuration.md) | Configuration precedence and production validation. |
| [Keycloak node setup](keycloak-setup.md) | Least-privilege realm, client, scope, audience, and key-rotation procedure. |
| [Operable node threat model](operable-node-threat-model.md) | Client-facing node assets, threats, and mitigations. |
| [Development](development.md) | Quality gates, test strategy, benchmarking, security, and contribution rules. |
| [Contributor onboarding](onboarding.md) | Clean-clone setup, local validation, fixtures, and review preparation. |
| [Operational runbooks](operations-runbooks.md) | Dependency, artifact, CI, security, and release-rollback response. |
| [Dependency management](dependency-management.md) | Locked builds, source and licence policy, audits, and reproducible Cargo caches. |
| [Code review](code-review-policy.md) | Required review, sensitive-change, and merge criteria. |
| [Compatibility](compatibility.md) | Compatibility commitments for crates, CLI, APIs, protocols, manifests, and configuration. |
| [Release process](release-process.md) | Release verification, SBOM/provenance, publishing, and withdrawal process. |
| [Roadmap](roadmap.md) | Intended delivery sequence and acceptance criteria. |
| [Implementation gap](implementation-gap.md) | Current workspace comparison against the normative design. |
| [Architecture decisions](adr/README.md) | Accepted platform, compatibility, and model/backend decisions. |

## Delivery direction

The [roadmap](roadmap.md) defines the integrated delivery baseline and planned
capabilities. [Implementation gap](implementation-gap.md) identifies the work
required for the current workspace to meet that design.

## Documentation conventions

The design documents describe the intended stable product, not temporary implementation details. Protocol examples are normative only after they are backed by a versioned schema and compatibility tests. Architecture decisions with material trade-offs should be recorded as ADRs.

Repository-wide contributor, security, licence, and changelog documents are available at the repository root: [CONTRIBUTING.md](../CONTRIBUTING.md), [SECURITY.md](../SECURITY.md), [LICENSE](../LICENSE), and [CHANGELOG.md](../CHANGELOG.md).
