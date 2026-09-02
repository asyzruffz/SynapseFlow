# Operational runbooks

Use these runbooks to restore a known-good state without weakening verification, exposing sensitive data, or silently changing released artifacts. Record the command output, source revision, platform, toolchain, and sanitized error before changing state.

## Dependency or registry failure

1. Preserve `Cargo.lock`; do not run an unlocked update to work around a fetch or resolution failure.
2. Confirm the selected toolchain and run the locked command again:

   ```text
   rustc --version
   cargo fetch --locked
   cargo check --workspace --all-targets --all-features --locked
   ```

3. For a permission or corrupted-cache failure, use an isolated workspace cache:

   ```powershell
   $env:CARGO_HOME = Join-Path (Get-Location) '.cache/cargo'
   cargo fetch --locked
   ```

4. If the isolated cache succeeds, inspect the original Cargo-home permissions, proxy, certificate, and registry credentials with the platform owner. Never commit credentials or a registry override.
5. To discard only the isolated SynapseFlow cache, verify the path first and remove that exact directory:

   ```powershell
   $cache = Join-Path (Get-Location) '.cache/cargo'
   if (Test-Path -LiteralPath $cache) {
       Remove-Item -LiteralPath $cache -Recurse -Force
   }
   ```

6. If the locked graph no longer resolves, stop and open a dependency incident. Update dependencies only through the [dependency-management policy](dependency-management.md), with licence, advisory, and test evidence.

## Artifact verification failure

1. Stop the acquisition or execution request. Do not retry with verification disabled or trust an artifact based on its filename.
2. Record the manifest reference, publisher key identifier, expected and observed hashes, registry endpoint, verification stage, and sanitized diagnostic.
3. Quarantine the affected cache entry; do not promote it or make it available to a worker.
4. Re-run the explicit verified-local CLI or node command with the same immutable `--model` reference, provisioned manifest/artifact paths, cache directory, and publisher public key. It verifies the manifest, compatibility declaration, signature, and provenance before cache use. Use this workflow until `models inspect` is introduced.
5. If signature, key, or manifest trust is in doubt, suspend the publisher/model version, notify the security owner, and follow [SECURITY.md](../SECURITY.md). Restore service only with a newly verified manifest/artifact pair.

## Verified local inference failure

1. Stop the affected CLI invocation; do not expose an ad-hoc network service to work around an error.
2. Retain only the stable error code, manifest reference, platform, backend version, and sanitized command result. Do not record prompts, raw manifest bytes, local artifact/cache paths, signing material, or model output unless the public fixture procedure expressly requires it.
3. For `SYN-MODEL-*` errors, inspect the provisioned manifest/reference/public-key inputs and source artifact against [the verified-local contract](verified-local-inference.md). Re-provision rather than editing a cache object.
4. For `SYN-INFER-*` errors, confirm the supported CPU platform, native build prerequisites, compatibility tuple, and policy/context bounds. Re-run the hermetic tests before retrying the provisioned fixture.
5. Do not disable manifest, signature, hash/size, context, deadline, or body-size validation. Escalate a repeated verification failure through the security reporting process.

## Loopback sharding execution failure

1. Stop the affected local session through its cancellation path; do not
   reroute it to a remote worker or bypass the activation-frame codec.
2. Retain only the session/trace identifiers, stable error code, declared model
   reference and shard IDs, retry/fallback counts, platform, backend version,
   and sanitized command result. Never retain prompt text, raw activations,
   logits, model weights, cache paths, or runtime diagnostics in a ticket.
3. Verify the immutable schema-v2 manifest identity, artifact hash/size,
   `layer_range_v1` strategy, `synapseflow-loom-llama-v1` runtime profile, and
   declared contiguous range before retrying. Re-provision the artifact rather
   than editing a cache object.
4. Reproduce first with the hermetic range/loopback tests, then—only in the
   explicitly provisioned environment—with the Loom acceptance procedure. Keep
   the original deadline and retry budget; do not increase either merely to
   obtain a passing result.
5. If recovery from a checkpoint fails, withdraw the schema-v2 manifest from
   selection and return to the unchanged verified-local workflow.
   Rollback uses a new selection/configuration decision; it never mutates a
   signed manifest or silently substitutes a runtime profile.

## CI failure

1. Open the failing job and capture the job name, source revision, operating system, toolchain, command, and sanitized log.
2. Reproduce the exact local command with `--locked`. For dependency failures, use the registry runbook; do not alter the lockfile merely to make CI green.
3. Treat formatter, Clippy, test, policy, audit, secret-scan, and SBOM failures as merge blockers. Do not disable a job, make a required check optional, or add a broad ignore rule.
4. Apply the smallest reviewed correction, add a regression test when behavior was defective, and rerun the affected local gate.
5. Re-open or rerun CI and attach the successful job links to the pull request. If a runner, action, or hosted service is unavailable, record the incident and keep the merge blocked until an equivalent required check completes.

## Security report

1. Do not put a suspected vulnerability, secret, private artifact, prompt, or raw activation in a public issue, pull request, workflow log, or chat channel.
2. Use the private reporting channel and include impact, affected revision/version, safe reproduction, affected configuration, and suggested mitigation.
3. Restrict access to need-to-know responders, rotate/revoke exposed credentials or keys, and preserve the original evidence securely.
4. Follow the triage, disclosure, regression-test, advisory, and patched-release process in [SECURITY.md](../SECURITY.md).

## Release rollback or withdrawal

1. Stop further publication and determine affected release versions, model manifests, keys, and deployments.
2. Mark the release withdrawn; do not rewrite a released tag or silently replace an artifact.
3. Revoke affected manifests or keys, quarantine affected artifacts, and publish a mitigation or replacement with checksums, SBOM, provenance, and migration notes.
4. Verify rollback/upgrade instructions on both Tier-1 targets and communicate the impact through the approved security/release channels.
5. Preserve the audit record, root cause, corrective actions, and follow-up regression tests. The authoritative release withdrawal policy is in [release process](release-process.md).
