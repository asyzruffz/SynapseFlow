# ADR 0004: Verified local inference contract

**Status:** Accepted

## Context

[ADR 0003](0003-initial-model-backend-scope.md) restricts the initial product to GGUF/Llama on Tier-1 CPUs through a llama.cpp-compatible Rust adapter. It intentionally did not choose a concrete adapter, fixture identity, manifest canonicalization, or signature algorithm. Those omissions leave the first acceptance test and public compatibility promise ambiguous.

The TinyLlama GGUF fixture has a known artifact hash and upstream provenance, but it is not a source-controlled test asset and has no SynapseFlow manifest or trusted publisher signature. Verified local inference requires a precise external fixture and a manifest profile before acquisition and backend implementation.

## Decision

- Pin the initial runtime adapter to crates.io package `llama-cpp-2` **`=0.1.154`**, CPU-only, with its bundled llama.cpp integration. The adapter remains behind a `ModelBackend` port and must not leak its unsafe or runtime-specific API into domain/application contracts.
- Support only the compatibility tuple defined in [the verified local inference contract](../verified-local-inference.md): TinyLlama 1.1B Chat v0.3, one non-sharded `Q5_K_M` GGUF artifact, the embedded `llama` tokenizer, and the Tier-1 x86_64 CPU targets.
- Use the externally provisioned `synapseflow-verified-local-tinyllama-q5km-v1` fixture. Its publisher, pinned upstream revision, Apache-2.0 licence, byte length, and SHA-256 are declared in the contract; a changed artifact requires a new fixture manifest and vector.
- Canonicalize manifests with RFC 8785 JSON Canonicalization Scheme and authenticate them with detached Ed25519 signatures over their semantic fields. Use lower-case SHA-256 content identities and unpadded base64url signature encoding.
- Keep the fixture-signing key dedicated to non-production integration testing. Production trust stores exclude it. The canonical signed manifest and immutable expected token vector are generated through the documented procedure, not guessed or checked into the default test path.

## Consequences

- Build/CI provisioning for the backend must supply the pinned adapter's native prerequisites and prove the exact locked dependency graph on both Tier-1 platforms before this adapter is promoted as supported.
- The default quality gate stays hermetic and model-free. The model-backed acceptance test runs only in an explicit fixture-provisioning environment.
- Safetensors/Candle, other llama.cpp-compatible model variants, alternate GGUF quantizations, external tokenizer files, GPU execution, and remote-model URLs remain unsupported. Their addition requires an ADR update or successor and complete reference-output evidence.
- The project gains an immutable fixture identity and signature profile that the registry, cache, CLI, API, and backend can all share.

## Superseding conditions

Supersede this ADR to change the selected adapter/version family, fixture source or model tuple, signature/canonicalization scheme, Tier-1 backend platform support, or deterministic-vector compatibility rule after supply-chain, security, compatibility, and reference-output review.
