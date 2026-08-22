# Contributor onboarding

This guide takes a contributor from a clean clone to the same local quality gate used for pull requests. It requires no model artifact, credential, or pre-populated dependency cache.

## Prerequisites

- A Tier-1 Windows or Linux environment as defined in [ADR 0001](adr/0001-supported-platforms-and-toolchain.md).
- Git and an internet connection to the approved Rust and crates.io sources.
- Rustup. The repository selects its pinned toolchain from [`rust-toolchain.toml`](../rust-toolchain.toml), including `rustfmt` and Clippy.

## Clone and bootstrap

```text
git clone https://github.com/asyzruffz/SynapseFlow.git
cd SynapseFlow
rustc --version
cargo fetch --locked
```

`rustc --version` confirms that Rustup selected the repository toolchain. Do not replace the pinned toolchain with a system default. `cargo fetch --locked` downloads exactly the dependency graph recorded in `Cargo.lock`.

If the default Cargo home is unavailable or an isolated verification is required, set `CARGO_HOME` before invoking Cargo:

```powershell
$env:CARGO_HOME = Join-Path (Get-Location) '.cache/cargo'
cargo fetch --locked
```

The `.cache` directory is ignored by Git. See [dependency management](dependency-management.md) and the registry-failure runbook before changing registry configuration or credentials.

## Validate a change

Run the full local gate before opening a pull request:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo deny check
cargo audit
```

The tests are hermetic and do not need model weights. The CLI help path is also available without a model:

```text
cargo run -p synapseflow-cli -- --help
```

## Model and integration fixtures

Default development and CI validation never download a model. The real-fixture integration environment provisions a signed manifest and its matching GGUF outside Git, then passes the immutable reference, manifest path, artifact path, cache directory, and public key explicitly to the CLI or node. Follow [the local CLI/API guide](cli.md) and the fixture procedure in [the verified-local contract](verified-local-inference.md).

### Planned model-management workflow

The following commands illustrate the intended onboarding workflow after the remote model-management milestone is delivered; they are **not available in Milestone 2**:

```text
synapseflow models pull --manifest registry://models/<name>@sha256:<manifest-hash>
synapseflow models inspect --manifest registry://models/<name>@sha256:<manifest-hash>
```

Milestone 2 provides the foundation for this path: immutable manifest references, publisher-signature verification, compatibility validation, content-addressed local caching, and safe provenance inspection through the application layer. The future `pull` command will add policy-controlled remote acquisition and the future `inspect` command will present the verified cache/provenance state without exposing sensitive filesystem or credential data.

Do not substitute a raw weight URL, copy a model into source control, or bypass manifest/signature/hash verification. The cache and provenance rules are in [model management](model-management.md).

## Before requesting review

Read [CONTRIBUTING.md](../CONTRIBUTING.md), the [code-review policy](code-review-policy.md), and relevant [ADRs](adr/README.md). Include the problem statement, test evidence, risk and rollback notes, documentation/compatibility impact, and any required ADR in the pull request.

For a failed command or operational incident, use [operational runbooks](operations-runbooks.md). Report vulnerabilities privately under [SECURITY.md](../SECURITY.md).
