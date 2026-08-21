# SynapseFlow documentation

SynapseFlow is a Rust-based distributed LLM inference system. It executes immutable model shards across authenticated workers, passes bounded activation frames through a versioned protocol, and coordinates execution under explicit deadlines, cancellation, observability, and security policies.

## Design documentation

| Document | Purpose |
|---|---|
| [Architecture](architecture.md) | System boundaries, components, dependency direction, and sharding strategy. |
| [Protocol](protocol.md) | Versioned manifests, activation frames, control semantics, and session lifecycle. |
| [Model management](model-management.md) | Remote acquisition, verification, local caching, provenance, and development artifacts. |
| [CLI](cli.md) | User-facing command design and operational behavior. |
| [Development](development.md) | Quality gates, test strategy, benchmarking, security, and contribution rules. |
| [Roadmap](roadmap.md) | Ordered delivery milestones and acceptance criteria. |
| [Architecture decisions](adr/README.md) | Accepted platform, compatibility, and model/backend decisions. |

## Implementation tracking

[Implementation gap](implementation-gap.md) is the only document that compares the desired design to the present repository. Keep it small and delete sections as their corresponding milestones are completed; remove the document once there is no meaningful disparity.

## Documentation conventions

The design documents describe the intended stable product, not temporary implementation details. Protocol examples are normative only after they are backed by a versioned schema and compatibility tests. Architecture decisions with material trade-offs should be recorded as ADRs.
