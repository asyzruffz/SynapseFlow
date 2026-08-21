# Command-line interface

The `synapseflow` CLI is an authenticated client for running and operating SynapseFlow nodes. It never requires users to manually route shards, handle activation frames, or manipulate local model-weight paths.

## Commands

| Command | Purpose |
|---|---|
| `synapseflow run` | Submit a generation request and stream its output. |
| `synapseflow models pull` | Resolve, download, and verify a model manifest and artifacts into the local cache. |
| `synapseflow models inspect` | Display verified manifest, compatibility, provenance, and cache state. |
| `synapseflow node start` | Start a node with validated configuration, identity, and observability. |
| `synapseflow diagnostics` | Collect safe health, configuration, and connectivity diagnostics. |

## Generation request

```text
synapseflow run \
  --model registry://models/example@sha256:... \
  --prompt "Write one sentence about Rust." \
  --max-tokens 64 \
  --temperature 0.7 \
  --top-p 0.9 \
  --seed 42
```

`--model` is a manifest reference. The client may fetch the manifest/artifacts through the model-management policy or send the reference to a node that already has them. It does not accept arbitrary remote weight URLs as execution input.

## Behavior guarantees

- Output is streamed until completion, cancellation, deadline expiration, or a typed error.
- Errors have a stable machine-readable code and safe human-readable diagnostic; invalid input never causes a panic.
- The CLI propagates cancellation and deadlines to the node and shows the request/session identifier when appropriate.
- `--output <path>` writes atomically or streams to a specified file; it never silently overwrites an implicit working-directory file.
- Structured output and metrics are opt-in and redact prompts, activations, weights, credentials, and private identifiers by default.

## Configuration

Configuration can be provided by a validated file, environment variables intended for deployment, and explicit CLI options with documented precedence. Secrets are referenced through the platform secret mechanism. The CLI validates configuration before starting a node or issuing a request and can print an effective configuration with secrets redacted.
