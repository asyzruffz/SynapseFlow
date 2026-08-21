# SynapseFlow

[![CI](https://github.com/asyzruffz/SynapseFlow/actions/workflows/ci.yml/badge.svg)](https://github.com/asyzruffz/SynapseFlow/actions/workflows/ci.yml)

SynapseFlow is a Rust-based distributed LLM inference system. It resolves immutable, signed model manifests; acquires verified shards from approved remote sources; executes layer groups across authenticated workers; and streams generation output through an operable node API and CLI.

## Design

The system is built around versioned model and frame contracts, content-addressed artifacts, mutually authenticated transport, explicit deadlines/cancellation, bounded resource use, replica-aware recovery, and privacy-safe observability.

The documentation entry point is [docs/synapseflow-documentation.md](docs/synapseflow-documentation.md).
New contributors can start with [Contributor onboarding](docs/onboarding.md).

[Implementation gap](docs/implementation-gap.md) is a temporary tracker for differences between this design and the codebase. It is intentionally the only document that records those differences.

## Governance

Contributions follow [CONTRIBUTING.md](CONTRIBUTING.md), the [code-review policy](docs/code-review-policy.md), and the [release process](docs/release-process.md). Vulnerabilities are reported privately under [SECURITY.md](SECURITY.md).

## License

SynapseFlow is licensed under the [MIT License](LICENSE).
