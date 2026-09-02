# Roadmap

> **Source of truth.** This document defines the intended delivery sequence.
> Change it only through an explicit product or architecture redesign; missing
> implementation is work to complete, not a reason to narrow this roadmap.

## 1-3. Integrated delivery baseline

SynapseFlow's baseline combines platform/toolchain policy, locked quality and dependency controls, immutable signed manifests, verified local inference, and bounded loopback sharding. It supports the GGUF/Llama compatibility tuple, the `synapseflow-loom-llama-v1` layer-range profile, activation-frame protocol v1, deterministic planning, cancellation, deadlines, and checkpointed replica recovery. The local llama.cpp profile and the Loom sharding profile remain separate compatibility contracts.

The baseline requires a clean-clone quality gate on every Tier-1 platform and deterministic fixture validation for the supported runtime profile. It does not provide remote workers, QUIC, authentication, authorization, public node APIs, or production observability.

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

The following are independent, optional milestones. They do not replace or renumber the delivery sequence above and may be pursued in any order once their listed prerequisites are complete. Each requires a dedicated ADR, reviewed threat model, benchmark/operational validation, and explicit roadmap approval.

### Wallet-based client authentication

Add a wallet-signature authentication adapter for clients that choose to use one. The node issues a domain-bound, single-use, expiring challenge, verifies the signature, and establishes an ordinary short-lived application session. Wallet identity authenticates a client only; SynapseFlow authorization, quotas, recovery, and per-worker mutual-TLS identity remain separate concerns.

**Prerequisite:** Operable node.

**Acceptance:** replayed, expired, cross-domain, invalid, and revoked-session attempts are rejected; authorization and quotas remain enforceable independent of wallet ownership.

### Manifest transparency and optional ledger anchoring

Publish signed immutable manifest commitments, publisher-key changes, revocations, and supersession decisions through an append-only transparency record. Evaluate anchoring compact commitment roots to an external or future SynapseFlow ledger only when independently verifiable public history has a clear product need. Model weights, shards, prompts, activations, raw outputs, and credentials remain off-ledger; a content hash proves identity, not storage availability, model quality, safety, or licence compliance.

**Prerequisite:** Controlled peer network.

**Acceptance:** an independent verifier can reconstruct publisher and revocation state from retained records, detect conflicting history, and prove that quarantined or revoked manifests cannot start new sessions.

### Metered compute credits and signed execution receipts

Pilot non-transferable or controlled-environment compute credits before any public cryptocurrency. Bind an expiring signed quote to a model manifest, rate-card version, input/output caps, cancellation terms, and maximum reservation. Workers and the node produce privacy-safe signed receipts for validated execution and metered use; settlement releases unused reservation and applies the documented refund/failure policy.

**Prerequisites:** Operable node and Remote workers.

**Acceptance:** controlled-cohort tests reconcile reservations, usage, completion, cancellation, timeout, worker failure, duplicate receipt, and client/worker dispute outcomes without exposing prompt, activation, or output content.

### Custom-currency settlement protocol

If a project-owned cryptocurrency remains a product goal, design it as a settlement, escrow, stake, and governance protocol for the signed-credit pilot—not as the inference data plane or a replacement for worker transport. Define issuance, validator membership/consensus, finality, escrow, stake, slashing, fee policy, balance recovery, dispute authority, and anti-Sybil controls before implementation. Batch settlement commitments rather than creating an on-ledger transaction for each generated token.

**Prerequisites:** Controlled peer network and metered compute credits and signed execution receipts.

**Acceptance:** adversarial tests demonstrate no duplicate settlement, bounded loss and recovery behavior for unavailable validators/workers, deterministic receipt reconciliation, enforced escrow/refunds, and no prompt or activation data in the ledger.

### Verifiable execution research

Evaluate redundant replay sampling, hardware attestation, and narrowly scoped verifiable-compute proofs as ways to increase confidence in paid execution. Do not use tensor inference as proof of work or as the chain's consensus algorithm: inference verification must be independently benchmarked for cost, determinism, privacy, and resistance to artificial or non-customer work.

**Prerequisites:** Remote workers; the custom-currency settlement protocol is also required if evidence will drive automatic rewards or slashing.

**Acceptance:** a documented adversarial benchmark compares verification cost, latency, privacy exposure, false acceptance/rejection, and fraud recovery to the signed-receipt baseline; it justifies any production mechanism selected.

Public discovery, incentives, TEEs, erasure coding, tensor parallelism, MoE routing, MPC, and homomorphic encryption are independent proposals that require dedicated ADRs, threat models, and benchmarks.
