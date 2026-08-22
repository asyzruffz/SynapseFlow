# Verified local inference contract

This contract defines the sole supported inference compatibility tuple for Roadmap milestone 2. It is intentionally narrow: adding a model family, GGUF quantization, tokenizer mode, hardware target, or backend requires a new compatibility decision and reference-output evidence.

The corresponding implementation plan is [Milestone 2 — Verified local inference](../MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md). The durable decisions behind this contract are in [ADR 0003](adr/0003-initial-model-backend-scope.md) and [ADR 0004](adr/0004-verified-local-inference-contract.md).

## Supported compatibility tuple

| Property | Milestone 2 support |
|---|---|
| Model family and architecture | TinyLlama 1.1B Chat v0.3, declared as `llama`. Other Llama-family variants are not supported until individually tested. |
| Weight format | A single, non-sharded GGUF artifact. Safetensors, ONNX, split GGUF files, adapters, and GPU layers are rejected as unsupported. |
| Quantization | `Q5_K_M` only. The fixture's declared quantization must match the inspected GGUF metadata. |
| Runtime | `llama-cpp-2` **`=0.1.154`**, using its bundled llama.cpp integration and CPU-only configuration. The exact lockfile-resolved `llama-cpp-sys-2` and llama.cpp revision are part of acceptance evidence. |
| Platforms | Tier-1 `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`, CPU execution only. Build provisioning must include the native C/C++/bindgen prerequisites required by the pinned adapter; the default Rust workspace remains backend-independent. |
| Tokenizer | The embedded `tokenizer.ggml.model = llama` tokenizer in the verified GGUF artifact. No separate `tokenizer.json` is accepted or loaded for this tuple. The application applies no implicit chat template; callers provide the complete prompt. |
| Context | A request is rejected when prompt tokens plus requested output tokens exceed the lesser of 2,048 and the GGUF-declared context limit. Context overflow never truncates input silently. |
| Sampling | `temperature` is finite and in `(0.0, 2.0]`; `top_p` is finite and in `(0.0, 1.0]`; `max_tokens` is `1..=256`; `seed` is an explicit `u64`. A deterministic test mode fixes all four values and the pinned runtime version. |
| Output | The application exposes token IDs and decoded UTF-8 text. Generation ends at the model EOS token, the requested maximum, caller cancellation, or deadline expiration. |

The adapter is an implementation detail behind the `ModelBackend` port. Domain and application code must not expose its types, unsafe API, or runtime-specific configuration.

## Fixture identity and provisioning

The real-fixture acceptance suite is provisioned explicitly and is not part of the default quality gate. The fixture is acquired only through the signed SynapseFlow fixture manifest described below; it is never copied from `models/` or any other checkout path.

| Field | Required value |
|---|---|
| Fixture ID | `synapseflow-verified-local-tinyllama-q5km-v1` |
| Upstream publisher | `TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF` |
| Pinned upstream revision | `787449158421637e2922ad034b666bc1f74d2ffd` |
| Artifact name | `tinyllama-1.1b-chat-v0.3.Q5_K_M.gguf` |
| Artifact size | `782052992` bytes |
| Artifact SHA-256 | `7c255febbf29c97b5d6f57cdf62db2f2bc95c0e541dc72c0ca29786ca0fa5eed` |
| Licence | `Apache-2.0` |
| Format / architecture / quantization | `gguf` / `llama` / `Q5_K_M` |
| Tokenizer declaration | Embedded `llama` tokenizer in the GGUF artifact; no separate tokenizer artifact |
| Distribution endpoint | The approved registry maps the signed manifest artifact URI to the pinned upstream revision. Direct Hugging Face URLs are not accepted as `synapseflow run` input. |
| SynapseFlow publisher key ID | `ed25519:synapseflow-fixture-2026-08` |
| Manifest reference | `registry://fixtures/synapseflow-verified-local-tinyllama-q5km-v1@sha256:<signed-manifest-hash>`; the literal hash is assigned only when the canonical manifest is signed in Step 3. |

The fixture signing key is a dedicated non-production test key. Its public key is distributed through the test trust-store fixture; its private half is supplied only by the fixture-provisioning environment. Production trust stores must not include this key.

The first implementation of the fixture manifest must include the values above, its own SHA-256 content identity, the publisher key ID, Apache-2.0 provenance, the compatible runtime tuple, and a detached signature. A changed field creates a new fixture ID and manifest reference.

## Deterministic reference-output procedure

The expected token vector is deliberately not specified before an adapter has generated and independently checked it. Publishing a guessed vector would make the acceptance test meaningless. Step 5 creates the immutable vector using this procedure:

1. Provision the fixture through the signed manifest and verify the manifest, artifact size, and SHA-256.
2. Run the pinned adapter on a Tier-1 CPU platform using prompt `The capital of France is`, `max_tokens = 16`, `temperature = 0.7`, `top_p = 0.9`, and `seed = 42`.
3. Record the complete generated token-ID sequence, decoded UTF-8 output, manifest hash, adapter and llama.cpp revisions, operating system, CPU architecture, and command/application version in a versioned acceptance-vector artifact.
4. Independently reproduce the vector on the other Tier-1 platform. If either the IDs or decoded UTF-8 differ, treat it as a compatibility failure; do not loosen the assertion to a text similarity check.
5. Pin the accepted vector and make the provisioned integration suite compare against it exactly. Any backend, tokenizer, sampling, or fixture change needs a new vector and compatibility review.

Until that vector exists, the fixture is suitable only for acquisition and metadata checks, not as proof of successful generation.

## Manifest and signature profile

Milestone 2 uses this profile when implementing the protocol manifest:

- Canonical serialization is UTF-8 JSON Canonicalization Scheme (RFC 8785).
- The signature is a detached Ed25519 signature over the canonical serialization of every semantic field; the signature envelope itself is excluded.
- The signature is encoded as unpadded base64url. Hashes use lower-case hexadecimal SHA-256 and are prefixed `sha256:` in manifests.
- Manifest JSON is limited to 64 KiB; Milestone 2 permits exactly one weight artifact and no more than one embedded-tokenizer declaration. URI, size, integer, string, and compatibility fields are validated before any fetch or allocation.
- Key IDs use `ed25519:<name>` and resolve only through the configured trust store. Unknown, revoked, expired, or environment-inappropriate keys return a typed trust error.

## Stable error contract

Public library errors expose a typed category and a stable `code`; application boundaries render that code with a safe diagnostic and never use an assertion for untrusted input. The eventual domain error type is the authority for this table; temporary crate-specific errors must map losslessly to it during the Step 2 migration.

| Code | Typed category | Required condition |
|---|---|---|
| `SYN-MODEL-001` | `InvalidReference` | Malformed, unversioned, or otherwise invalid manifest reference. |
| `SYN-MODEL-002` | `DisallowedSource` | Reference or artifact source violates the configured scheme/registry policy. |
| `SYN-MODEL-003` | `ManifestInvalid` | Manifest cannot be decoded, canonicalized, bounded, or validated. |
| `SYN-MODEL-004` | `ManifestUnavailable` | An allowed manifest reference cannot be resolved or read. |
| `SYN-MODEL-005` | `ManifestUnsupported` | Schema, format, architecture, tokenizer mode, quantization, or runtime declaration is unsupported. |
| `SYN-MODEL-006` | `PublisherUntrusted` | Publisher key is unknown, revoked, expired, or outside the active environment's trust policy. |
| `SYN-MODEL-007` | `SignatureInvalid` | Manifest signature is absent, malformed, or fails verification. |
| `SYN-MODEL-008` | `ArtifactUnavailable` | A declared artifact cannot be read, fetched, resumed, or promoted. |
| `SYN-MODEL-009` | `ArtifactIntegrity` | An artifact size or SHA-256 does not match its verified manifest declaration. |
| `SYN-MODEL-010` | `CacheFailure` | Cache staging, leasing, metadata, eviction, or atomic promotion fails. |
| `SYN-INFER-001` | `BackendUnavailable` | The configured GGUF backend cannot be initialized on the current host. |
| `SYN-INFER-002` | `BackendIncompatible` | A verified artifact cannot run under the declared adapter/platform/context capability. |
| `SYN-INFER-003` | `TokenizerFailure` | Embedded tokenizer validation, encoding, or decoding fails. |
| `SYN-INFER-004` | `GenerationPolicyInvalid` | Prompt, token limit, seed, temperature, top-p, or context bound is invalid. |
| `SYN-INFER-005` | `GenerationFailed` | Backend evaluation/sampling fails after validated initialization. |

Errors must not include credentials, raw manifests, prompt text, weights, activation data, or backend paths. The application may add sanitized context while retaining the typed code.

## Acceptance evidence

The milestone cannot claim verified local inference until all of the following exist:

- canonical and signed manifest golden vectors, including signature and trust negative cases;
- hermetic unit/integration coverage of every error category above where its adapter is available;
- a provisioned fixture run that exactly matches the accepted token-ID vector on both Tier-1 platforms; and
- a review by the runtime/model and compatibility maintainers required by [the code-review policy](code-review-policy.md).
