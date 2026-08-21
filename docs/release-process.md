# Release process

## Preconditions

A release candidate begins from a reviewed commit with a clean worktree and the pinned toolchain. The release owner confirms the version against [ADR 0002](adr/0002-release-and-compatibility-policy.md), updates package metadata and [CHANGELOG.md](../CHANGELOG.md), and prepares migration/security notes where required.

## Verification

Run the Tier-1 quality gate, locked dependency resolution, dependency-policy checks, security audit, secret scan, compatibility tests, and release packaging checks. Record the source revision, `Cargo.lock`, toolchain, supported target results, and known limitations. Do not release if a required check, security reporting channel, or licence/provenance review is incomplete.

## SBOM and provenance

Generate a CycloneDX or SPDX SBOM from the locked release dependency graph using a version-pinned release tool. Attach the SBOM, source revision, checksums, and build provenance to the release. Retain the exact configuration, tool versions, command output, and artifact hashes needed to reproduce the SBOM.

The release includes model-manifest provenance separately from the binary SBOM: publisher key identifier, manifest hash, model/artifact hashes, licence, and approved registry source. Never package model weights, private credentials, prompts, or activation data in a source or binary release.

## Publish and follow up

Create a signed version tag, publish approved crate/node artifacts, and attach release notes, migration notes, SBOM, checksums, and security advisories. Verify installation and rollback instructions on every Tier-1 platform. Announce only after artifacts and documentation are available.

If a release must be withdrawn, mark it as withdrawn, revoke affected manifests/keys when necessary, publish a replacement or mitigation, and preserve the audit record. Do not rewrite released tags or silently replace artifacts.
