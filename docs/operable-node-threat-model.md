# Operable node threat model

This threat model applies to Roadmap Milestone 4's client-facing node. It does
not cover remote-worker transport, peer enrolment, or controlled-peer-network
governance; those are later milestones.

## Assets and trust boundaries

- Client access tokens, Keycloak issuer metadata, and JWKS signing keys.
- Immutable model references, verified artifacts, and runtime configuration.
- Session ownership, idempotency, checkpoint references, and admission state.
- Availability of bounded CPU, memory, queue, file-descriptor, and audit
  storage resources.
- Privacy-safe audit events, logs, traces, and metrics.

The public listener is untrusted. The management listener is private by
deployment policy. A TLS-terminating proxy is trusted only when its addresses
are explicitly configured. Activation frames remain a separate data plane and
never carry bearer credentials, authorization, quota, or client identity.

## Threats and required controls

| Threat | Required control |
|---|---|
| Stolen, malformed, expired, wrong-issuer, wrong-audience, or algorithm-confused token | Validate the configured access-token profile locally; accept only configured asymmetric algorithms and Keycloak JWKS keys. |
| Signing-key rotation or Keycloak outage | Bound JWKS cache staleness, coordinate unknown-`kid` refresh, and reject new authentication when no suitable valid key is available. |
| Cross-principal session access or cancellation | Bind every session to a principal pseudonym; require ownership or the `synapseflow:cancel:any` scope for status, events, and cancellation. |
| Model/backend/worker selection by a caller | Resolve only immutable model references permitted by application-owned model policy; never accept backend, cache, worker, or route selection from the API. |
| Duplicate create/retry work | Bind a bounded idempotency key to principal plus canonical request identity; store its result durably. |
| CPU, memory, queue, connection, or audit-disk exhaustion | Enforce input/context/output/deadline/rate/concurrency/queue/storage limits before work; reject rather than queue unbounded work. |
| Cancellation/completion race | Use application-owned state transitions and exactly one terminal audit record/outcome. |
| SSE disconnect or process restart | Stop only the subscriber on disconnect; retain application cleanup. Persist session state and record interrupted terminal outcomes when recovery is unsupported. |
| Sensitive data in operational signals | Prohibit prompt, generated text, raw token text, activations, logits, model/cache paths, credentials, raw JWTs, and high-cardinality identifiers from logs, metrics, traces, and audit records. |
| Spoofed forwarded client address or protocol | Trust forwarded headers only from configured proxy addresses; otherwise use the direct peer address and reject insecure operational binds. |
| Audit loss or tampering by normal service operation | Use restrictive permissions, atomic append/flush, rotation, bounded local retention, readiness degradation, and admission failure on persistence failure. |

## Security invariants

1. Authentication and authorization finish before artifact acquisition or
   execution.
2. The application control plane, not the API shell or a worker, owns admission,
   session transitions, cancellation, retry, checkpoint selection, and cleanup.
3. A completed session has exactly one terminal result and one terminal audit
   outcome.
4. A client cannot use a frame control message to establish identity, change
   authorization, select a route, extend a deadline, or bypass limits.
5. Health, metrics, audit, and error responses are useful without exposing
   request payloads or secrets.
