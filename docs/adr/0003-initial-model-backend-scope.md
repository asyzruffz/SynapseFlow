# ADR 0003: Initial model and backend scope

**Status:** Accepted  
**Date:** 2026-08-21

## Context

SynapseFlow must establish a reliable single-node baseline before it can validate shard routing, frames, retries, or remote workers. Supporting multiple model formats and inference backends at once would multiply unverified code paths. The assessment identified a mismatch between a GGUF development model and a safetensors-only discovery path.

## Decision

- The first supported model scope is **GGUF weights for the Llama model family** on the Tier-1 CPU platforms.
- The initial runtime is one maintained **llama.cpp-compatible Rust adapter** behind the `ModelBackend` port. The adapter owns GGUF parsing, tokenizer compatibility, context/cache behavior, and CPU execution details.
- A model is selected by a verified manifest reference. The manifest declares the exact GGUF artifact, architecture, tokenizer behavior, content hash, publisher signature, licence, and runtime compatibility. Local development fixtures use the same manifest and verification path.
- Generation uses a documented deterministic seed mode and implements the advertised temperature/top-p policy before it is exposed as a supported interface.
- Safetensors/Candle, other architectures, GPU backends, and additional quantization formats are deferred adapters. They are added only with an ADR, compatibility matrix entry, reference-output tests, model provenance, and Tier-1 quality-gate coverage.

## Consequences

- The first local-inference milestone has one format/backend/tokenizer contract to test end-to-end.
- Existing generic safetensors-only loading paths are not the supported product path and must be replaced or isolated behind a future adapter.
- Backend code remains outside domain and application contracts, so later formats do not force protocol or scheduler rewrites.

## Superseding conditions

Supersede this ADR to change the primary model family/format, promote another backend to Tier-1, or expand the compatibility promise after reference, integration, performance, and licence/provenance evidence is available.
