# Roadmap

## 1. Foundation

Define supported platforms and Rust version; establish CI, release policy, licence, contribution rules, dependency policy, artifact policy, typed domain errors, and architecture decision records.

**Acceptance:** a clean clone passes the quality gate on every supported platform without developer-specific state.

## 2. Verified local inference

Deliver one supported model format and backend, manifest resolution, tokenizer support, correct generation/sampling, and a CLI/API path using verified model references.

**Acceptance:** a fixed model fixture and seed produce a tested token stream; invalid model references, formats, and signatures return typed errors.

**Completion note:** Windows quality and provisioned fixture evidence are recorded in the [Milestone 2 tracker](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md). Linux platform-specific validation is explicitly deferred and must be completed before a release claims current cross-platform validation.

## 3. Loopback sharding

Implement the manifest/frame/session contracts, the deterministic Loom Llama
subgraph executor, and two local workers using the production codec and
transport semantics. Its contiguous full-model mode is the sharding baseline.
Measure activation size, latency, compression, memory, cancellation, corruption,
timeout, and replica recovery.

**Acceptance:** a two-shard integration test matches the same pinned Loom
full-model baseline within its declared numerical tolerance and handles induced
failure within a bounded retry/deadline policy. The Milestone 2 llama.cpp record
remains a separate verified-local-inference compatibility record.

**Completion note (2026-08-24):** The schema-v2 manifest/frame/session
contracts, Loom `layer_range_v1` adapter, bounded local loopback workers, and
replica-recovery harness are complete. The generated-fixture and provisioned
Windows records demonstrate the declared contiguous-baseline comparison and
recovery policy. Milestone acceptance is explicitly Windows-scoped; this is not
a cross-platform release claim. See [the Milestone 3 tracker](../MILESTONE-3-LOOPBACK-SHARDING.md),
[ADR 0006](adr/0006-loom-layer-range-backend.md), and [the Windows acceptance
record](acceptance/loopback-sharding-windows-2026-08-24.md).

## 4. Operable node

Add streaming API, authentication, authorization, limits, cancellation, configuration validation, readiness/liveness, traces, metrics, audit events, and runbooks.

**Acceptance:** local load tests demonstrate observable latency/error/resource behavior and cover authorization and cancellation paths.

## 5. Remote workers

Add QUIC with mutual TLS, static peer enrollment, capabilities/health, bounded transport, deadline propagation, circuit breakers, replicas, and key management.

**Acceptance:** a two-machine failure test records p50/p95 latency, activation bandwidth, and recovery behavior under controlled loss and worker failure.

## 6. Controlled peer network

Add signed manifest publication, key rotation/revocation, peer governance, audit retention, and a reviewed security threat model. Evaluate discovery only after static-peer operation is stable and observable.

**Acceptance:** a controlled cohort can prove model provenance, authenticate every worker, enforce policy, and investigate a request audit trail.

## Optional future milestones

The following are independent, optional milestones. They do not replace or
renumber the delivery path above, and may be pursued in any order once their
listed prerequisites are complete. Each requires a dedicated ADR, reviewed
threat model, benchmark/operational evidence, and explicit roadmap approval.

### Wallet-based client authentication

Add a wallet-signature authentication adapter for clients that choose to use
one. The node issues a domain-bound, single-use, expiring challenge, verifies
the signature, and establishes an ordinary short-lived application session.
Wallet identity authenticates a client only; SynapseFlow authorization,
quotas, recovery, and per-worker mutual-TLS identity remain separate concerns.

**Prerequisite:** Operable node.

**Acceptance:** replayed, expired, cross-domain, invalid, and revoked-session
attempts are rejected; authorization and quotas remain enforceable independent
of wallet ownership.

### Manifest transparency and optional ledger anchoring

Publish signed immutable manifest commitments, publisher-key changes,
revocations, and supersession decisions through an append-only transparency
record. Evaluate anchoring compact commitment roots to an external or future
SynapseFlow ledger only when independently verifiable public history has a
clear product need. Model weights, shards, prompts, activations, raw outputs,
and credentials remain off-ledger; a content hash proves identity, not storage
availability, model quality, safety, or licence compliance.

**Prerequisite:** Controlled peer network.

**Acceptance:** an independent verifier can reconstruct publisher and
revocation state from retained records, detect conflicting history, and prove
that quarantined or revoked manifests cannot start new sessions.

### Metered compute credits and signed execution receipts

Pilot non-transferable or controlled-environment compute credits before any
public cryptocurrency. Bind an expiring signed quote to a model manifest,
rate-card version, input/output caps, cancellation terms, and maximum
reservation. Workers and the node produce privacy-safe signed receipts for
validated execution and metered use; settlement releases unused reservation
and applies the documented refund/failure policy.

**Prerequisites:** Operable node and Remote workers.

**Acceptance:** controlled-cohort tests reconcile reservations, usage,
completion, cancellation, timeout, worker failure, duplicate receipt, and
client/worker dispute outcomes without exposing prompt, activation, or output
content.

### Custom-currency settlement protocol

If a project-owned cryptocurrency remains a product goal, design it as a
settlement, escrow, stake, and governance protocol for the signed-credit
pilot—not as the inference data plane or a replacement for worker transport.
Define issuance, validator membership/consensus, finality, escrow, stake,
slashing, fee policy, balance recovery, dispute authority, and anti-Sybil
controls before implementation. Batch settlement commitments rather than
creating an on-ledger transaction for each generated token.

**Prerequisites:** Controlled peer network and metered compute credits and
signed execution receipts.

**Acceptance:** adversarial tests demonstrate no duplicate settlement, bounded
loss and recovery behavior for unavailable validators/workers, deterministic
receipt reconciliation, enforced escrow/refunds, and no prompt or activation
data in the ledger.

### Verifiable execution research

Evaluate redundant replay sampling, hardware attestation, and narrowly scoped
verifiable-compute proofs as ways to increase confidence in paid execution.
Do not use tensor inference as proof of work or as the chain's consensus
algorithm: inference verification must be independently benchmarked for
cost, determinism, privacy, and resistance to artificial or non-customer work.

**Prerequisites:** Remote workers; the custom-currency settlement protocol is
also required if evidence will drive automatic rewards or slashing.

**Acceptance:** a documented adversarial benchmark compares verification cost,
latency, privacy exposure, false acceptance/rejection, and fraud recovery to
the signed-receipt baseline; it justifies any production mechanism selected.

Public discovery, incentives, TEEs, erasure coding, tensor parallelism, MoE routing, MPC, and homomorphic encryption are independent proposals that require dedicated ADRs, threat models, and benchmarks.
