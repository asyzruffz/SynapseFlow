# Dependency management

## Source and lockfile policy

SynapseFlow depends on crates from the crates.io registry unless an ADR approves another source. Git dependencies and unknown registries are denied by [`deny.toml`](../deny.toml). Every dependency change updates and commits `Cargo.lock`; release and CI builds use `--locked` so a resolver change cannot silently alter the build.

Dependencies must be compatible with the Rust 1.87 MSRV and both Tier-1 targets defined in [ADR 0001](adr/0001-supported-platforms-and-toolchain.md). Adding, upgrading, or removing a direct dependency requires a documented reason, licence review, vulnerability review, and test evidence. A dependency that expands the supported model/backend surface also requires an ADR.

## Security and licence policy

`cargo deny check` evaluates advisory, licence, duplicate-version, wildcard, and source policy. The allowlist in `deny.toml` is the authoritative set of permitted licences. A licence or source exception requires an ADR or a reviewed exception file with the dependency, version, reason, owner, and expiry date.

`cargo audit` checks the locked dependency graph against RustSec advisories. An ignored advisory requires a documented risk acceptance, mitigation, owner, and expiry date. Neither tool replaces code review or a release SBOM.

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

The dependency graph is reviewed when these commands pass on both Tier-1 platforms. The clean-clone Foundation verification records the final evidence.
