# Verified local inference acceptance record — 2026-08-22

This record captures the provisioned Windows acceptance run for Milestone 2. It contains no model, private key, cache path, or complete token vector.

| Field | Value |
|---|---|
| Platform | Microsoft Windows 10.0.26100, x86_64 CPU |
| Processor identifier | Intel64 Family 6 Model 151 Stepping 2 |
| Rust toolchain | `rustc 1.89.0 (29483883e 2025-08-04)` |
| Backend | `llama-cpp-2 =0.1.154`, CPU-only; llama.cpp revision `b10200` |
| Manifest reference | `registry://fixtures/synapseflow-verified-local-tinyllama-q5km-v1@sha256:8c9c17c57eed25a84f908c6a72a24d90d37546713d4480aba2e3dadb5d0e29e5` |
| Publisher key ID | `ed25519:synapseflow-fixture-2026-08` |
| Artifact | `tinyllama-1.1b-chat-v0.3.Q5_K_M.gguf`; 782,052,992 bytes; SHA-256 `7c255febbf29c97b5d6f57cdf62db2f2bc95c0e541dc72c0ca29786ca0fa5eed` |
| Request | Public fixture prompt defined in `verified-local-inference.md`; `max_tokens=16`, `temperature=0.7`, `top_p=0.9`, `seed=42` |
| Result | 16 generated tokens; canonical token-ID/text vector SHA-256 `e4ca0a473f3b71fcbbb31b28f556089d95741bfb9ad0debae6a2e20126d71a6b`; matched the accepted external vector |
| Measurement method | One direct `synapseflow --json` process using the verified local cache; stopwatch from process launch to completed output; working set sampled every 25 ms until process exit |
| Elapsed time | 12,032 ms end-to-end, including process startup, model mapping, and generation |
| Throughput | 1.33 generated tokens/s over the end-to-end measurement |
| Peak working set | 766.4 MiB sampled process working set |

The acceptance vector itself remains externally controlled. The user previously attested that the same vector was accepted on both Tier-1 platforms; a platform-specific Linux measurement record is still required before Step 7 can be closed.
