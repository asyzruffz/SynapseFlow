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

## Compatibility evidence

Protocol and manifest changes require golden compatibility vectors, malformed-input tests, and an upgrade/downgrade test plan. Public crate/API/CLI changes require release notes, migration guidance when breaking, and changelog entries. Compatibility support windows and deprecation removals are recorded in release notes.
