# Active workspace architecture

The active Cargo workspace implements the dependency direction in [architecture](architecture.md):

```text
applications: synapseflow-cli, synapseflow-node
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

`synapseflow-domain` contains only standard-library types and typed errors.
`synapseflow-ports` uses domain contracts and no runtime or infrastructure
framework. `synapseflow-application` has dependencies only on domain and ports.
Applications are composition roots; adapters implement ports and own
infrastructure dependencies. `synapseflow-adapter-local-cache` owns
provisioned-manifest verification and the filesystem-only content-addressed
cache; cache paths are omitted from application inspection results and are
handed only to a backend adapter for loading.
`synapseflow-adapter-llama-cpp` owns the optional Milestone 2 CPU-only native
runtime. `synapseflow-adapter-loopback` owns the bounded production-codec test
transport. `synapseflow-adapter-layer-range` owns Loom, the Milestone 3 Llama
runtime, including its future pinned Candle dependencies; none may leak
into domain, ports, or application.

The pre-milestone `core`, `coord`, `inference`, `runtime`, `network`, `security`,
`utils`, and `incentive` crates were removed after their useful responsibilities
were captured in the target architecture, roadmap, ADRs, and active contracts.
The retired Candle/safetensors path is not revived; ADR 0006 instead introduces
a newly scoped, GGUF/`Q5_K_M`, Loom Llama adapter with its own tests,
provenance, and compatibility evidence.

Use `cargo tree --workspace --edges normal` and the architecture tests in `synapseflow-application` to review the active dependency direction. Any direct infrastructure dependency added to domain, ports, or application is an architecture violation.
