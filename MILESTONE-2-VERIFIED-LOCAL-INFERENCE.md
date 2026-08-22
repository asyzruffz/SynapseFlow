# Milestone 2 — Verified local inference

**Status:** Step 6 complete
**Roadmap milestone:** [Verified local inference](docs/roadmap.md#2-verified-local-inference)
**Last updated:** 2026-08-22

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

- [x] (2026-08-22: [verified local inference contract](docs/verified-local-inference.md)) Document the exact supported compatibility tuple: GGUF, Llama architecture, llama.cpp-compatible Rust adapter, Tier-1 CPU platforms, supported quantization(s), tokenizer behavior, context limits, and sampling semantics.
- [x] (2026-08-22: [verified local inference contract](docs/verified-local-inference.md)) Define the fixed integration fixture’s approved manifest identity, publisher key ID, licence/provenance, artifact/tokenizer hashes and sizes, test prompt, named seed, and generation parameters. The literal signed-manifest hash and expected token stream are deliberately generated as Step 3/Step 5 evidence after canonical signing and backend validation, rather than guessed before either exists.
- [x] (2026-08-22: [stable error contract](docs/verified-local-inference.md#stable-error-contract)) Define stable public error codes/types and error mapping for reference parsing, manifest/schema validation, trust/signature failure, artifact integrity/cache failure, compatibility failure, tokenizer failure, backend initialization, and generation failure.
- [x] (2026-08-22: [ADR 0004](docs/adr/0004-verified-local-inference-contract.md)) Add or amend an ADR only if the selected adapter, cryptographic scheme, canonical manifest representation, or fixture source requires a durable decision beyond [ADR 0003](docs/adr/0003-initial-model-backend-scope.md).

**Evidence:** compatibility/fixture specification, typed-error contract, and ADR 0004. Implementation tests for the contract are mandatory evidence in Steps 3–6.

### 2. Restructure the workspace around the target architecture

- [x] (2026-08-22: [active workspace architecture](docs/workspace-architecture.md)) Introduce or rename crates/modules to make the architecture explicit: `synapseflow-domain`, `synapseflow-ports`, `synapseflow-application`, `synapseflow-adapter-in-memory`, and the CLI/local-node application boundaries.
- [x] (2026-08-22: [`domain`](domain/src/lib.rs)) Move framework-independent identifiers, model/manifest declarations, compatibility rules, generation policy, request/result types, and typed domain errors into domain.
- [x] (2026-08-22: [`ports`](ports/src/lib.rs)) Define object-safe ports for manifest resolution, artifact/cache storage, model backend/tokenization/generation, audit/telemetry, clock, peer directory, and transport; keep ports independent of Tokio, HTTP, QUIC, databases, Candle, and llama.cpp.
- [x] (2026-08-22: [`GenerationService`](application/src/lib.rs)) Put the generation use case in the application layer and make the CLI/node call it through ports. Exclude the legacy `core`/`inference` and Candle path from the active workspace so they cannot be used as supported behavior.
- [x] (2026-08-22: [`GenerationService` tests](application/src/lib.rs)) Add deterministic unit tests that demonstrate the domain and application layers can run with in-memory test adapters; use the active workspace graph as the dependency-direction check.

**Evidence:** workspace dependency graph review, domain/application unit tests, and strict all-feature build without warnings.

### 3. Implement versioned manifest parsing and trust verification

- [x] (2026-08-22: [`ModelManifest` parser](domain/src/model/manifest_parser/mod.rs)) Implement the canonical manifest schema from [the protocol](docs/protocol.md), including schema version, immutable model identity/version, GGUF/Llama compatibility declaration, tokenizer and artifact declarations, publisher key identifier, licence/provenance, and signature envelope.
- [x] (2026-08-22: [`ModelManifest` parser](domain/src/model/manifest_parser/mod.rs)) Validate manifest invariants before use: bounded input, supported schema and algorithms, canonical representation, declared hashes/sizes, unique artifact IDs, valid URIs/references, and Llama/GGUF/tokenizer/backend compatibility.
- [x] (2026-08-22: [`TrustStore`](domain/src/model/trust/mod.rs)) Implement a trusted-publisher key source suitable for the local milestone and verify the publisher signature over every semantic manifest field.
- [x] (2026-08-22: [`manifest_parser` tests](domain/src/model/manifest_parser/tests.rs)) Create golden manifest vectors and negative tests for malformed, oversized, unsigned, altered, unknown-key, unsupported-version, and incompatible manifests. Ensure each failure maps to a stable typed error.

**Evidence:** eight golden/negative contract tests in `synapseflow-domain`, the documented local trust-store fixture, the 2026-08-22 passing format/check/Clippy/test suite, and user-reported clean `cargo deny check` and `cargo audit` results.

### 4. Implement verified acquisition and content-addressed local caching

- [x] (2026-08-22: [`ProvisionedManifestRegistry`](adapters/local-cache/src/manifest_registry/registry.rs)) Resolve only approved versioned manifest references; reject arbitrary model-weight URLs and unsafe redirects/schemes.
- [x] (2026-08-22: [`ModelAcquisitionService`](application/src/model_acquisition_service.rs)) Fetch or provision the manifest, validate it, and record safe provenance/audit metadata without secrets or raw prompts.
- [x] (2026-08-22: [`ContentAddressedArtifactStore`](adapters/local-cache/src/artifact_cache/cache.rs)) Download or import the declared GGUF and tokenizer into a content-addressed cache using bounded size limits, temporary staging, hash/size verification, and atomic promotion.
- [x] (2026-08-22: [`ContentAddressedArtifactStore`](adapters/local-cache/src/artifact_cache/cache.rs)) Implement cache lookup, metadata, leases, cleanup/eviction policy, and safe failure recovery sufficient for one local active model. Expose inspected verification/provenance state through the application layer.
- [x] (2026-08-22: [local cache tests](adapters/local-cache/src/artifact_cache/tests.rs)) Exercise cache hit, interrupted/failed acquisition, hash/size mismatch, disallowed source, and invalid signature paths hermetically with local test servers/files; reserve the real fixture for provisioned integration testing.

**Evidence:** six hermetic `synapseflow-adapter-local-cache` tests, one application inspection/audit test, the passing 2026-08-22 format/check/Clippy/test suite, and user-reported clean `cargo deny check` and `cargo audit` results.

### 5. Deliver the GGUF/Llama backend and correct generation behavior

- [x] (2026-08-22: [`synapseflow-adapter-llama-cpp`](adapters/llama-cpp/src/lib.rs)) Integrate the selected maintained llama.cpp-compatible Rust adapter as a concrete adapter behind `ModelBackend`; pin and document its version/features according to dependency policy.
- [x] (2026-08-22: [`LlamaCppBackend`](adapters/llama-cpp/src/runtime.rs)) Load only manifest-verified GGUF/tokenizer artifacts and enforce the declared architecture, tokenizer, context, and runtime compatibility before initialization.
- [x] (2026-08-22: [`LlamaCppBackend`](adapters/llama-cpp/src/runtime.rs)) Implement prompt tokenization, context/cache lifecycle, decoding, and end-of-generation handling consistent with the selected model/tokenizer.
- [x] (2026-08-22: [`LlamaCppBackend`](adapters/llama-cpp/src/runtime.rs)) Implement deterministic seeded generation and the advertised temperature/top-p policy, including explicit validation of parameter bounds and a deterministic mode that is stable for the fixed backend/version/fixture.
- [x] (2026-08-22: user-attested provisioned fixture acceptance on both Tier-1 platforms) Add adapter unit/contract tests for compatibility and sampling validation, plus a provisioned model-backed reference-output test that verifies the pinned token stream.

**Evidence:** backend compatibility tests, sampling tests, and the fixed-fixture reference token-stream test.

**Current status (2026-08-22):** `synapseflow-adapter-llama-cpp` contains the pinned CPU-only `llama-cpp-2 =0.1.154` runtime feature, verified-artifact loading, embedded-tokenizer prompt handling, bounded context validation, end-of-generation handling, and seeded `top_p`/temperature sampling. `cargo check --workspace --all-targets --all-features --locked` and strict all-feature Clippy complete with `LIBCLANG_PATH` configured; the default workspace suite passes 24 tests, including three model-free adapter contracts, two artifact-port invariants, and the fixture-provisioner self-verification test. User-reported `cargo test -p synapseflow-adapter-llama-cpp --features runtime --locked` passed four tests on 2026-08-22. The user attested that the provisioned fixture reference-output evidence was accepted on both Tier-1 platforms; the vector remains external to Git. [`synapseflow-fixture-provisioner`](tools/fixture-provisioner/src/main.rs) creates and self-verifies canonical signed fixture manifests, and [`fixture_reference`](adapters/llama-cpp/src/tests/fixture_reference/mod.rs) remains available for future fixture revalidation.

### 6. Expose one application workflow through CLI and local API

- [x] (2026-08-22: [`LocalNode`](node/src/local_node.rs), [`GenerationService`](application/src/generation_service.rs)) Implement the application-level local generation workflow: validate request and policy, resolve/verify/cache/load the model, tokenize, generate, and return/stream tokens with a request/session identifier.
- [x] (2026-08-22: [CLI runner](cli/src/runner.rs), [CLI tests](cli/tests/cli_help.rs)) Make `synapseflow run --model <manifest-reference>` invoke that workflow; validate input without `assert!`, return non-zero exits with stable error codes, support `--seed`, `--temperature`, `--top-p`, opt-in token-vector JSON, and explicit safe output destinations.
- [x] (2026-08-22: [local HTTP API](node/src/http/mod.rs), [API tests](node/src/tests/http_api_tests.rs)) Implement the minimal local API endpoint/service over the same workflow with typed error translation, bounded request size, deadline propagation, and token streaming appropriate to the selected local API framework.
- [x] (2026-08-22: [CLI output test](cli/src/tests/output_tests.rs), [API parity tests](node/src/tests/http_api_tests.rs)) Add CLI/API integration tests with fake adapters for hermetic coverage. Verify that the JSON and SSE API surfaces yield the same seeded token stream.
- [x] (2026-08-22: provisioned Windows CPU fixture run; external vector retained outside Git) Run the CLI and API against the provisioned verified fixture and compare each resulting token-ID/text stream with the accepted reference vector. The user-reported CLI acceptance and the locally executed CLI `--json`, API JSON, and API SSE responses emitted the same complete 16-token ID/text stream for the documented reference, prompt, policy, and seed.

**Evidence:** strict all-feature Clippy and the full workspace suite passed on 2026-08-22. The suite includes request/session workflow, expired-deadline, explicit no-overwrite output, CLI typed-error/exit-code, bounded API request, typed API error, and JSON/SSE parity tests. The user reported clean `cargo deny check` and `cargo audit` results and an accepted provisioned CLI `--json` token-ID/text vector. A local provisioned Windows CPU run then confirmed that CLI JSON, API JSON, and API SSE emitted the same complete 16-token ID/text stream. The real fixture and reference vector remain outside Git.

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
