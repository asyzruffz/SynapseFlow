# Roadmap

## 1. Foundation

Define supported platforms and Rust version; establish CI, release policy, licence, contribution rules, dependency policy, artifact policy, typed domain errors, and architecture decision records.

**Acceptance:** a clean clone passes the quality gate on every supported platform without developer-specific state.

## 2. Verified local inference

Deliver one supported model format and backend, manifest resolution, tokenizer support, correct generation/sampling, and a CLI/API path using verified model references.

**Acceptance:** a fixed model fixture and seed produce a tested token stream; invalid model references, formats, and signatures return typed errors.

**Completion note:** Windows quality and provisioned fixture evidence are recorded in the [Milestone 2 tracker](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md). Linux platform-specific validation is explicitly deferred and must be completed before a release claims current cross-platform validation.

## 3. Loopback sharding

Implement the manifest/frame/session contracts, a deterministic subgraph executor, and two local workers using the production codec and transport semantics. Measure activation size, latency, compression, memory, cancellation, corruption, timeout, and replica recovery.

**Acceptance:** a two-shard integration test matches the full-model baseline within its declared numerical tolerance and handles induced failure within a bounded retry/deadline policy.

## 4. Operable node

Add streaming API, authentication, authorization, limits, cancellation, configuration validation, readiness/liveness, traces, metrics, audit events, and runbooks.

**Acceptance:** local load tests demonstrate observable latency/error/resource behavior and cover authorization and cancellation paths.

## 5. Remote workers

Add QUIC with mutual TLS, static peer enrollment, capabilities/health, bounded transport, deadline propagation, circuit breakers, replicas, and key management.

**Acceptance:** a two-machine failure test records p50/p95 latency, activation bandwidth, and recovery behavior under controlled loss and worker failure.

## 6. Controlled peer network

Add signed manifest publication, key rotation/revocation, peer governance, audit retention, and a reviewed security threat model. Evaluate discovery only after static-peer operation is stable and observable.

**Acceptance:** a controlled cohort can prove model provenance, authenticate every worker, enforce policy, and investigate a request audit trail.

Public discovery, incentives, TEEs, erasure coding, tensor parallelism, MoE routing, MPC, and homomorphic encryption are independent proposals that require dedicated ADRs, threat models, and benchmarks.
