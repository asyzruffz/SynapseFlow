# Implementation gap

This tracker compares the target [architecture](architecture.md) with the active workspace. It was refreshed on 2026-08-23 after the Verified local inference milestone. Completed local-inference work is recorded in [the Milestone 2 tracker](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md), not repeated here.

## Delivered baseline

The active workspace has framework-independent manifest, generation, and port contracts; signed-manifest trust verification; a provisioned content-addressed local cache; a CPU-only GGUF/Llama backend; one shared application workflow; and CLI plus loopback API entry points. The active crate map and dependency direction are documented in [workspace architecture](workspace-architecture.md).

## Remaining gaps

| Design area | Current code boundary | Required migration outcome |
|---|---|---|
| Distributed domain and application contracts | Manifest, artifact, generation, audit, backend, registry, transport, clock, and peer-directory contracts exist. There are no active shard, execution-plan, frame, session-state, cancellation, checkpoint, routing, or replica contracts/use cases. | Add versioned distributed contracts and deterministic planning/session application services before implementing distributed execution. |
| Loopback sharding and recovery | The supported backend loads one whole local GGUF. The historical executor/kernel code is excluded from the active workspace and is not a supported path. | Implement the roadmap’s layer-wise two-shard baseline, validated activation-frame codec, cancellation/deadline semantics, checkpoint boundaries, corruption handling, bounded retries, and replica recovery. |
| Transport, workers, and discovery | `Transport` and `PeerDirectory` are framework-independent ports only. There is no frame transport implementation, worker process, enrollment, health reporting, backpressure, QUIC, or remote execution. | Deliver bounded loopback transport first, then mutually authenticated QUIC, static peer enrollment, capabilities/health, and remote-worker operation in their roadmap milestones. |
| Model distribution and trust operations | Milestone 2 resolves explicitly provisioned local manifest bytes and maps a declared HTTPS artifact URI to an explicitly provisioned local source. It has no remote registry fetch, resumable download, credential handling, publisher rotation/revocation, or quarantine workflow. | Add policy-controlled remote model distribution and durable trust/key operations without weakening manifest, signature, or content verification. |
| Node security and operations | The node is intentionally loopback-only with bounded JSON and stable errors. It has no authentication, authorization, quotas, cancellation endpoint, configuration-file/environment precedence, readiness/liveness endpoints, tracing, metrics, persistent audit store, or multi-user isolation. | Add the production node/API security, observability, configuration, and operational controls in the API hardening milestone. |
| Incentives and advanced security | No incentive, governance, discovery, TEE, privacy-computation, erasure-coding, tensor-parallel, or MoE implementation is active. | Treat each as a separately designed and reviewed proposal with threat model, ADR, performance evidence, and roadmap approval. |
| Deferred cross-platform release evidence | Windows local quality and fixture acceptance are recorded; the user explicitly deferred the platform-specific Linux quality and benchmark record. CI defines secret-scan and SBOM jobs, but no successful CI run is recorded in the milestone evidence. | Obtain Linux quality/fixture measurements and successful CI secret-scan/SBOM evidence before any release claims current cross-platform validation. |

## Evidence basis

- The completed local workflow, fixture acceptance, Windows quality gate, and explicit Linux deferral are recorded in [the Milestone 2 tracker](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md).
- The Windows fixture measurement is recorded in [the acceptance record](acceptance/verified-local-inference-2026-08-22.md); the accepted vector and model remain outside Git.
- The active public module boundaries were reviewed on 2026-08-23. The obsolete pre-milestone crates were removed; their target responsibilities are represented by the remaining rows and active architecture documentation.

## Exit rule

Narrow or remove a row only when its contract, implementation, focused automated evidence, and relevant operational evidence exist. Keep completion history in the [Milestone 2 tracker](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md), not in this gap document.
