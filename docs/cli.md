# Command-line interface

> **Source of truth.** This document defines the CLI contract. Change it only
> through an explicit interface redesign with compatibility guidance; the CLI
> implementation must conform to this contract.

The `synapseflow` CLI exposes one verified-local generation workflow. It drives
a fresh `synapseflow-kernel` lifecycle and never accepts a raw model-weight URL
as its model selector.

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

Remote access, multi-user authorization, model-management commands, a node API, distributed routing, and live backend token streaming remain future work. Any future transport is a separate client shell and must drive the kernel directly.
