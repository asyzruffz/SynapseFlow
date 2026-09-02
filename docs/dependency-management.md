# Dependency management

> **Source of truth.** This policy governs dependency selection and review.
> Change it only through an explicit supply-chain policy redesign.

## Source and lockfile policy

SynapseFlow depends on crates from the crates.io registry unless an ADR approves another source. Git dependencies and unknown registries are denied by [`deny.toml`](../deny.toml). Every dependency change updates and commits `Cargo.lock`; release and CI builds use `--locked` so a resolver change cannot silently alter the build.

Dependencies must be compatible with the Rust 1.87 MSRV and both Tier-1 targets defined in [ADR 0001](adr/0001-supported-platforms-and-toolchain.md). Adding, upgrading, or removing a direct dependency requires a documented reason, licence review, vulnerability review, and test evidence. A dependency that expands the supported model/backend surface also requires an ADR.

## Security and licence policy

`cargo deny check` evaluates advisory, licence, duplicate-version, wildcard, and source policy. The allowlist in `deny.toml` is the authoritative set of permitted licences. Every dependency, including an internal path dependency, declares a non-wildcard version. A licence or source exception requires an ADR or a reviewed exception file with the dependency, version, reason, owner, and expiry date. An advisory exception is a versioned `deny.toml` entry containing that same information; unused exception entries fail the policy check so they are removed once resolved.

`cargo audit` checks the locked dependency graph against RustSec advisories. Its repository configuration is [`.cargo/audit.toml`](../.cargo/audit.toml). Any ignored advisory must be mirrored in `deny.toml` and carry the same documented risk acceptance, mitigation, owner, and expiry date. Neither tool replaces code review or a release SBOM.

## Reproducible Cargo home

Cargo stores registry indexes, downloaded crates, git checkouts, installed subcommands, and global configuration under `CARGO_HOME`. Set `CARGO_HOME` **before** invoking Cargo when the default home is not writable or when a fresh, isolated dependency cache is required. Cargo configuration is read from that home, so a repository `.cargo/config.toml` cannot relocate it after startup.

Use a workspace-local cache for an isolated development verification and keep it untracked:

```powershell
$env:CARGO_HOME = Join-Path (Get-Location) '.cache/cargo'
cargo fetch --locked
cargo check --workspace --all-targets --all-features --locked
```

CI sets `CARGO_HOME` to a writable runner-local directory before every Cargo command and keys its cache on the operating system, `rust-toolchain.toml`, `Cargo.lock`, and `deny.toml`. Credentials belong in the platform secret store or the Cargo home, never repository configuration.

## Required commands

```text
cargo fetch --locked
cargo check --workspace --all-targets --all-features --locked
cargo deny check
cargo audit
```

The dependency graph is reviewed when these commands pass on both Tier-1 platforms.
