# Implementation gap

> **Current-state assessment.** This document compares the workspace with the
> normative design documents. It does not redefine architecture, protocol, or
> policy: implementation must close the gaps recorded here. Update it when the
> codebase changes materially.

## Assessment baseline

The comparison uses [architecture](architecture.md),
[workspace architecture](workspace-architecture.md), [protocol](protocol.md),
[model management](model-management.md), [roadmap](roadmap.md), and the
applicable ADRs as the design baseline. The implementation baseline is the
current Cargo workspace, including its crate manifests, production modules,
focused tests, and CI configuration.

## Aligned implementation

- The workspace has distinct kernel, domain, ports, application, and adapter
  crates. Domain and ports remain independent of runtime, transport, and HTTP
  frameworks; application depends only on domain and ports.
- The domain implements signed manifest parsing and validation for the
  verified-local-inference schema and the loopback-sharding schema, including
  immutable references, artifact bindings, shard coverage, runtime profiles,
  and trusted-publisher verification.
- The activation-frame codec implements the bounded protocol-v1 envelope,
  checksums, extension handling, deadlines, and typed failures. The loopback
  transport sends canonical encoded bytes rather than Rust frame objects.
- The CLI composes the Crux kernel with the application-owned generation
  orchestrator, provisioned manifest registry, content-addressed local cache,
  and CPU-only llama.cpp adapter for the verified-local profile. The
  orchestrator selects a runtime only from a verified manifest and rejects a
  sharded profile when its runtime is not composed.
- Loom implements the Llama `layer_range_v1` shard-execution adapter with
  declared-range loading, activation-boundary conversion, range-local KV state,
  cancellation, and deadline checks. The loopback harness exercises a
  contiguous baseline, two ranges, a checkpoint, and replica recovery.
- CI defines locked formatting, build, Clippy, test, dependency-policy, audit,
  secret-scan, SBOM, and Tier-1/MSRV jobs.

## Gaps to close

| Design area | Current implementation boundary | Required outcome |
|---|---|---|
| Production sharded runtime composition | The generation orchestrator is the sole application entry point and selects schema-v1 verified-local or schema-v2 Loom profiles from a verified manifest. The CLI composes only the local runtime. Loom, loopback transport, planning, session checkpoints, and recovery remain independently exercised by adapter tests; no Loom-backed `ShardedGenerationRuntime` translates a prompt into token frames, drives a planned session, or samples final logits. | Implement the Loom-backed `ShardedGenerationRuntime` behind the orchestrator. It must tokenize the request, obtain the application-owned route and session context, execute declared stages through bounded transport, recover only through the session manager, sample validated final logits under the public policy, and release runtime state on every terminal path. |
| Operable node | The workspace has no node/server crate or HTTP implementation. `axum`, `tokio`, and `tokio-stream` are declared workspace dependencies but do not back a service boundary. | Add a node composition root that drives the kernel and shared orchestrator with bounded streaming requests, authentication, authorization, quotas, cancellation, configuration validation, readiness/liveness, typed errors, and safe output streaming. |
| Remote workers and transport | `synapseflow-adapter-loopback` is an in-process bounded transport. Worker capabilities come from an in-memory static directory; no remote process, connection lifecycle, peer enrollment, or discovery adapter exists. | Add mutually authenticated QUIC, static peer enrollment, remote health/capability reporting, bounded queues and backpressure, circuit breakers, and two-machine failure handling behind the existing ports. |
| Model distribution and trust operations | `synapseflow-adapter-local-cache` resolves explicitly provisioned manifest bytes and maps declared artifact URIs to local source files. It has no remote registry client, resumable transfer, credential handling, publisher rotation/revocation distribution, quarantine, or durable trust state. | Implement policy-controlled remote registry and artifact adapters while retaining signature, hash, size, provenance, staging, and atomic-promotion guarantees. |
| Authentication and authorization | The CLI accepts explicit local inputs and the ports expose no client identity, authorization, quota, or model-access policy. | Introduce authenticated client and worker identities, authorization decisions, rate/size/concurrency limits, and model-access controls at the node and transport boundaries. Keep those decisions in the application control plane, never in activation frames. |
| Operations and observability | Audit events are payload-safe but are currently consumed by in-memory sinks. Session state and checkpoint references are process-local; there is no persistent audit store, tracing, metrics exporter, health service, configuration source precedence, or multi-user isolation. | Add durable privacy-safe audit retention, session/checkpoint-reference persistence, traces, metrics, health/readiness reporting, configuration management, operational dashboards, and incident-safe correlation. |
| Provisioned runtime validation | The checked-in harness covers generated fixtures and the local runtime supports an externally provisioned fixture. There is no production composition for the schema-v2 Loom path, no signed external schema-v2 fixture workflow, and no Linux validation profile for either supported runtime profile. | Add provisioned schema-v2 fixture validation through the production sharded composition root and validate the supported runtime profiles on every Tier-1 platform before making cross-platform release claims. |

## Exit rule

Remove or narrow a row only when the corresponding implementation, focused
automated coverage, and operational behavior satisfy the normative contract.
