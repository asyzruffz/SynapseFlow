# Milestone 2 — Verified local inference

**Status:** Not started  
**Roadmap milestone:** [Verified local inference](docs/roadmap.md#2-verified-local-inference)  
**Last updated:** 2026-08-21

## Objective

Deliver a single-node, verified local inference path for one model family, format, and backend:

- GGUF weights for the Llama family on Tier-1 CPU platforms;
- one maintained llama.cpp-compatible Rust adapter;
- selection and acquisition exclusively through a verified immutable manifest reference;
- tokenizer-aware generation with deterministic seed mode and supported temperature/top-p sampling; and
- CLI and local API entry points that return typed, safe errors for invalid model references, formats, artifacts, and signatures.

This milestone also establishes the dependency direction and public contracts required by [the target architecture](docs/architecture.md). It does **not** implement future-milestone behavior such as sharded execution, frame transport, remote workers, authentication/authorization, or peer discovery. Those capabilities will be represented only by framework-independent ports where required to preserve the architecture.

## Completion criteria

- The fixed, separately provisioned verified GGUF fixture and a named seed produce the documented, tested token stream.
- Invalid manifest references, signatures, artifact hashes/sizes, unsupported format/architecture/tokenizer declarations, and backend incompatibility each return a stable typed error without a panic.
- `synapseflow run --model <manifest-reference> ...` resolves, verifies, caches, loads, tokenizes, and generates locally; the local API exposes the same application use case rather than a separate inference path.
- The code conforms to the architectural dependency direction: applications → application services → domain/ports → adapters. Domain and ports have no runtime, HTTP, QUIC, database, or model-backend dependencies.
- The milestone’s tests and the repository quality gate pass on Tier-1 platforms. Model-backed acceptance tests use an explicitly provisioned integration environment and never make the default gate download or require model artifacts.

## Progress rules

- Complete steps in order unless a recorded dependency makes parallel work safe.
- Change a step’s checkbox only after its listed evidence is present and the relevant quality checks pass.
- Add the date and concise evidence link/path to the completed step; record decisions that alter scope in an ADR.
- Keep real weights, credentials, private prompts, and activation data out of Git. The existing checked-in GGUF must be removed from normal source tracking through a separately reviewed, recoverable artifact migration before treating the model-management policy as satisfied.
- Do not update `docs/implementation-gap.md` until the final step; use this file as the in-progress source of truth.

## Execution steps

### 1. Establish the milestone contract and fixture provenance

- [ ] Document the exact supported compatibility tuple: GGUF, Llama architecture, llama.cpp-compatible Rust adapter, Tier-1 CPU platforms, supported quantization(s), tokenizer behavior, context limits, and sampling semantics.
- [ ] Define the fixed integration fixture’s approved manifest reference, publisher key, licence/provenance, artifact/tokenizer hashes and sizes, test prompt, named seed, generation parameters, and expected token stream.
- [ ] Define stable public error codes/types and error mapping for reference parsing, manifest/schema validation, trust/signature failure, artifact integrity/cache failure, compatibility failure, tokenizer failure, backend initialization, and generation failure.
- [ ] Add or amend an ADR only if the selected adapter, cryptographic scheme, canonical manifest representation, or fixture source requires a durable decision beyond [ADR 0003](docs/adr/0003-initial-model-backend-scope.md).

**Evidence:** compatibility/fixture specification, typed-error tests, and any required ADR.

### 2. Restructure the workspace around the target architecture

- [ ] Introduce or rename crates/modules to make the architecture explicit: `synapseflow-domain`, `synapseflow-ports`, `synapseflow-application`, concrete `synapseflow-adapters-*`, and application binaries/services for CLI and the local node/API.
- [ ] Move framework-independent identifiers, model/manifest declarations, compatibility rules, generation policy, request/result types, and typed domain errors into domain.
- [ ] Define object-safe ports for manifest resolution, artifact/cache storage, model backend/tokenization/generation, audit/telemetry, clock, and local request serving; keep ports independent of Tokio, HTTP, QUIC, databases, Candle, and llama.cpp.
- [ ] Put the generation use case in the application layer and make the CLI/API call it through ports. Isolate or retire the current direct `core`/`inference` coupling and unavailable Candle/LlamaCpp placeholder path so they cannot be mistaken for supported behavior.
- [ ] Add compile-time/dependency-direction checks and deterministic unit tests that demonstrate the domain and application layers can run with in-memory test adapters.

**Evidence:** workspace dependency graph review, domain/application unit tests, and strict all-feature build without warnings.

### 3. Implement versioned manifest parsing and trust verification

- [ ] Implement the canonical manifest schema from [the protocol](docs/protocol.md), including schema version, immutable model identity/version, GGUF/Llama compatibility declaration, tokenizer and artifact declarations, publisher key identifier, licence/provenance, and signature envelope.
- [ ] Validate manifest invariants before use: bounded input, supported schema and algorithms, canonical representation, declared hashes/sizes, unique artifact IDs, valid URIs/references, and Llama/GGUF/tokenizer/backend compatibility.
- [ ] Implement a trusted-publisher key source suitable for the local milestone and verify the publisher signature over every semantic manifest field.
- [ ] Create golden manifest vectors and negative tests for malformed, oversized, unsigned, altered, unknown-key, unsupported-version, and incompatible manifests. Ensure each failure maps to a stable typed error.

**Evidence:** golden/negative contract tests and a documented trusted-key fixture.

### 4. Implement verified acquisition and content-addressed local caching

- [ ] Resolve only approved versioned manifest references; reject arbitrary model-weight URLs and unsafe redirects/schemes.
- [ ] Fetch or provision the manifest, validate it, and record safe provenance/audit metadata without secrets or raw prompts.
- [ ] Download or import the declared GGUF and tokenizer into a content-addressed cache using bounded size limits, temporary staging, hash/size verification, and atomic promotion.
- [ ] Implement cache lookup, metadata, leases, cleanup/eviction policy, and safe failure recovery sufficient for one local active model. Expose inspected verification/provenance state through the application layer.
- [ ] Exercise cache hit, interrupted/failed acquisition, hash/size mismatch, disallowed source, and invalid signature paths hermetically with local test servers/files; reserve the real fixture for provisioned integration testing.

**Evidence:** acquisition/cache integration tests, cache metadata inspection, and typed negative-error tests.

### 5. Deliver the GGUF/Llama backend and correct generation behavior

- [ ] Integrate the selected maintained llama.cpp-compatible Rust adapter as a concrete adapter behind `ModelBackend`; pin and document its version/features according to dependency policy.
- [ ] Load only manifest-verified GGUF/tokenizer artifacts and enforce the declared architecture, tokenizer, context, and runtime compatibility before initialization.
- [ ] Implement prompt tokenization, context/cache lifecycle, decoding, and end-of-generation handling consistent with the selected model/tokenizer.
- [ ] Implement deterministic seeded generation and the advertised temperature/top-p policy, including explicit validation of parameter bounds and a deterministic mode that is stable for the fixed backend/version/fixture.
- [ ] Add adapter unit/contract tests for compatibility and sampling validation, plus a provisioned model-backed reference-output test that verifies the pinned token stream.

**Evidence:** backend compatibility tests, sampling tests, and the fixed-fixture reference token-stream test.

### 6. Expose one application workflow through CLI and local API

- [ ] Implement the application-level local generation workflow: validate request and policy, resolve/verify/cache/load the model, tokenize, generate, and return/stream tokens with a request/session identifier.
- [ ] Make `synapseflow run --model <manifest-reference>` invoke that workflow; validate input without `assert!`, return non-zero exits with stable error codes, support `--seed`, `--temperature`, `--top-p`, and explicit safe output destinations.
- [ ] Implement the minimal local API endpoint/service over the same workflow with typed error translation, bounded request size, deadline propagation, and token streaming appropriate to the selected local API framework.
- [ ] Add CLI/API integration tests with fake adapters for hermetic coverage and with the verified fixture in the provisioned integration environment. Verify that both surfaces yield the same seeded token stream.

**Evidence:** CLI/API integration tests, streamed-output test, typed-error/exit-code tests, and fixture-backed parity test.

### 7. Complete milestone validation, documentation, and operational handoff

- [ ] Run and record the full [development quality gate](docs/development.md#quality-gate) on Tier-1 platforms; resolve all formatting, compilation, Clippy, test, dependency-policy, audit, secret-scan, and SBOM failures.
- [ ] Run the provisioned real-fixture acceptance suite and record the manifest hash, adapter version, hardware/OS, seed, generation parameters, expected token stream hash, latency, throughput, and memory measurement method.
- [ ] Update the README, onboarding, CLI, model-management, compatibility, release, and operations documentation to describe the supported local workflow, fixture provisioning, cache location/inspection, trusted publisher configuration, failure handling, and known milestone boundaries.
- [ ] Confirm that no documentation promises sharding, remote workers, discovery, incentives, TEEs, or unsupported formats/backends as delivered functionality.

**Evidence:** CI links/logs, acceptance record, benchmark record, and documentation review.

### 8. Update the implementation gap

- [ ] Refresh [docs/implementation-gap.md](docs/implementation-gap.md) only after all preceding steps are evidenced: remove or narrow the completed model-acquisition, initial-local-inference, domain-contract/port, local API, and test-coverage gaps; retain future-milestone sharding, transport, remote-worker, security, and incentive gaps with their current scope.
- [ ] Update the evidence basis and date, and move any durable completion history to the project progress tracker if it is restored; do not duplicate this checklist’s history in stable design documents.

**Evidence:** reviewed implementation-gap diff accurately matching the delivered code, tests, and documentation.

## Milestone sign-off

- [ ] All completion criteria are met.
- [ ] Step 8 is complete and the remaining gap document accurately represents post-milestone work.
- [ ] The roadmap milestone can be marked complete.
