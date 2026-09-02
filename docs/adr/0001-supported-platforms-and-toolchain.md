# ADR 0001: Supported platforms and Rust toolchain

**Status:** Accepted

## Context

SynapseFlow needs a reproducible baseline for development, CI, releases, and support. The project targets a networked inference node that should be operable on common developer and server environments without coupling its core contracts to a GPU vendor or one operating system.

## Decision

- The minimum supported Rust version (MSRV) is **Rust 1.87.0**.
- The Tier-1 build and test targets are:
  - `x86_64-pc-windows-msvc`
  - `x86_64-unknown-linux-gnu`
- Tier-1 inference support is **64-bit x86 CPU execution**. CPU support is the compatibility baseline for correctness, local development, and the first node release.
- GPU, accelerator, ARM, macOS, and alternate Linux-libc support are adapter-specific experimental targets until a later ADR promotes them. Experimental targets do not alter the Tier-1 release gate.
- The repository pins a specific stable toolchain in `rust-toolchain.toml`. The pinned toolchain may be newer than the MSRV, but every supported dependency and public crate must remain compatible with the MSRV.
- CI runs the quality gate on both Tier-1 targets. MSRV validation is an additional CI job once the toolchain pin is introduced.

## Consequences

- Dependencies requiring Rust newer than 1.87.0 cannot be introduced without an ADR that raises the MSRV and documents the migration impact.
- Backend adapters must declare their platform/capability support. A GPU adapter cannot silently become required for core compilation or correctness tests.
- Release artifacts and documentation target the two Tier-1 triples first. Other targets may be offered on a best-effort basis only when explicitly labelled experimental.

## Superseding conditions

Supersede this ADR to add a Tier-1 platform, change the CPU baseline, or raise the MSRV after a dependency/security/support review and successful CI validation on every affected target.
