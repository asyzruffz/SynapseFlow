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
```

`synapseflow-domain` contains only standard-library types and typed errors. `synapseflow-ports` uses domain contracts and no runtime or infrastructure framework. `synapseflow-application` has dependencies only on domain and ports. Applications are composition roots; adapters implement ports and own infrastructure dependencies.

The historical `core`, `coord`, `inference`, `runtime`, `network`, `security`, `utils`, and `incentive` directories are deliberately excluded from the active Cargo workspace. They preserve pre-milestone source for migration reference but are not supported product paths and must not be imported by active crates. In particular, the Candle/safetensors inference path is retired from the supported build. Their responsibilities migrate into the active layers only when the relevant roadmap milestone supplies a tested contract.

Use `cargo tree --workspace --edges normal` and the architecture tests in `synapseflow-application` to review the active dependency direction. Any direct infrastructure dependency added to domain, ports, or application is an architecture violation.
