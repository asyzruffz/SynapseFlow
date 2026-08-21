# SynapseFlow progress tracker

This is the active progress tracker for the delivery roadmap. It starts with the **Foundation** milestone and is updated only with completed work and verifiable evidence. Do not mark a step complete until every completion condition in that step is met.

## Foundation milestone

**Goal:** establish a reproducible, reviewable, and releasable Rust project foundation.

**Completion condition:** a clean clone passes the required quality gate on every supported platform without developer-specific state.

### Progress log

| Date | Step | Status/evidence |
|---|---:|---|
| 2026-08-21 | 1 | Accepted ADRs 0001–0003 define the Tier-1 platforms, MSRV, release/compatibility policy, and initial GGUF/Llama backend scope. |
| 2026-08-21 | 2 | `rust-toolchain.toml` pins Rust 1.89.0 with `rustfmt` and Clippy; Rustup selected it and `cargo fmt --all -- --check` passed. |
| 2026-08-21 | 3 | Workspace warning/panic lint policy, typed core/inference errors, and non-panicking CLI validation added; strict all-features Clippy and tests passed. |
| 2026-08-21 | 4 | Cargo metadata now inherits consistent workspace fields; dependency policy and `deny.toml` added; a fresh local `CARGO_HOME` passed locked all-features dependency resolution and build. |
| 2026-08-21 | 5 | MIT licence and contributor, review, security, compatibility, release, changelog, and SBOM policies added and cross-linked. |
| 2026-08-21 | 6 | Skipped at the user's direction; no model-artifact-policy work was performed in this step. |
| 2026-08-21 | 7 | GitHub Actions completed successfully: native Windows/Linux quality checks, Rust 1.87 MSRV validation, dependency policy/audit, secret scanning, and SPDX SBOM publication all ran from the CI workflow. |
| 2026-08-21 | 8 | Hermetic core model-loader, inference backend-selection, and CLI help smoke tests added; generated-fixture and deterministic-input rules documented. Local and GitHub Actions format, strict Clippy, and all-features tests passed. |
| 2026-08-21 | 9 | Contributor onboarding and operational runbooks document clean-clone setup, fixture acquisition, quality gates, cache cleanup, and dependency, artifact, CI, security, and release response. Markdown links verified. |

### Ordered steps

- [x] **1. Define the support and release baseline.**
  - Decide the supported Rust MSRV, Windows and Linux targets, supported CPU/GPU scope, and release/versioning convention.
  - Record the decisions in ADRs, including the initial model/backend compatibility scope and the policy for breaking changes.
  - Completion: the support matrix and versioning policy are reviewed, versioned, and referenced by the development documentation. **Completed 2026-08-21:** [ADR 0001](docs/adr/0001-supported-platforms-and-toolchain.md), [ADR 0002](docs/adr/0002-release-and-compatibility-policy.md), and [ADR 0003](docs/adr/0003-initial-model-backend-scope.md) are linked from [Development](docs/development.md) and [Model management](docs/model-management.md).

- [x] **2. Pin the development toolchain.**
  - Add `rust-toolchain.toml` for the selected Rust version and required components (`rustfmt`, Clippy, and any required targets).
  - Add `rustfmt.toml` only where the project needs non-default formatting rules; otherwise document use of Rust defaults.
  - Completion: a fresh environment selects the pinned toolchain without relying on a locally installed default. **Completed 2026-08-21:** [`rust-toolchain.toml`](rust-toolchain.toml) pins Rust 1.89.0 with `rustfmt` and Clippy; no `rustfmt.toml` is needed because the project adopts Rust's standard formatting rules, and Tier-1 CI runs natively rather than through local cross-compilation. Rustup selected the pinned override and `cargo fmt --all -- --check` passed.

- [x] **3. Establish workspace lint and error conventions.**
  - Define workspace lint policy and make the strict Clippy gate actionable by resolving existing warnings rather than suppressing them globally.
  - Introduce the typed public-error boundary: libraries expose `thiserror` error types; application/binary boundaries may add `anyhow` context.
  - Document rules for user, artifact, and network input: no panics/assertions; all decoding, decompression, allocation, and buffering are bounded.
  - Completion: `cargo clippy --workspace --all-targets --all-features -- -D warnings` is a supported local and CI command. **Completed 2026-08-21:** workspace lints deny Rust warnings and Clippy `dbg!`, `todo!`, and `unwrap()` use; every crate opts into the policy. `synapseflow-core` and `synapseflow-inference` now expose typed errors, and CLI input/output failures return errors rather than panicking. Strict all-features Clippy and tests passed.

- [x] **4. Make dependency and Cargo metadata reproducible.**
  - Reconcile workspace/package metadata: description, authors, repository, licence, readme, categories, and version inheritance.
  - Select and document a dependency policy, including licence/security review, allowed sources, lockfile update rules, and the use of `cargo deny` and `cargo audit`.
  - Resolve the all-features dependency/cache failure using a writable, reproducible Cargo cache configuration; do not depend on a developer-specific global registry path.
  - Completion: Cargo metadata and the lockfile resolve reproducibly from an isolated writable Cargo home. Tier-1 clean-clone verification is performed by step 10. **Completed 2026-08-21:** all crates inherit the workspace package metadata and MSRV; obsolete optional Candle 0.1 dependencies were removed; [dependency management](docs/dependency-management.md), `.cargo/config.toml`, and `deny.toml` define the locked source/licence/audit/cache policy. A fresh workspace-local `CARGO_HOME` completed `cargo fetch --locked` and `cargo check --workspace --all-targets --all-features --locked`.

- [x] **5. Complete project governance and release artifacts.**
  - Add the definitive `LICENSE` and make every Cargo/readme reference agree with it.
  - Add `CONTRIBUTING.md`, a code-review policy, security reporting guidance, release process, changelog format, compatibility statement, and SBOM publication process.
  - Completion: a contributor can follow repository documentation to make, review, and release a change without unwritten process requirements. **Completed 2026-08-21:** [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), [CHANGELOG.md](CHANGELOG.md), [code-review policy](docs/code-review-policy.md), [compatibility statement](docs/compatibility.md), and [release process](docs/release-process.md) define the required workflow, including SBOM publication and private vulnerability reporting prerequisites.

- [-] **6. Enforce the model-artifact policy — skipped by user direction (2026-08-21).**
  - Keep development/test models, tokenizers, benchmark data, credentials, prompts, and activation dumps out of Git.
  - Define the approved development-fixture acquisition path and the supported remote model-source policy: manifest reference, TLS/authentication, provenance, signed manifest, content hash, and content-addressed cache.
  - Add automated checks that reject prohibited artifacts/secrets and validate the documented ignore policy.
  - Completion: the repository contains no unapproved model artifacts, and a new developer can obtain approved test fixtures without committing them.

- [x] **7. Build the cross-platform CI pipeline.**
  - Add CI for the pinned toolchain on the selected Windows and Linux targets.
  - Configure caches only in runner-writable paths and make cache misses safe and reproducible.
  - Run formatting, check, strict Clippy, tests, dependency-policy, audit, secret scan, and licence/SBOM jobs; publish their artifacts/results.
  - Completion: required checks run automatically on pull requests and block merges on failure. **Completed 2026-08-21:** [CI workflow](.github/workflows/ci.yml) completed successfully on GitHub Actions with native Windows/Linux quality checks, Rust 1.87 MSRV validation, dependency-policy/audit, secret scanning, and an uploaded SPDX SBOM.

- [x] **8. Establish the initial automated-test baseline.**
  - Replace no-op tests with meaningful positive and negative tests for the currently exposed public behavior.
  - Add a minimal integration/smoke-test harness that runs without large local model weights or developer state.
  - Define test-fixture provenance and deterministic seed/input policy.
  - Completion: the test suite proves real behavior, has no placeholder-success tests, and runs in CI from a clean clone. **Completed 2026-08-21:** hermetic model-loader and unsupported-backend tests, plus the CLI `--help` smoke test, pass locally and in GitHub Actions without a model artifact or developer-specific state.

- [x] **9. Publish developer onboarding and operational runbooks.**
  - Document setup, supported platforms, toolchain bootstrap, fixture acquisition, quality-gate commands, troubleshooting, and cache cleanup.
  - Add runbooks for dependency/registry failures, artifact verification failures, CI failures, security reporting, and release rollback.
  - Completion: an unfamiliar contributor can set up and validate the repository using only versioned documentation. **Completed 2026-08-21:** [Contributor onboarding](docs/onboarding.md) and [operational runbooks](docs/operations-runbooks.md) cover the required setup, quality gate, fixture, cache, troubleshooting, security, and rollback workflows.

- [ ] **10. Perform the clean-clone Foundation verification.**
  - Verify a fresh clone on every supported platform with no pre-populated Cargo cache, model artifacts, credentials, or untracked setup files.
  - Run and record the complete quality gate:

    ```text
    cargo fmt --all -- --check
    cargo check --workspace --all-targets --all-features
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-targets --all-features
    cargo deny check
    cargo audit
    ```

  - Completion: all commands succeed and evidence links are recorded in the progress log.

- [ ] **11. Update the implementation gap.**
  - Remove or narrow the Foundation and quality rows in [`docs/implementation-gap.md`](docs/implementation-gap.md) using the evidence from the completed steps.
  - Keep only remaining design-versus-code disparities; do not add implementation-status notes to the stable design documents.
  - Completion: the implementation gap accurately reflects the post-Foundation state.
