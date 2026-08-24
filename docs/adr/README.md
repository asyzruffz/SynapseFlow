# Architecture decision records

Architecture Decision Records (ADRs) capture durable technical and product decisions. Each ADR states its status, context, decision, consequences, and the conditions that would justify superseding it.

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-supported-platforms-and-toolchain.md) | Tier-1 platforms, CPU scope, and Rust MSRV. | Accepted |
| [0002](0002-release-and-compatibility-policy.md) | Pre-1.0 release, API, protocol, and manifest compatibility policy. | Accepted |
| [0003](0003-initial-model-backend-scope.md) | Initial model format and backend compatibility scope. | Accepted |
| [0004](0004-verified-local-inference-contract.md) | Concrete local backend, fixture, and manifest-signature profile. | Accepted |
| [0005](0005-loopback-layer-range-execution.md) | Native layer-range backend and loopback recovery model. | Superseded in part by 0006 |
| [0006](0006-loom-layer-range-backend.md) | Loom Llama layer-range backend and Milestone 3 baseline. | Accepted |

New ADRs use a zero-padded sequence number and are not rewritten after acceptance. A later ADR supersedes an earlier one when a decision changes.
