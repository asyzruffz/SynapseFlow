# Milestone 3 — Loopback sharding

**Status:** Planned  
**Roadmap milestone:** [Loopback sharding](docs/roadmap.md#3-loopback-sharding)  
**Last updated:** 2026-08-24

## Objective

Deliver deterministic, layer-wise execution across two loopback workers. The
workers must use SynapseFlow's versioned activation-frame codec and transport
semantics, recover from an induced failure within an explicit retry/deadline
policy, and match the verified whole-model baseline within a declared tolerance.

This milestone excludes remote workers, QUIC, worker enrolment,
authentication/authorization, streaming-node hardening, and new model families.

## Approved design decision

Milestone 3 includes a new **layer-range execution backend**. The current
`llama-cpp-2` adapter remains the verified whole-model baseline only: it cannot
execute a declared contiguous layer range or expose a boundary activation for a
second worker. The new backend must be an adapter behind framework-independent
ports; it must not introduce runtime, transport, or native-library dependencies
into `synapseflow-domain`, `synapseflow-ports`, or `synapseflow-application`.

Step 1 records the precise backend design in an ADR before implementation. It
must state the upstream/native API surface, fixture compatibility, numerical
comparison method, checkpoint/KV-state ownership, and safe rollback to the
whole-model baseline.

### Extensibility guardrail

Treat layer-range execution as the first versioned `layer_range_v1` sharding
strategy, not as the definition of sharding. The domain plan and backend port
must carry an explicit strategy identifier plus strategy-specific, validated
execution requirements. They must share only the concepts that are genuinely
common: immutable model/shard identities, bounded work, input/output frame
contracts, deadline/cancellation, checkpoint ownership, and safe outcomes.

Do not pre-implement tensor-parallel or MoE behavior, and do not force their
future collective-communication or routing requirements into a layer-range
trait. A later strategy receives its own ADR, manifest strategy version,
capability requirements, adapter implementation, and acceptance evidence.

## Completion criteria

- A signed immutable manifest declares two ordered contiguous layer shards and
  replica assignments without changing a Milestone 2 manifest.
- Two loopback workers execute the verified fixture through production frame
  encoding/decoding and bounded transport behavior.
- The sharded path matches the whole-model token/output baseline within its
  recorded numerical tolerance.
- Corruption, cancellation, timeout, and an induced worker failure produce safe
  stable errors or bounded replica recovery; no retry or duplicate work is
  unbounded.
- Golden protocol vectors, focused tests, and performance/recovery measurements
  exist.

## Progress rules

- Complete steps in order unless an explicit dependency makes parallel work
  safe. Do not check a box without the stated evidence.
- Preserve the architecture direction: applications depend on application,
  domain, and ports; adapters implement ports; domain and ports remain
  infrastructure-independent.
- Keep prompts, raw activations, weights, credentials, cache paths, and native
  backend diagnostics out of source control, fixtures, and safe logs.
- Treat protocol, manifest, codec, backend, and scheduler/recovery changes as
  sensitive changes under [the code-review policy](docs/code-review-policy.md).
- Run the standard format/check/Clippy/test gate with locked dependencies. Ask
  the project owner to run `cargo deny check` and `cargo audit`; do not run
  either command as part of this tracker.
- Update [docs/implementation-gap.md](docs/implementation-gap.md) only after
  every preceding completion criterion has evidence.

## Execution steps

### 1. Record the layer-range backend ADR

- [x] (2026-08-24: [ADR 0005](docs/adr/0005-loopback-layer-range-execution.md))
  Add the next zero-padded ADR that records the approved decision to add a
  layer-range backend for Milestone 3.
- [x] (2026-08-24: [ADR 0005](docs/adr/0005-loopback-layer-range-execution.md))
  Specify verified shard loading, contiguous range execution, boundary-state
  transfer, and final-logit production.
- [x] (2026-08-24: [ADR 0005](docs/adr/0005-loopback-layer-range-execution.md))
  Specify fixture compatibility, tolerance calculation, KV/cache ownership,
  checkpoint boundaries, resource limits, native dependency provenance, and
  rollback/failure behavior.
- [x] (2026-08-24: [ADR 0005](docs/adr/0005-loopback-layer-range-execution.md))
  Confirm that llama.cpp whole-model/RPC mechanisms are not substitutes for
  SynapseFlow-owned activation codec and transport semantics.

**Review:** Does the ADR make every layer boundary observable and testable
without leaking native runtime types into domain/application contracts? Are the
tolerance and rollback criteria concrete enough to reject an unsound backend?

**Evidence:** accepted [ADR 0005](docs/adr/0005-loopback-layer-range-execution.md).
Runtime/distributed-systems and compatibility review, plus dependency/licence
review for any new direct dependency, remain required before merge.

### 2. Define versioned distributed domain contracts

- [x] (2026-08-24: [`execution`](domain/src/execution/mod.rs)) Add focused
  modules for shard identity, ordered layer ranges, execution plans,
  tensor/activation metadata, frame envelopes, session IDs, checkpoints, retry
  policy, cancellation, and session transitions.
- [x] (2026-08-24: [`ExecutionStrategy`](domain/src/execution/strategy.rs))
  Model the execution strategy and its version explicitly, with
  `layer_range_v1` as the only accepted implementation in this milestone.
- [x] (2026-08-24: [`frame`](domain/src/execution/frame.rs)) Add explicit
  bounded types for byte lengths, tensor rank/dimensions, sequences, in-flight
  work, and remaining deadline.
- [x] (2026-08-24: [`DomainError`](domain/src/error.rs)) Extend stable
  errors/codes for protocol/model versions, frame bounds/hash/dtype/order
  failures, cancellation, retry exhaustion, unavailable workers, and replica
  recovery failure.
- [x] (2026-08-24: domain execution unit tests) Add focused invariant and
  transition tests beside every contract.

**Review:** Can every contract be tested without Tokio, channels, HTTP, QUIC,
a codec library, or a model runtime? Are terminal transitions and cancellation
idempotent where required? Does the strategy seam avoid assuming that all future
strategies have linear layer boundaries?

**Evidence:** 20 passing `synapseflow-domain` tests for valid/invalid ranges,
limits, frame metadata, errors, cancellation, retries, and session transitions;
the all-feature workspace build and Clippy gate pass. Architecture review remains
required before merge.

### 3. Version the manifest for shard execution

- [ ] Introduce a manifest schema version for shard declarations; retain
  Milestone 2 schema-v1 parsing and semantics unchanged.
- [ ] Declare shard ID, artifact identity/hash/size, contiguous layer interval,
  execution order, runtime compatibility, and allowed replica placement.
- [ ] Validate complete ordered non-overlapping coverage and compatibility with
  the declared model/backend before acquisition or allocation.
- [ ] Add canonical signed golden vectors and negative tests for malformed,
  incomplete, overlapping, reordered, altered, or incompatible shard manifests.

**Review:** Is a changed shard layout an immutable new manifest identity? Can a
schema-v1 consumer safely reject schema-v2 execution instead of misreading it?

**Evidence:** parser compatibility vectors; signature and malformed-input tests;
compatibility-maintainer approval and migration note.

### 4. Specify and implement the activation-frame codec

- [ ] Record protocol-v1 fields for envelope, message type, session/stream ID,
  sequence, model/shard identity, tensor descriptor, compression, payload hash,
  remaining deadline, and safe trace ID.
- [ ] Select and document an explicit binary schema/codec with reproducible
  encoding; define canonical payload bytes covered by the hash and compression
  order.
- [ ] Validate every untrusted bound before decode, decompression, allocation,
  buffering, or reassembly.
- [ ] Add golden bytes, cross-version behavior, malformed/truncated input,
  oversized input, decompression-bomb, checksum, dtype, endianness, and sequence
  tests.

**Review:** Does the codec reject unsupported versions safely while allowing
documented additive fields? Is every allocation and decompression limit enforced
before resource use?

**Evidence:** protocol update, golden vectors, fuzz/property coverage, and
compatibility-maintainer approval.

### 5. Replace placeholder distributed ports

- [ ] Evolve `Transport` into framework-independent send/receive, ACK/NACK,
  bounded-queue, availability, and shutdown/cancellation operations.
- [ ] Evolve `PeerDirectory` into static loopback worker capability, health,
  shard availability, and replica lookup operations.
- [ ] Add a shard-execution backend port that accepts a verified declared range
  and validated input boundary, then emits the next boundary or final logits;
  make the strategy capability explicit rather than naming the port after layers.
- [ ] Define safe audit events for model, shard, worker, session outcome, and
  retry/fallback count without payload content.

**Review:** Are adapter concerns absent from the traits? Can deterministic
in-memory fakes fully drive planning and session tests? Can a future strategy
add its own validated execution requirements without weakening this port?

**Evidence:** port contract tests and workspace dependency-graph review.

### 6. Build deterministic planning and session management

- [ ] Implement a planner that derives two contiguous assignments from the
  verified schema-v2 manifest and static loopback capabilities.
- [ ] Validate that the selected workers advertise support for `layer_range_v1`;
  leave unsupported future strategy identifiers unplanned and safely rejected.
- [ ] Implement the documented lifecycle: `Created` → `Planned` → `Running` →
  terminal, with bounded `Retrying` and idempotent `Cancelling` paths.
- [ ] Define idempotency keys, checkpoint ownership, remaining-deadline
  propagation, retry budget, retryable errors, and duplicate-work prevention.
- [ ] Test deterministic plans, terminal transitions, expired deadlines,
  repeated cancellation, exhausted retries, and safe audit events with fakes.

**Review:** Does one source own session state and checkpoint selection? Can a
failing worker cause only bounded, attributable work?

**Evidence:** hermetic application tests for planning, deadline/cancellation,
retry, and audit behavior; distributed-systems maintainer approval.

### 7. Implement the bounded loopback transport and workers

- [ ] Add a loopback transport adapter that uses the production codec for all
  control and data frames; never pass Rust objects directly between workers.
- [ ] Run two independently addressable local workers with bounded queues,
  ordered streams, ACK/NACK processing, and backpressure.
- [ ] Add deterministic controls for delay, timeout, dropped ACK, corrupt frame,
  unavailable worker, and mid-stage failure.
- [ ] Enforce cancellation/shutdown cleanup and prevent queued work surviving a
  terminal session.

**Review:** Does loopback exercise the same frame validation/control semantics
needed by future transport adapters? Are queues and worker lifetimes bounded
under every injected failure?

**Evidence:** adapter integration tests for ordering, bounds, corruption,
backpressure, timeout, and cancellation.

### 8. Implement the layer-range execution backend

- [ ] Create focused modules for verified shard loading, range validation,
  native/runtime integration, execution, and output conversion; keep `lib.rs`
  limited to intended exports.
- [ ] Implement only the `layer_range_v1` strategy capability behind the generic
  sharding port; keep range-specific validation inside this adapter.
- [ ] Load only immutable manifest-verified shards and reject range, model
  version, hash, dtype, or runtime incompatibility before execution.
- [ ] Execute only the declared contiguous range, accept validated prior
  boundary state, and return the next boundary or final logits.
- [ ] Enforce approved KV/context ownership, memory limits, cancellation checks,
  remaining deadline, and cleanup behavior.
- [ ] Add range-isolation and deterministic-output tests using small
  licence-cleared generated fixtures; keep the real model external to the
  default test gate.

**Review:** Does a worker provably avoid loading/executing layers outside its
declared range? Are native failures translated into safe stable domain errors?

**Evidence:** runtime/model-maintainer review; focused backend tests; locked
dependency/build evidence on both Tier-1 targets.

### 9. Integrate the two-shard baseline and replica recovery

- [ ] Compose the schema-v2 manifest, planner, session manager, workers,
  production codec/transport, and layer-range backend in one harness.
- [ ] Establish the whole-model baseline and record comparison inputs, backend
  versions, tolerance, and comparison method.
- [ ] Compare the two-shard result to the baseline for token IDs and approved
  numeric output/activation tolerance.
- [ ] Fail a primary worker after a known checkpoint; select the allowed replica
  and prove recovery completes within retry budget and deadline.

**Review:** Does this prove stage-to-stage activation transfer rather than two
whole-model calls? Is recovery from a valid checkpoint and free of untracked
duplicate execution?

**Evidence:** provisioned two-shard record; baseline-equivalence, recovery,
timeout, and cancellation test results.

### 10. Measure performance and resource behavior

- [ ] Measure activation bytes, compression ratio/CPU cost, per-stage and
  end-to-end latency, throughput, peak process memory, queue depth, retry count,
  and recovery latency.
- [ ] Record fixture/manifest hash, shard layout, backend/codec/protocol
  versions, platform, hardware, policy, input shape, and measurement method.
- [ ] Compare compressed and uncompressed loopback runs; retain raw sensitive
  payloads outside the repository and publish safe aggregates only.
- [ ] Define evidence required before any future block-chunking decision; do not
  add tensor parallelism or remote transport in this milestone.

**Review:** Are measurements reproducible and comparable to the whole-model
baseline? Do they expose codec, compression, and recovery costs?

**Evidence:** dated acceptance/performance record for each Tier-1 platform.

### 11. Complete validation and review handoff

- [ ] Run `cargo fmt --all -- --check`, `cargo check --workspace --all-targets
  --all-features --locked`, `cargo clippy --workspace --all-targets
  --all-features --locked -- -D warnings`, and `cargo test --workspace
  --all-targets --all-features --locked` on both Tier-1 targets.
- [ ] Ask the project owner to run `cargo deny check` and `cargo audit`, then
  record their reported results; do not run them on the owner's behalf.
- [ ] Verify protocol/manifest migration notes, runbooks, privacy rules,
  dependency provenance, and rollback behavior.
- [ ] Obtain compatibility, runtime/model, distributed-systems, and security
  reviews required by the code-review policy.

**Review:** Does every criterion have automated and operational evidence? Are
all public compatibility surfaces and rollback procedures documented?

**Evidence:** successful Tier-1 quality records, user-reported dependency/audit
results, approvals, and completed acceptance record.

### 12. Update delivery documentation and implementation gap

- [ ] Update roadmap, architecture/protocol/compatibility/model-management
  documents, developer guide, and runbooks with delivered scope and boundaries.
- [ ] Refresh `docs/implementation-gap.md` only after Steps 1–11; retain QUIC,
  enrolment, remote workers, authorization, node operations, and advanced
  parallelism as future work.
- [ ] Add a dated completion note linking ADR, protocol vectors, acceptance
  results, and performance record.

**Review:** Does documentation distinguish delivered loopback sharding from
future remote/distributed-node claims?

**Evidence:** reviewed documentation diff and final implementation-gap review.

## Milestone sign-off

- [ ] The layer-range backend ADR and all implementation steps are complete.
- [ ] The two-shard path matches the declared whole-model baseline tolerance.
- [ ] Corruption, cancellation, timeout, and replica failure satisfy the bounded
  policy with safe errors or recovery.
- [ ] Tier-1 validation and review requirements have recorded evidence.
- [ ] The remaining implementation gap accurately lists only future work.
