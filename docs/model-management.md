# Model management

> **Source of truth.** This document defines the model lifecycle and trust
> rules. Change it only through an explicit model-management redesign; the
> implementation must conform without weakening verification.

## Principles

Models are immutable, verified artifacts, not source files. Development and test fixtures are provisioned separately from source control. A model can be resolved from an approved remote registry, downloaded into a local content-addressed cache, verified before use, and evicted without changing application behavior. Verified local inference accepts one explicit local GGUF source that is declared by a signed immutable manifest, verifies it into a content-addressed cache, and uses it only through the verified-local workflow.

The initial supported model/runtime combination is defined in [ADR
0003](adr/0003-initial-model-backend-scope.md). [ADR
0006](adr/0006-loom-layer-range-backend.md) adds Loom, the separate
Llama profile that reuses only immutable verified GGUF artifacts; it
must maintain its own dependency provenance, tokenizer/layout validation, and
baseline validation. Its schema-v2 runtime profile is
`synapseflow-loom-llama-v1`, so a previously signed
`llama-layer-range-v1` manifest is not silently reused. Additional formats and backends require their own
compatibility decision and test validation.

Loom does not broaden acquisition or model-family support: it consumes only a
manifest-verified, `gguf`/Llama/`Q5_K_M` artifact already admitted by the
declared schema-v2 compatibility tuple. Its role-limited loader reads only the
tensors declared for its assigned range. A different model family, format,
quantization, or sharding strategy needs a new runtime profile, loader,
fixtures, and reviewed ADR.

## Supported sources

A model selector is a versioned manifest reference, not an arbitrary weight path or URL. Supported adapters may resolve approved HTTPS registries, object storage, model hubs, or an enterprise registry. Each adapter enforces an allowlist, redirects policy, TLS validation, authentication method, bandwidth/size limits, and a provenance record. The current local registry resolves only explicitly provisioned manifest bytes, and the current cache maps a manifest-declared HTTPS artifact URI only to an explicitly provisioned local source file. Remote registry discovery/download, redirects, credentials, and bandwidth control are future work.

```text
manifest reference
  │
  ▼
registry adapter ── provisioned manifest ── verify publisher signature
  │
  ▼
artifact downloader ── resumable download ── verify content hash and size
  │
  ▼
content-addressed local cache ── lease to backend
```

## Manifest-driven acquisition

The signed manifest declares the model format, tokenizer, shard/artifact URI, expected byte sizes, content hashes, publisher key identifier, licence, and provenance. Downloaded artifacts or the local source are not trusted until the manifest signature and every declared hash/size validate. A worker refuses a model whose format, architecture, tokenizer, quantization, or runtime compatibility is outside the supported tuple.

## Cache behavior

The local cache is keyed by content hash and uses atomic staging: download to a temporary location or copy from the provisioned source to a temporary location, verify, then promote atomically. Leases prevent eviction while a worker uses an artifact. Eviction observes capacity, pinning, active sessions, and an LRU-style policy. Short-lived leases prevent concurrent staging, and cleanup retains the selected active model while removing inactive objects and staging files. Cache metadata records origin, fetch time, verified manifest/artifact provenance without host paths or secrets.

## Development artifacts

Local GGUF, safetensors, tokenizer, and benchmark artifacts are excluded from Git. Developers obtain them through a documented fixture command, an approved registry, or local test setup. Small generated fixtures may be versioned only when their licence, provenance, and size are explicitly acceptable. Private models, credentials, prompts, and activation dumps are never committed.

## Security and operations

Model download credentials are supplied through the deployment secret mechanism, never command-line history or repository files. The registry adapter records an audit event for fetch, verification failure, promotion, eviction, and access denial. Key rotation/revocation invalidates affected manifests; operators can quarantine a model version and prevent new sessions while safely completing or cancelling existing ones.

The fixture’s public key is an explicit input; its signing material remains outside Git. The application records safe generation/acquisition audit metadata. Remote download credentials, key rotation/revocation, publisher quarantine, multi-session control, and operational model distribution are planned capabilities.
