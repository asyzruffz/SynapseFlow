# SynapseFlow

[![CI](https://github.com/asyzruffz/SynapseFlow/actions/workflows/ci.yml/badge.svg)](https://github.com/asyzruffz/SynapseFlow/actions/workflows/ci.yml)

SynapseFlow is planned to be a Rust-based distributed LLM inference system. It resolves immutable, signed model manifests; acquires verified shards from approved remote sources; executes layer groups across authenticated workers; and streams generation output through an operable node API and CLI.

SynapseFlow currently provides verified local inference for one narrow CPU-only
GGUF/Llama compatibility tuple. It also delivers a Milestone 3 engineering
path that executes two declared Llama layer ranges across bounded local
loopback workers using versioned activation frames and replica recovery. A
caller still selects a signed immutable manifest reference, and the local
runtime verifies and content-addresses the provisioned artifact before use.

## Design

The system will be built around versioned model and frame contracts, content-addressed artifacts, mutually authenticated transport, explicit deadlines/cancellation, bounded resource use, replica-aware recovery, and privacy-safe observability.

The delivered Milestone 2 and 3 slices provide versioned manifest and
activation-frame contracts, content-addressed local caching, bounded local
workers/queues, declared layer-range execution, deadlines, cancellation,
checkpointed replica recovery, and privacy-safe diagnostics. This does not
make SynapseFlow a remotely operable node: remote workers, QUIC,
authentication/authorization, public distributed APIs, and production
observability remain roadmap work.

The documentation entry point is [docs/synapseflow-documentation.md](docs/synapseflow-documentation.md).
New contributors can start with [Contributor onboarding](docs/onboarding.md).
Operators can run the supported local workflow through [the CLI and API guide](docs/cli.md).

[Milestone 2 tracker](MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md) and
[Milestone 3 tracker](MILESTONE-3-LOOPBACK-SHARDING.md) record delivery
evidence. [Implementation gap](docs/implementation-gap.md) identifies the
remaining work.

## Governance

Contributions follow [CONTRIBUTING.md](CONTRIBUTING.md), the [code-review policy](docs/code-review-policy.md), and the [release process](docs/release-process.md). Vulnerabilities are reported privately under [SECURITY.md](SECURITY.md).

## License

SynapseFlow is licensed under the [MIT License](LICENSE).
