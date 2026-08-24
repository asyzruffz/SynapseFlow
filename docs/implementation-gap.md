# Implementation gap

This tracker compares the target [architecture](architecture.md) with the active
workspace. It was refreshed on 2026-08-24 after the Loopback sharding
milestone. Completed local-inference and loopback-sharding work is recorded in
[the Milestone 2 tracker](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md) and [the
Milestone 3 tracker](../MILESTONE-3-LOOPBACK-SHARDING.md), not repeated here.

## Delivered baseline

The active workspace has framework-independent manifest, generation, sharding,
frame, session, and port contracts; signed-manifest trust verification; a
provisioned content-addressed local cache; a CPU-only GGUF/Llama local backend;
and the Loom layer-range backend. Deterministic planning, bounded local
loopback transport, checkpointed replica recovery, and a contiguous-baseline
comparison are delivered for the schema-v2 `layer_range_v1` path. The active
crate map and dependency direction are documented in [workspace
architecture](workspace-architecture.md).

## Remaining gaps

| Design area | Current code boundary | Required migration outcome |
|---|---|---|
| Remote workers and discovery | The bounded loopback transport and independently addressable local workers exercise frame, queue, cancellation, retry, and replica semantics. There is no remote process transport, mutual TLS, enrollment, remote health reporting, circuit breaking, or peer discovery. | Add mutually authenticated QUIC, static peer enrollment, capabilities/health, circuit breakers, and two-machine remote-worker operation in the Remote workers milestone. |
| Model distribution and trust operations | The local registry resolves explicitly provisioned manifest bytes and maps a declared HTTPS artifact URI to an explicitly provisioned local source. Loom reuses only a manifest-verified artifact; it adds no remote acquisition or trust operation. | Add policy-controlled remote model distribution, resumable download, credential handling, publisher rotation/revocation, quarantine, and durable trust/key operations without weakening verification. |
| Node security and operations | The node is intentionally loopback-only with bounded JSON and stable errors. It has no authentication, authorization, quotas, cancellation endpoint, configuration-file/environment precedence, readiness/liveness endpoints, tracing, metrics, persistent audit store, or multi-user isolation. | Add the production node/API security, observability, configuration, and operational controls in the API hardening milestone. |
| Incentives and advanced security | No incentive, governance, discovery, TEE, privacy-computation, erasure-coding, tensor-parallel, or MoE implementation is active. | Treat each as a separately designed and reviewed proposal with threat model, ADR, performance evidence, and roadmap approval. |
| Deferred cross-platform release evidence | Milestone 3 acceptance is explicitly Windows-scoped. The Windows Loom record is not a signed external schema-v2 fixture record, and no successful Linux or CI secret-scan/SBOM evidence is recorded for this milestone. | Obtain a signed schema-v2 external-fixture record, Linux quality/fixture measurement, and successful CI secret-scan/SBOM evidence before any release claims current cross-platform loopback-sharding validation. |

## Evidence basis

- The completed local workflow and fixture evidence are recorded in [the Milestone 2 tracker](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md).
- The completed schema-v2 contracts, Loom range execution, loopback recovery, Windows validation, and dependency review are recorded in [the Milestone 3 tracker](../MILESTONE-3-LOOPBACK-SHARDING.md).
- The sharding design and rollback boundary are recorded in [ADR 0006](adr/0006-loom-layer-range-backend.md); the canonical schema-v2 manifest and activation-frame vectors are committed domain tests.
- The generated-fixture and provisioned Windows acceptance records are [available here](acceptance/loopback-sharding-generated-fixture-2026-08-24.md) and [here](acceptance/loopback-sharding-windows-2026-08-24.md). They retain no model weights, prompts, raw activations, or signing material.

## Exit rule

Narrow or remove a row only when its contract, implementation, focused automated
evidence, and relevant operational evidence exist. Keep completion history in
the milestone trackers, not in this gap document.
