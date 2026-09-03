# ADR 0007: Operable node composition and security boundary

**Status:** Accepted

## Context

The integrated baseline has a portable Crux kernel, an application-owned
generation orchestrator, and local/loopback runtime adapters. It has no public
HTTP node, client identity, authorization, bounded multi-user admission,
production telemetry, or durable operational session store.

Roadmap Milestone 4 requires an operable node without pulling remote workers,
QUIC, peer enrolment, or worker mutual TLS forward from Milestone 5. The node
must also preserve the kernel-and-shell architecture: the kernel remains
portable, application owns execution decisions, adapters own infrastructure,
and only composition code wires them together.

## Decision

- Add `synapseflow-node` as a library-only workspace member. It provides the
  versioned HTTP/SSE shell, drives a kernel workflow, and maps safe application
  outcomes to the public API. It has no executable target.
- Add `synapseflow serve` to the existing `synapseflow` CLI as the only server
  process entry point. The command parses server configuration, wires concrete
  adapters, starts listeners, and owns graceful process shutdown.
- Publish the first public API as `/v1`. A generation is represented by an
  application-owned durable session: `POST /v1/sessions`, `GET
  /v1/sessions/{id}`, `GET /v1/sessions/{id}/events`, and `DELETE
  /v1/sessions/{id}`. SSE event replay after restart is not promised.
- Authenticate public callers as Keycloak OIDC access-token bearers. The node
  validates a configured issuer, audience, asymmetric algorithm allowlist,
  signature, expiry, not-before value, and non-empty subject using cached JWKS
  keys. It does not make per-request introspection calls.
- Authorize generation through configured Keycloak scopes and explicit
  immutable-model policy. The application control plane owns authorization,
  admission, session state, cancellation, retries, checkpoints, and cleanup;
  no such decision is placed in an activation frame or delegated to a worker.
- Require bounded request body, context, deadline, rate, concurrency, and
  queue limits. Capacity exhaustion rejects work rather than allowing an
  unbounded queue.
- Persist session ownership, idempotency, state, and checkpoint references.
  On restart, the application recovers only supported work; otherwise it
  durably records an interrupted terminal outcome and releases reservations.
- Run health and metrics endpoints on a separately configured private
  management listener. The liveness check is process-local; readiness requires
  usable authentication keys, durable audit availability, completed startup,
  and no drain state.
- Use OpenTelemetry-compatible traces and bounded-cardinality metrics. Persist
  privacy-safe audit events in a node-local rotated store. Node-local retention
  is Milestone 4 work; controlled-cohort/network-wide retention and
  investigation guarantees remain Milestone 6 work.

## Consequences

- The CLI remains the only binary to install and operate, while tests or later
  shells can reuse the node library without spawning a subprocess.
- Existing completed-output ports and shell-issued session IDs require a
  deliberate migration to application-issued session handles, ordered output
  events, and cancellation observation for both local and sharded profiles.
- New identity, persistence, audit, telemetry, and HTTP dependencies remain in
  node/adapters or CLI composition. Domain, ports, application, and kernel do
  not depend on Keycloak, JWT, HTTP, Tokio, or exporter types.
- The node exposes only client-facing HTTPS API behavior. Remote worker
  authentication and QUIC remain a later data-plane capability.

## Superseding conditions

Supersede this decision if the project adopts a different client identity
provider, changes the public API versioning/lifecycle model, introduces durable
cross-node session execution, or changes the CLI-only server-process boundary.
