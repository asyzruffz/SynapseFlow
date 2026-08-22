# SynapseFlow

[![CI](https://github.com/asyzruffz/SynapseFlow/actions/workflows/ci.yml/badge.svg)](https://github.com/asyzruffz/SynapseFlow/actions/workflows/ci.yml)

SynapseFlow is planned to be a Rust-based distributed LLM inference system. It resolves immutable, signed model manifests; acquires verified shards from approved remote sources; executes layer groups across authenticated workers; and streams generation output through an operable node API and CLI.

However, SynapseFlow currently provides verified local inference for one narrow CPU-only GGUF/Llama compatibility tuple. A caller selects a signed immutable manifest reference, the local runtime verifies and content-addresses the provisioned artifact, and the shared application workflow serves a CLI and a loopback-only API.

## Design

The system will be built around versioned model and frame contracts, content-addressed artifacts, mutually authenticated transport, explicit deadlines/cancellation, bounded resource use, replica-aware recovery, and privacy-safe observability.

The delivered Milestone 2 slice has versioned manifest contracts, content-addressed local caching, bounded local API input, typed errors, seeded generation, deadlines, and privacy-safe diagnostics. Sharding, remote workers, QUIC, authentication, distributed cancellation/recovery, and observability are planned roadmap work—not current product behavior.

The documentation entry point is [docs/synapseflow-documentation.md](docs/synapseflow-documentation.md).
New contributors can start with [Contributor onboarding](docs/onboarding.md).
Operators can run the supported local workflow through [the CLI and API guide](docs/cli.md).

[Milestone 2 tracker](MILESTONE-2-VERIFIED-LOCAL-INFERENCE.md) records current delivery evidence. [Implementation gap](docs/implementation-gap.md) is refreshed only after that milestone is fully evidenced.

## Governance

Contributions follow [CONTRIBUTING.md](CONTRIBUTING.md), the [code-review policy](docs/code-review-policy.md), and the [release process](docs/release-process.md). Vulnerabilities are reported privately under [SECURITY.md](SECURITY.md).

## License

SynapseFlow is licensed under the [MIT License](LICENSE).
