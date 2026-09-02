# Workspace architecture

> **Source of truth.** This document defines the intended Cargo workspace
> boundaries and dependency direction. Change it only through an explicit
> architecture redesign; the crate graph must converge on this structure.

The active Cargo workspace implements the Crux kernel-and-shell design in [architecture](architecture.md):

```text
client shell: synapseflow-cli
        ↓ drives and resolves effects
kernel: synapseflow-kernel
        ↓
application: synapseflow-application
        ↓
domain: synapseflow-domain  ←  ports: synapseflow-ports
                                    ↑
adapters: synapseflow-adapter-in-memory
          synapseflow-adapter-local-cache
          synapseflow-adapter-llama-cpp
          synapseflow-adapter-loopback
          synapseflow-adapter-layer-range
```

`synapseflow-kernel` depends on `crux_core` and portable effect contracts only. It never starts a runtime, binds a listener, touches storage, or loads a model. It represents client workflow state, asks shells to perform typed effects, and can therefore serve Rust, FFI, and future UI shell. `synapseflow-cli` is the only current shell and performs that complete exchange.

`synapseflow-domain` contains only standard-library types and typed errors.
`synapseflow-ports` uses domain contracts and no runtime or infrastructure
framework. `synapseflow-application` has dependencies only on domain and ports.
Applications are composition roots; adapters implement ports and own
infrastructure dependencies. `synapseflow-adapter-local-cache` owns
provisioned-manifest verification and the filesystem-only content-addressed
cache; cache paths are omitted from application inspection results and are
handed only to a backend adapter for loading.
`synapseflow-adapter-llama-cpp` owns the optional CPU-only native
runtime. `synapseflow-adapter-loopback` owns the bounded production-codec test
transport. `synapseflow-adapter-layer-range` owns Loom, the Llama
runtime, including its pinned Candle dependencies; none may leak
into domain, ports, or application.

The workspace does not include `core`, `coord`, `inference`, `runtime`,
`network`, `security`, `utils`, or `incentive` crates. Their former
responsibilities are represented through the architecture, roadmap, ADRs, and
contracts above. The workspace uses a scoped GGUF/`Q5_K_M` Loom Llama adapter
with its own validation, provenance, and compatibility requirements.

Use `cargo tree --workspace --edges normal` and the architecture tests in `synapseflow-application` to review the active dependency direction. Any direct infrastructure dependency added to domain, ports, or application is an architecture violation.
