# Command-line interface

Milestone 2 exposes one verified-local generation workflow through the `synapseflow` CLI and the loopback-only `synapseflow-node` API. Both surfaces compose the same `LocalNode`, which delegates to `GenerationService`; neither accepts a raw model-weight URL as its model selector.

## CLI generation

The native backend is opt-in so normal workspace development does not require llama.cpp build prerequisites. Supply every local runtime input explicitly:

```powershell
cargo run -p synapseflow-cli --features runtime --locked -- run `
  --model registry://fixtures/synapseflow-verified-local-tinyllama-q5km-v1@sha256:8c9c17c57eed25a84f908c6a72a24d90d37546713d4480aba2e3dadb5d0e29e5 `
  --prompt "The capital of France is" `
  --max-tokens 16 `
  --temperature 0.7 `
  --top-p 0.9 `
  --seed 42 `
  --manifest D:\Workspace\SynapseFlow\models\tinyllama\fixture-manifest.json `
  --artifact D:\Workspace\SynapseFlow\models\tinyllama\tinyllama-1.1b-chat-v0.3.Q5_K_M.gguf `
  --cache-dir D:\Workspace\SynapseFlow\.cache\tinyllama `
  --publisher-public-key KkKTwJqz36t2XlAOvoxVmwsSqiXpClD456q5soZTG5A
```

The command validates the reference, manifest, signature, compatibility tuple, cache entry, and artifact before loading the backend. It writes generated text to standard output and reports an opaque session ID on standard error. Use `--json` to emit the session ID, decoded text, and token-ID vector for reference-vector comparison. `--output <path>` is optional, but when present it creates a new explicit destination and refuses to overwrite an existing file. Failures exit non-zero and begin with a stable error code; diagnostics never include the prompt, paths, manifest bytes, or weights.

## Local API node

Start the API with the same explicit runtime configuration. It binds only to a loopback address; a non-loopback `--bind` value is rejected.

```powershell
cargo run -p synapseflow-node --features cli,http,runtime --locked -- `
  --model registry://fixtures/synapseflow-verified-local-tinyllama-q5km-v1@sha256:8c9c17c57eed25a84f908c6a72a24d90d37546713d4480aba2e3dadb5d0e29e5 `
  --manifest D:\Workspace\SynapseFlow\models\tinyllama\fixture-manifest.json `
  --artifact D:\Workspace\SynapseFlow\models\tinyllama\tinyllama-1.1b-chat-v0.3.Q5_K_M.gguf `
  --cache-dir D:\Workspace\SynapseFlow\.cache\tinyllama `
  --publisher-public-key KkKTwJqz36t2XlAOvoxVmwsSqiXpClD456q5soZTG5A
```

`POST /v1/generate` accepts at most 16 KiB of JSON and returns a session ID plus the full token vector and decoded text. `POST /v1/generate/stream` accepts the same body and sends the token vector as Server-Sent Events (`token` followed by `complete`). The current `ModelBackend` port returns a completed output atomically, so this initial SSE surface frames its verified, ordered tokens after the workflow completes; it does not claim live decoder-token delivery.

```json
{
  "model": "registry://fixtures/synapseflow-verified-local-tinyllama-q5km-v1@sha256:8c9c17c57eed25a84f908c6a72a24d90d37546713d4480aba2e3dadb5d0e29e5",
  "prompt": "The capital of France is",
  "max_tokens": 16,
  "temperature": 0.7,
  "top_p": 0.9,
  "seed": 42,
  "deadline_ms": 30000
}
```

`deadline_ms` is optional. Once set, its monotonic deadline is propagated through the application workflow and is checked before resolution, acquisition, backend invocation, and every generated token. A timeout returns `SYN-INFER-005`; the API maps it to HTTP 504. API errors are JSON objects with only `code` and a safe `message`.

The API has no authentication because it is loopback-only and is limited to this local milestone. Remote access, multi-user authorization, model-management commands, distributed routing, and live backend token streaming remain future work.
