# Model management

## Principles

Models are immutable, verified artifacts, not source files. Development and test fixtures are acquired separately from source control. A model can be resolved from an approved remote registry, downloaded into a local content-addressed cache, verified before use, and evicted without changing application behavior.

## Supported sources

A model source is a versioned manifest reference, not an arbitrary path. Supported adapters may resolve approved HTTPS registries, object storage, model hubs, or an enterprise registry. Each adapter enforces an allowlist, redirects policy, TLS validation, authentication method, bandwidth/size limits, and a provenance record.

```text
manifest reference
  │
  ▼
registry adapter ── fetch manifest ── verify publisher signature
  │
  ▼
artifact downloader ── resumable download ── verify content hash and size
  │
  ▼
content-addressed local cache ── lease to backend/worker
```

## Manifest-driven acquisition

The signed manifest declares the model format, tokenizer, shard URIs, expected byte sizes, content hashes, publisher key identifier, licence, and provenance. Downloaded artifacts are not trusted until every declared hash and signature validates. A worker refuses a model whose format, architecture, tokenizer, or runtime compatibility does not match its capability declaration.

## Cache behavior

The local cache is keyed by content hash and uses atomic staging: download to a temporary location, verify, then promote atomically. Leases prevent eviction while a worker uses an artifact. Eviction observes capacity, pinning, active sessions, and an LRU-style policy. Cache metadata records origin, fetch time, verification result, licence/provenance, and last use; it contains no secrets.

## Development artifacts

Local GGUF, safetensors, tokenizer, and benchmark artifacts are excluded from Git. Developers obtain them through a documented fixture command, an approved registry, or local test setup. Small generated fixtures may be versioned only when their licence, provenance, and size are explicitly acceptable. Private models, credentials, prompts, and activation dumps are never committed.

## Security and operations

Model download credentials are supplied through the deployment secret mechanism, never command-line history or repository files. The registry adapter records an audit event for fetch, verification failure, promotion, eviction, and access denial. Key rotation/revocation invalidates affected manifests; operators can quarantine a model version and prevent new sessions while safely completing or cancelling existing ones.
