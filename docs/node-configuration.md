# Operable node configuration

`synapseflow serve` reads node configuration with this precedence:

1. Explicit command-line option.
2. `SYNAPSEFLOW_*` environment variable.
3. TOML configuration file.
4. Documented safe default.

An operational configuration must validate completely before the public or
management listener starts. Unknown keys and incompatible combinations are
rejected. Secrets are supplied by deployment secret references/environment, not
through shell history or repository files.

`development` is the safe default profile and binds the public listener only to
loopback. `operational` is required for a non-loopback public listener and then
requires direct TLS material or at least one trusted TLS-terminating proxy
address. Forwarded headers are not interpreted until the public API handlers
are installed.

## Required sections

```toml
[public]
bind = "127.0.0.1:8080"
# tls_cert_file and tls_key_file, or trusted_proxy_addresses, are required for
# a non-loopback operational bind.

[management]
bind = "127.0.0.1:9090"

[keycloak]
issuer = "https://identity.example/realms/synapseflow"
audience = "synapseflow-node"
allowed_algorithms = ["RS256"]
jwks_max_staleness_seconds = 3600
clock_skew_seconds = 60

[admission]
max_request_bytes = 16384
max_prompt_bytes = 8192
max_output_tokens = 256
max_deadline_ms = 30000
max_concurrent_sessions = 1
max_sessions_per_principal = 1
max_queue_depth = 0

[state]
# SQLite file containing safe session metadata, idempotency keys, and checkpoint references.
# It never contains prompts, generated text, bearer tokens, or activation payloads.
database_path = "/var/lib/synapseflow/state.db"

[model_policy]
# Each value is an immutable, signed manifest reference. Do not configure a URL,
# backend, cache entry, worker, or transport route here.
allowed_models = ["registry://example/approved@sha256:<64-lowercase-hex-characters>"]

[verified_local_runtime]
# v1 serves one configured, immutable verified-local model. `model` must be
# the sole value in model_policy.allowed_models.
model = "registry://example/approved@sha256:<64-lowercase-hex-characters>"
manifest_path = "/var/lib/synapseflow/manifests/approved.json"
artifact_path = "/var/lib/synapseflow/artifacts/approved.gguf"
cache_directory = "/var/lib/synapseflow/cache"
publisher_public_key = "<base64url-ed25519-public-key>"

[audit]
directory = "/var/lib/synapseflow/audit"
max_file_bytes = 10485760
max_file_age_seconds = 86400
max_retained_files = 10

[telemetry]
queue_capacity = 256

[shutdown]
drain_seconds = 30
```

The example values are deployment inputs, not universal defaults. The acceptance
load test establishes supported values for the selected model, hardware, and
runtime profile.

## Validation rules

- A public non-loopback listener requires direct TLS or configured trusted TLS
  proxy addresses. Forwarded headers are ignored for all other peers.
- The Keycloak issuer uses HTTPS, has a non-empty audience and algorithm
  allowlist, and permits a bounded JWKS cache lifetime.
- The management listener is private by default and cannot share the public
  listener without an explicit reviewed operator policy.
- `max_request_bytes` bounds the buffered HTTP body. `max_prompt_bytes`,
  `max_output_tokens`, and `max_deadline_ms` are independently checked before
  model acquisition; the runtime still rejects a prompt/output combination that
  exceeds the verified model's context window.
- Audit rotation is bounded by both file size and active-file age. On Unix the
  node enforces owner-only modes for the audit directory and files. On Windows,
  provision the audit directory with an NTFS ACL granting access only to the
  account that runs `synapseflow serve`; native ACL enforcement is tracked as a
  platform-hardening follow-up.
- Audit storage is writable before readiness can pass. A persistence failure
  makes readiness fail and blocks new admission.
- The SQLite state database is single-node state. Place it on local durable
  storage; do not use a network filesystem or share it between node processes.
- Model access policy names immutable manifest references only. It never names
  raw artifact URLs, backend implementations, workers, cache locations, or
  transport routes.
- The initial verified-local node profile serves exactly one immutable model.
  Its runtime paths and publisher public key are CLI deployment composition,
  kept in `[verified_local_runtime]` rather than exposed through the API.
