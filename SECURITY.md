# Security policy

## Reporting a vulnerability

Do not report suspected vulnerabilities in public issues, discussions, pull requests, or chat channels. Use the repository's private security-advisory reporting flow. If private reporting is not enabled or unavailable, contact a repository owner privately through the repository hosting service and include the report in the subject.

A report should include the affected version or commit, a clear impact statement, reproduction steps or a proof of concept, affected configuration/model/transport conditions, and any suggested mitigation. Do not attach secrets, private model artifacts, prompts, raw activations, or production credentials.

## Handling process

Maintainers acknowledge, triage, reproduce, mitigate, and coordinate disclosure through the private channel. They credit reporters only with permission. Fixes receive regression tests and release notes; high-impact fixes receive a security advisory and a patched release.

Before the first public release, maintainers must enable a verified private reporting channel and publish a response/contact commitment. This is a release blocker; this policy does not invent an unverified email address or response SLA.

## Supported versions

Security fixes are issued for the latest released minor version and any earlier version explicitly listed in release notes as supported. Development snapshots are not supported deployments and should be updated to the latest commit before reporting.
