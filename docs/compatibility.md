# Compatibility statement

SynapseFlow follows the versioning and compatibility rules in [ADR 0002](adr/0002-release-and-compatibility-policy.md). This page defines how those rules apply to product surfaces.

| Surface | Compatibility commitment |
|---|---|
| Rust crates | Patch releases preserve documented public APIs; pre-1.0 minor releases may break them with migration notes. |
| CLI | Commands, options, output schemas, and exit codes follow the same policy; deprecations name a replacement and removal release. |
| Node API | Versioned endpoints and streaming schemas are additive within a version; incompatible changes use a new API version. |
| Frame protocol | Envelopes declare a protocol version; nodes negotiate supported versions and reject unsupported input safely. |
| Model manifest | Immutable manifests are identified by schema version, model version, publisher identity, and content hash; changed semantics create a new manifest. |
| Configuration | Configuration has explicit precedence and validation; incompatible keys receive migration guidance. |

## Delivered Milestone 2 and 3 surfaces

The supported CLI command is `synapseflow run` with an immutable `--model` reference and explicit verified-local runtime inputs. Its seeded sampling options, `--json` schema, stable domain error codes, and no-overwrite `--output` behavior are compatibility surfaces.

The loopback API provides `POST /v1/generate` and `POST /v1/generate/stream`. The JSON response and SSE `token` events carry the same ordered token ID/text data; a final SSE `complete` event carries the session ID. The initial backend produces its complete output before SSE framing, so live decoder-token delivery is not a compatibility promise.

Milestone 3 adds internal compatibility contracts for schema-v2 signed shard
declarations, `synapseflow-loom-llama-v1`, and activation-frame protocol v1.
They are exercised by the bounded local loopback harness, not exposed as a
public remote node or worker API. A changed runtime profile or shard layout
requires a new immutable manifest identity; the prior
`llama-layer-range-v1` profile is rejected.

Remote node/worker APIs, peer negotiation, authorization, configuration
files/environment precedence, and remote-worker compatibility are not
delivered surfaces yet.

## Compatibility evidence

Protocol and manifest changes require golden compatibility vectors, malformed-input tests, and an upgrade/downgrade test plan. Public crate/API/CLI changes require release notes, migration guidance when breaking, and changelog entries. Compatibility support windows and deprecation removals are recorded in release notes.
