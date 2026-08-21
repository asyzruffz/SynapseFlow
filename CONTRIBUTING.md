# Contributing to SynapseFlow

## Before you begin

Read the [architecture documentation](docs/architecture.md), [development guide](docs/development.md), [dependency policy](docs/dependency-management.md), and the accepted [ADRs](docs/adr/README.md). Contributions must preserve the documented dependency direction, error/input boundaries, model-artifact policy, and compatibility commitments.

Do not include model weights, tokenizer files, benchmarks, credentials, prompts, or activation dumps in a change. Report suspected vulnerabilities through [SECURITY.md](SECURITY.md), not a public issue.

## Contribution workflow

1. Open or reference an issue that states the problem, user impact, and intended scope.
2. Create a focused branch and keep unrelated formatting or refactors out of the change.
3. For a durable architectural, protocol, backend, security, authorization, or scheduler choice, add an ADR before or with the implementation.
4. Implement the change with typed errors, bounded input handling, and privacy-safe telemetry as applicable.
5. Add positive, negative, and regression tests appropriate to the affected contract.
6. Update public documentation, compatibility notes, and the changelog when behavior changes.
7. Run the quality gate in [Development](docs/development.md) and include the results in the pull request.

## Pull-request requirements

Every pull request includes a concise summary, test evidence, risk assessment, rollback notes where applicable, and links to its issue/ADR. Reviewers may request smaller changes when a pull request mixes independent concerns.

Changes affecting public APIs, manifests, frames, authentication, authorization, cryptography, artifact acquisition, or release automation require the review described in the [code-review policy](docs/code-review-policy.md). Releases follow the [release process](docs/release-process.md).

## Coding and documentation standards

- Keep library interfaces typed and fallible; do not panic on user, peer, or artifact input.
- Treat compatibility fixtures and golden protocol vectors as production contracts.
- Pin/review dependencies according to the dependency policy.
- Write clear public rustdoc and update the documentation index when adding a durable capability.
- Use the [changelog format](CHANGELOG.md) for user-visible changes.
