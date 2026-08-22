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
```

`synapseflow-domain` contains only standard-library types and typed errors. `synapseflow-ports` uses domain contracts and no runtime or infrastructure framework. `synapseflow-application` has dependencies only on domain and ports. Applications are composition roots; adapters implement ports and own infrastructure dependencies. `synapseflow-adapter-local-cache` owns provisioned-manifest verification and the filesystem-only content-addressed cache; cache paths are omitted from application inspection results and are handed only to a backend adapter for loading. `synapseflow-adapter-llama-cpp` owns the optional, CPU-only native `llama-cpp-2` runtime feature.

The historical `core`, `coord`, `inference`, `runtime`, `network`, `security`, `utils`, and `incentive` directories are deliberately excluded from the active Cargo workspace. They preserve pre-milestone source for migration reference but are not supported product paths and must not be imported by active crates. In particular, the Candle/safetensors inference path is retired from the supported build. Their responsibilities migrate into the active layers only when the relevant roadmap milestone supplies a tested contract.

Use `cargo tree --workspace --edges normal` and the architecture tests in `synapseflow-application` to review the active dependency direction. Any direct infrastructure dependency added to domain, ports, or application is an architecture violation.
