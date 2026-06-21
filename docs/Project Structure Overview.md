# Project Structure Overview

```
D:\Workspace\SynapseFlow/                  # <- Workspace root
├── api/
│   ├── src/
│   │   ├── admin                          # Shard upload API endpoint
│   │   ├── lib.rs
│   │   └── local_api                      # REST/gRPC for user
│   └── Cargo.toml
├── cli/
│   ├── src/
│   │   ├── commands
│   │   │   ├── mod.rs
│   │   |   └── options.rs
│   │   └── main.rs
│   └── Cargo.toml
├── coord/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── planner.rs                     # Execution plan builder
│   │   └── session_manager.rs             # In-flight sessions tracking
│   └── Cargo.toml
├── core/
│   ├── src/
│   │   ├── shards
│   │   |   └── mod.rs
│   │   ├── lib.rs
│   │   ├── model_loader.rs                # Model manifest parsing, signature
│   │   └── shard_index.rs                 # Local metadata store via sled
│   └── Cargo.toml
├── incentive/
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── network/
│   ├── src/
│   │   ├── discovery                      # libp2p DHT overlay
│   │   │   ├── bootstrap.rs
│   │   │   ├── mod.rs
│   │   |   └── peer_health.rs
│   │   ├── transport                      # QUIC connection manager
│   │   │   ├── frame.rs
│   │   │   ├── frame_decoder.rs
│   │   │   ├── frame_encoder.rs
│   │   │   ├── mod.rs
│   │   │   └── transport_manager.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── runtime/
│   ├── src/
│   │   ├── kernels.rs                     # Quantized math modes
│   │   │   ├── mod.rs
│   │   │   ├── quantized_ops.rs
│   │   │   └── sampler.rs
│   │   ├── executor.rs                    # Subgraph execution engine
│   │   └── lib.rs
│   └── Cargo.toml
├── security/
│   ├── src/
│   │   ├── attestation.rs                 # (optional TEE support)
│   │   ├── audit.rs                       # Checksum verification & logging
│   │   ├── crypto.rs                      # Ed25519 signatures, TLS/DTLS
│   │   └── lib.rs
│   └── Cargo.toml
├── utils/                                 # Shared utilities
│   ├── src/
│   │   ├── lib.rs
│   │   ├── serializer.rs                  # JSON/proto helpers
│   │   └── storage.rs                     # Local disk cache + eviction
│   └── Cargo.toml
├── docs/                                  # This documentation folder
│   ├── Local Single-Machine Prototype.md
│   ├── Project Structure Overview.md
│   └── SynapseFlow Plan - Distributed LLM Inference.md
├── Cargo.toml
└── README.md
```
