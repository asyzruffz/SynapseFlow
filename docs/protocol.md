# Protocol

## Compatibility rules

The wire protocol uses an explicit binary schema with a `protocol_version` on every envelope. New fields are additive and have defined defaults; incompatible semantic changes require a new version. Generated code, golden byte vectors, malformed-input tests, and cross-version tests are part of the protocol release.

All untrusted sizes are checked before decoding, decompression, allocation, buffering, or reassembly. Every implementation enforces declared limits for envelope bytes, payload bytes, decompressed bytes, tensor rank/dimensions, in-flight frames, and session lifetime.

## Model manifest

A canonical, signed manifest identifies one immutable model version. The publisher signature covers every semantic field except its signature envelope.

```json
{
  "schema_version": 1,
  "model_id": "tinyllama-chat",
  "model_version": "1.1b-q5km-2026-08-22",
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

For Milestone 2, `schema_version` is exactly `1`; the only supported tuple is one `gguf` artifact for the `llama` architecture with `Q5_K_M` quantization and an embedded `llama` tokenizer. The document is limited to 64 KiB and rejects unknown fields, duplicate or invalid artifact identities, unsafe URIs, malformed SHA-256 values, and unsupported compatibility declarations before any artifact is acquired.

The manifest reference has the form `registry://<name>@sha256:<signed-manifest-hash>`, where the hash is lower-case SHA-256 of the complete RFC 8785 canonical JSON document, including the signature envelope. The publisher signs a second RFC 8785 canonical representation containing every field except `signature`, with Ed25519. Signatures and trusted public keys use unpadded base64url; key identifiers have the form `ed25519:<name>` and resolve only through the active trust store. A content hash validates artifact bytes; a signature authenticates the publisher.

Shards, external tokenizers, replica requirements, rotation/revocation distribution, and other distributed-execution fields remain future-milestone protocol work and are not accepted by the Milestone 2 parser.

## Frame envelope

Frames are sent over authenticated, multiplexed streams. Each envelope contains:

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

## Control semantics

| Message | Required behavior |
|---|---|
| `data` | Validate the entire envelope and payload, then ACK or return a typed NACK/error. |
| `ack` | Confirm a contiguous sequence range; ACKs are idempotent. |
| `nack` | Identify sequence and stable reason, such as checksum, bounds, dtype, or model mismatch. |
| `cancel` | Stop queued and active work; repeated cancellation succeeds. |
| `heartbeat` | Supports health observation and carries no model data. |
| `error` | Provides a stable code, retriable flag, and safe diagnostic text. |

Retries use an idempotency key and a bounded retry budget. On a deadline or peer failure, the session manager either retries the worker or resumes through a replica from the last valid checkpoint. It never permits untracked duplicate work indefinitely.

## Session lifecycle

```text
Created → Planned → Running → Completing → Completed
                     │  │
                     │  └→ Cancelling → Cancelled
                     └────→ Retrying → Running
                     └────→ Failed
```

Transitions, checkpoint ownership, audit events, and result-delivery semantics are tested contracts. A session becomes `Completed` only after validated final output has been delivered or durably recorded according to the API contract.
