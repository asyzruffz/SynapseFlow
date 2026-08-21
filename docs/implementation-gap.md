# Implementation gap

This is a temporary migration tracker. It compares the design documents to the repository state observed on 2026-08-21. Remove completed rows and delete this document when the implementation conforms to the documented design.

## Gap summary

| Design area | Observed state | Migration outcome |
|---|---|---|
| Model management | Local loader discovers safetensors only; the development model is GGUF; tokenizer path resolves to `config.json`. | Implement manifest-driven local/remote acquisition, compatible backend selection, tokenizer discovery, verification, and cache. |
| Local inference | Candle Llama path is experimental; LlamaCpp is `todo!`; sampling settings are unused. | Deliver one tested model/backend vertical slice with correct cache and sampling behavior. |
| Domain contracts | No implemented manifest, shard index, execution plan, session model, or typed public errors. | Introduce versioned domain/port contracts before transport and scheduling work. |
| Shard execution | Runtime executor and kernels are stubs; the existing shard module is not exported and is incomplete. | Implement/test a deterministic subgraph executor and loopback two-shard baseline. |
| Transport | Frame is a private serde sketch; codec, QUIC, backpressure, retries, and discovery are stubs. | Implement the protocol schema and loopback codec before QUIC transport. |
| Node/API/security | API, coordinator, security, storage, and incentives are skeletons. | Add only the components required by the roadmap, beginning with operable node/API and manifest trust. |
| Quality | Default tests pass but provide two no-op core tests and zero tests in the other crates; strict Clippy fails on warnings. | Enforce the development quality gate and add unit, contract, integration, fuzz, performance, and security tests. |

## Evidence snapshot

- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --all-targets` passed with five warnings.
- `cargo test --workspace --all-targets` passed, with only two no-op tests.
- `cargo clippy --workspace --all-targets -- -D warnings` failed on unused imports/dead code.
- `cargo check --workspace --all-features` was blocked by access denied while Cargo unpacked optional `candle 0.1.0` into the configured global registry cache.

## Exit rule

Each migration PR links to the design contract it implements, removes or narrows the applicable row, and adds the required automated evidence. Do not keep historical implementation facts in the design documents; this file is their single home.
