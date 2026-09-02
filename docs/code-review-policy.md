# Code review policy

> **Source of truth.** This policy governs review requirements. Change it only
> through an explicit policy redesign.

## Baseline review

Every change requires approval from a maintainer who did not author the change. Review verifies scope, correctness, tests, documentation, compatibility, dependency policy, and the required quality-gate evidence. Authors resolve review comments or record why a suggestion is not applicable.

## Sensitive changes

| Change area | Additional review requirement |
|---|---|
| Protocol, manifest, frame, or serialization | A maintainer responsible for compatibility; golden/negative tests and migration notes. |
| Cryptography, authentication, authorization, secrets, or security controls | A security-designated maintainer; threat/risk analysis and negative tests. |
| Model backend, tokenizer, cache, or artifact acquisition | A runtime/model maintainer; provenance, licence, reference-output, and resource-impact evidence. |
| Scheduler, retry, backpressure, or concurrency | A runtime/distributed-systems maintainer; failure, cancellation, and resource-bound tests. |
| Dependency, build, CI, or release tooling | A maintainer responsible for supply chain/release work; locked-build and policy evidence. |

When an ADR is required, reviewers approve the decision separately from its implementation. Emergency fixes may use an expedited review, but must receive retrospective review, tests, documentation, and changelog/security-advisory updates before the next release.

## Merge criteria

A change may merge only when required reviewers approve it, required checks pass, merge conflicts are resolved, and release/rollback implications are documented. Branch protection should enforce these requirements once CI is enabled.
