# Protocol

> **Source of truth.** This document defines the manifest and frame contracts.
> Change it only through an explicit protocol redesign with compatible
> implementation and migration work; implementation gaps do not redefine this
> contract.

## Compatibility rules

The wire protocol uses an explicit binary schema with a `protocol_version` on every envelope. New fields are additive and have defined defaults; incompatible semantic changes require a new version. Generated code, golden byte vectors, malformed-input tests, and cross-version tests are part of the protocol release.

All untrusted sizes are checked before decoding, decompression, allocation, buffering, or reassembly. Every implementation enforces declared limits for envelope bytes, payload bytes, decompressed bytes, tensor rank/dimensions, in-flight frames, and session lifetime.

## Model manifest

A canonical, signed manifest identifies one immutable model version. The publisher signature covers every semantic field except its signature envelope.

```json
{
  "schema_version": 1,
  "model_id": "tinyllama-chat",
  "model_version": "1.1b-q5km-v1",
  "format": "gguf",
  "architecture": "llama",
  "quantization": "Q5_K_M",
  "tokenizer": {
    "kind": "embedded",
    "model": "llama"
  },
  "artifacts": [
    {
      "artifact_id": "weights",
      "uri": "https://registry.example/models/tinyllama.Q5_K_M.gguf",
      "content_sha256": "sha256:...",
      "size_bytes": 782052992
    }
  ],
  "publisher_key_id": "ed25519:publisher-2026-01",
  "license": "Apache-2.0",
  "provenance": "fixture:tinyllama",
  "signature": "base64url:..."
}
```

For verified local inference, `schema_version` is exactly `1`; the only supported tuple is one `gguf` artifact for the `llama` architecture with `Q5_K_M` quantization and an embedded `llama` tokenizer. The document is limited to 64 KiB and rejects unknown fields, duplicate or invalid artifact identities, unsafe URIs, malformed SHA-256 values, and unsupported compatibility declarations before any artifact is acquired.

The manifest reference has the form `registry://<name>@sha256:<signed-manifest-hash>`, where the hash is lower-case SHA-256 of the complete RFC 8785 canonical JSON document, including the signature envelope. The publisher signs a second RFC 8785 canonical representation containing every field except `signature`, with Ed25519. Signatures and trusted public keys use unpadded base64url; key identifiers have the form `ed25519:<name>` and resolve only through the active trust store. A content hash validates artifact bytes; a signature authenticates the publisher.

The schema-v1 parser continues to reject shards, external
tokenizers, replica requirements, rotation/revocation distribution, and other
distributed-execution fields. Loopback sharding adds the separate, validated
schema-v2 loopback-sharding declaration described below; remote distribution,
key operations, and other distributed-execution protocol work remain future
capabilities.

### Loopback-sharding runtime profile

Loopback sharding uses the schema-v2 shard declaration shape with execution
strategy `layer_range_v1` and runtime profile
`synapseflow-loom-llama-v1`. The profile binds the declaration to Loom, the
pinned Llama runtime defined by ADR 0006. A profile change creates a
new signed manifest identity and canonical vector; consumers must reject the
previous `llama-layer-range-v1` profile rather than treating the two runtimes as
interchangeable.

## Activation-frame protocol v1

The activation-frame protocol is the data-plane contract. Application-owned
control-plane decisions—identity, authorization, quotas, manifest selection,
route selection, checkpoint selection, retry policy, and terminal cleanup—are
never encoded as frames or delegated to workers. Frames are sent over
authenticated, multiplexed streams. Protocol v1 uses a
deterministic big-endian binary schema with no generated runtime code. The
decoder first validates the complete fixed 16-byte prefix, the declared header
and payload lengths, and the total frame length. It does not allocate, buffer,
or decompress payload bytes before those checks pass.

The prefix is `SYNF` (four ASCII bytes), followed by the 16-bit protocol
version, one-byte message type, one-byte compression tag, 32-bit header byte
length, and 32-bit payload byte length. The header fields are ordered exactly as
follows:

| Field | Meaning |
|---|---|
| `protocol_version` | Supported schema version. |
| `message_type` | `data`, `ack`, `nack`, `cancel`, `heartbeat`, or `error`. |
| `session_id` | Opaque, unguessable request correlation identifier. |
| `stream_id`, `sequence` | Monotonic ordering within one stream. |
| `model_version`, `shard_id` | Immutable execution target. |
| `tensor` | Dtype, dimensions, byte order, encoding, and uncompressed length. |
| `compression` | Named, versioned compression algorithm or `none`. |
| `payload_sha256` | Hash of the explicitly specified canonical payload bytes. |
| `deadline` | Remaining request deadline. |
| `trace_id` | Observability correlation without prompt content. |

Variable strings use a one-byte byte length and UTF-8. `session_id` and
`shard_id` are limited to 128 bytes, model references to 255 bytes, and safe
trace IDs to 128 bytes. Integers and tensor dimensions are big-endian. A tensor
is present only for `data` frames and is encoded as a presence byte, dtype tag,
rank, and dimensions; v1 accepts `f32` boundary/logit values (tag `1`) and
little-endian `u32` token IDs (tag `2`), rank 1–8, and at most 64 MiB of
uncompressed payload. Control frames have no tensor and an empty payload.

For Loom's `layer_range_v1` execution, an initial input is a rank-1 `u32`
tensor shaped `[token_count]`; an intermediate boundary is a rank-2 `f32`
tensor shaped `[token_count, activation_width]`; and final logits are a rank-1
`f32` tensor shaped `[vocabulary_size]`. Carrying every prompt-token activation
lets each receiving range build its own session-scoped KV entries without
crossing runtime state over the wire.

The `payload_sha256` is 32 raw SHA-256 bytes over the canonical *uncompressed*
payload. v1 supports only compression tag `none`; any other tag is rejected
before a decompressor could be selected or invoked. A future compression
algorithm must be versioned, must verify the compressed bound before decoding,
must enforce a decompressed bound before allocation, and must retain this hash
definition. This makes a decompression bomb an unsupported, safely rejected
frame in v1 rather than a latent resource-risk path.

The header may end with zero or more additive TLV extensions: a non-zero tag,
16-bit big-endian value length, then that many value bytes. A v1 decoder
preserves well-formed extensions within the already validated header bound so a
consumer can use documented tags and ignore unknown tags. A semantic, decoding,
or required-field change needs a new protocol version.

Layer-range execution reserves extension tag `2` for an eight-byte big-endian
`position_start`. Every token-ID input and activation-boundary frame carries
exactly one such extension; it binds a stage's values to their model position
without exposing runtime state.

Protocol-v1 packets must have a positive remaining deadline no greater than 24
hours. The encoder emits no extensions, uses `none` compression, and produces
one canonical byte representation for a given frame. The committed domain test
contains the corresponding golden byte vector.

## Control semantics

These messages control bounded transport behavior for an existing,
application-authorized session. They do not create a session, select a worker,
grant authorization, change a retry budget, or extend a deadline.

| Message | Required behavior |
|---|---|
| `data` | Validate the entire envelope and payload, then ACK or return a typed NACK/error. |
| `ack` | Confirm a contiguous sequence range; ACKs are idempotent. |
| `nack` | Identify sequence and stable reason, such as checksum, bounds, dtype, or model mismatch. Protocol-v1 encodes the ASCII stable error code in additive extension tag `1`. |
| `cancel` | Stop queued and active work; repeated cancellation succeeds. |
| `heartbeat` | Supports health observation and carries no model data. |
| `error` | Provides a stable code, retriable flag, and safe diagnostic text. |

Retries use an idempotency key and a bounded retry budget. On a deadline or peer
failure, the application-owned session manager either retries the worker or
resumes through a replica from the last valid checkpoint reference. It never
permits untracked duplicate work indefinitely.

## Session lifecycle

```text
Created → Planned → Running → Completing → Completed
                     │  │
                     │  └→ Cancelling → Cancelled
                     └────→ Retrying → Running
                     └────→ Failed
```

The application layer owns transitions, checkpoint references, audit events,
and result-delivery semantics. A session becomes `Completed` only after
validated final output has been delivered or durably recorded according to the
API contract. Workers may retain only bounded runtime data required for their
declared stage; they do not own public session state or terminal policy.
