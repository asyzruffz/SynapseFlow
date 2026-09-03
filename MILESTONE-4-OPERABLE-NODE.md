# Milestone 4 — Operable node implementation guide

This is the approved implementation order for Roadmap Milestone 4. It turns
SynapseFlow's verified-local and loopback-sharding baseline into an operable
node without adding remote workers, QUIC, peer enrolment, peer discovery, or
worker mutual TLS. Those remain Milestone 5 work.

`synapseflow-node` is a reusable streaming API library. The existing
`synapseflow` CLI is the sole executable and provides the `serve` command that
starts a server through that library. Together they drive the shared
`synapseflow-kernel` workflow, resolve its typed effects, and invoke the one
application-owned generation orchestrator. They must not introduce a second
generation, session, planning, retry, or sampling workflow.

## Approved operating design

- Keycloak is the OpenID Connect issuer and SynapseFlow is an OIDC resource
  server. The node validates access tokens locally from the issuer's discovery
  document and JWKS; it does not call token introspection for every request.
- The public API is `/v1`. No public node API has been delivered yet; any
  development-only interface must never be available from an operational bind.
- A caller creates a durable application-owned session, observes its live SSE
  events, and cancels it idempotently. Session state, ownership, idempotency
  records, and checkpoint references survive process restart. SSE event replay
  after restart is not a Milestone 4 promise: the application safely recovers
  supported work or records an interrupted terminal outcome before delivery.
- Admission is bounded. The node rejects work rather than accumulating an
  unbounded queue.
- Liveness, readiness, and metrics use a separate management listener that is
  private by default. Liveness does not depend on remote services; readiness
  does not flap merely because normal admission capacity is full.
- Metrics and traces use OpenTelemetry conventions. Audit events use a durable
  node-local, structured, rotated sink with explicit retention bounds. Milestone
  6 owns controlled-cohort and network-wide retention/investigation guarantees.
  Prompts, generated text, token text, raw activations/logits, paths,
  credentials, and raw tokens are prohibited from logs, traces, metrics, and
  audit records.

## Completion discipline

Complete each step in order. Do not begin an externally visible API or
configuration surface before its contract and failure behavior are documented.
Each new dependency follows the repository dependency policy. If dependencies
change, the project owner runs `cargo deny check` or `cargo audit` and records
the result; contributors do not bypass or run those checks ad hoc.

## 1. Record the security and compatibility decisions

- [x] Add an ADR for the operable-node boundary: HTTPS or a trusted TLS
  terminating proxy, Keycloak issuer trust, management-listener isolation,
  audit durability, and the Milestone 5 exclusions.
- [x] Add a focused node threat model covering stolen bearer tokens, malformed
  JWTs, JWKS rotation/outage, cross-tenant access, request replay, cancellation
  races, resource exhaustion, SSE disconnects, audit failure, and sensitive
  telemetry leakage.
- [x] Define the public `/v1` API and SSE event schema in documentation before
  implementing handlers. Document every status code, terminal event, stable
  error code, idempotency rule, and versioning/migration policy.
- [x] Define the configuration schema, precedence, defaults, secret handling,
  validation failures, and operational/development profiles before adding a
  parser.

**Done when:** the ADR, threat model, API contract, and configuration contract
have reviewable acceptance tests listed for every trust and failure boundary.

## 2. Establish Keycloak realm and token policy

- [ ] Create one Keycloak OIDC resource-server client with client ID and
  expected audience `synapseflow-node`.
- [ ] Configure an audience mapper so access tokens accepted by the node carry
  that audience.
- [ ] Create and map the client scopes `synapseflow:generate`,
  `synapseflow:cancel:any`, and `synapseflow:observe`. Model-access policy maps
  authorized generation scopes to explicitly configured immutable model
  references; a client cannot nominate a backend, cache entry, or worker.
- [ ] Use Keycloak service accounts and constrained client scopes for
  machine-to-machine callers. If a browser client is introduced later, it uses
  the authorization-code flow with PKCE; direct password grants are not part of
  this node contract.
- [x] Define the accepted access-token profile: one configured asymmetric
  signing algorithm initially (`RS256`), exact issuer, required audience,
  `exp` and `nbf` validation, a bounded clock-skew allowance, and a non-empty
  subject. Reject missing, duplicate, malformed, unexpected, or ambiguous
  claims.
- [x] Design JWKS behavior: cache successful keys with a bounded maximum
  staleness; perform one coordinated refresh for an unknown `kid`; reject
  requests if no suitable key is available. A cached valid key permits normal
  service through a transient Keycloak outage.

**Done when:** a Keycloak realm export or reproducible setup procedure exists,
and tests cover valid tokens, wrong issuer/audience/algorithm, expiry,
not-before, unknown key ID, key rotation, stale keys, missing scope, and model
policy denial.

## 3. Add stable domain and port contracts

- [x] Add payload-free domain values for an authenticated principal, granted
  scopes, admission decision, public session identity/state, and cancellation
  result. Do not put a JWT, Keycloak type, HTTP header, or framework runtime in
  the domain.
- [ ] Add ports for identity verification, authorization/model policy,
  admission accounting, durable session/checkpoint-reference storage,
  active-session lookup/control, durable audit, and telemetry. Port results
  use typed domain outcomes and stable error codes.
- [ ] Extend audit events with principal pseudonym, authorization/admission
  decision, session/trace ID, configured model reference, token count, stable
  failure code, and cancellation result. Do not add payload-bearing fields.
- [ ] Add a framework-independent live-generation output contract and
  cancellation observation. It must deliver ordered tokens and one terminal
  outcome without depending on Tokio streams or SSE types. Migrate both
  `ModelBackend` and `ShardedGenerationRuntime` from their atomic
  `GenerationOutput` result to this contract.
- [ ] Make one profile-neutral application session manager own session creation,
  state transitions, cancellation, deadline propagation, terminal audit, and
  cleanup for both local and sharded execution. It persists ownership,
  idempotency, state, and checkpoint references before externally observable
  transitions. Reuse existing idempotent cancellation and sharding-session
  semantics; do not create a shell-owned session manager.
- [ ] Update kernel events, effects, state, and view models so a client surface
  can start, observe, cancel, and render a generation session without encoding
  authorization, planning, retry, or backend policy in the kernel.
- [ ] Replace shell-issued session IDs and completed-only
  `GenerationExecution` results with application-issued session handles and
  ordered generation events. A shell presents/resolves those events; it never
  invents a session ID after execution has already completed.
- [ ] Define the node workflow registry boundary. The composition root retains
  a kernel instance and subscriber bridge per active client workflow, while the
  application session store remains the sole authority for state, ownership,
  authorization, cancellation, retries, checkpoints, and terminal cleanup.

**Done when:** domain/port/kernel tests prove ordered output, one terminal
state, idempotent cancellation, owner-only cancellation, no duplicate active
session for an idempotency key, and no infrastructure dependency leak into
domain, ports, application, or kernel.

## 4. Create the node library, CLI server command, configuration, and adapters

- [x] Add `synapseflow-node` as a Cargo workspace member containing a library
  only; do not add a `synapseflow-node` executable or `[[bin]]` target. It
  provides the streaming HTTP API shell and its reusable server construction
  surface.
- [ ] Add `synapseflow serve` to the existing `synapseflow-cli` executable as
  the only server-process entry point. The command parses server configuration,
  composes the selected adapters and application use cases, constructs the node
  library, and owns listener startup and graceful process shutdown.
- [ ] Keep the library/CLI boundary explicit: `synapseflow-node` owns HTTP
  routing, request/response/SSE mapping, and kernel workflow driving;
  `synapseflow-cli` owns command parsing, process lifetime, and concrete
  deployment composition. No production adapter may depend on either crate.
- [ ] Add a validated TOML configuration file with this precedence:
  command-line override, `SYNAPSEFLOW_*` environment variable, configuration
  file, then documented safe default.
- [ ] Separate public, management, Keycloak, model-policy, admission, audit,
  telemetry, shutdown, and development settings. Reject unknown keys and
  incompatible combinations with stable diagnostics.
- [ ] Require an HTTPS listener or explicitly configured trusted reverse proxy
  for an operational public listener. Trust forwarded address/protocol headers
  only from configured proxy addresses. Disable permissive CORS by default.
- [ ] Implement a Keycloak OIDC adapter that obtains issuer metadata/JWKS,
  validates the approved access-token profile, and returns only the domain
  principal/scopes.
- [ ] Implement a durable local audit adapter with restrictive filesystem
  permissions, atomic append/flush behavior, size/time rotation, and explicit
  node-local retention limits. Audit persistence failure prevents new admission
  and makes readiness fail; it must not silently discard security events. Do
  not claim the controlled-cohort retention/investigation capabilities reserved
  for Milestone 6.
- [ ] Implement telemetry adapters with non-blocking bounded export. Telemetry
  exporter failure is observable but does not lose or weaken audit behavior.
- [ ] Keep concrete adapter wiring in the CLI `serve` composition path. It
  constructs the node library, which creates a kernel instance per client
  workflow and resolves its effects through the shared orchestrator and
  workflow registry.

**Done when:** invalid or insecure production configuration prevents startup;
development mode cannot accidentally bind the public interface; configuration
precedence, proxy trust, audit failure, and Keycloak adapter behavior have
focused tests.

## 5. Implement the versioned streaming node API

- [ ] Add `POST /v1/sessions`. It authenticates, authorizes, validates input,
  performs atomic admission, creates a session, records durable admission
  audit, and returns `202 Accepted` with an opaque session ID and `Location`.
- [ ] Add `GET /v1/sessions/{session_id}` for the caller that owns the session.
  Return only presentation-safe state; do not expose internal route, cache,
  worker, backend, or checkpoint details.
- [ ] Add `GET /v1/sessions/{session_id}/events` as SSE. Emit `started`, ordered
  `token`, then exactly one of `completed`, `cancelled`, or `failed`.
- [ ] Add `DELETE /v1/sessions/{session_id}`. It is idempotent, verifies the
  owner or `synapseflow:cancel:any`, starts cancellation, and returns a safe
  accepted/terminal result. A cancellation race with normal completion must
  still produce exactly one terminal state and one terminal audit record.
- [ ] Support a bounded `Idempotency-Key` on session creation, scoped by
  principal and canonical request identity. Return the original session result
  for an equivalent replay and reject a conflicting reuse.
- [ ] Ensure client disconnect stops event delivery but does not implicitly
  authorize retry or lose application-owned cleanup. Define whether disconnect
  requests cancellation in the API contract and test that choice.
- [ ] Keep errors minimal and stable. Never reflect token details, Keycloak
  internals, model/cache paths, prompt content, or backend diagnostics.

**Done when:** integration tests verify authentication precedes execution,
scope/model authorization, versioned-routing compatibility, SSE ordering, cancellation,
idempotency, disconnect behavior, body bounds, and privacy-safe errors.

## 6. Enforce bounded admission and request limits

- [ ] Retain strict request-body, prompt, output-token, context, and deadline
  bounds before allocation or model acquisition.
- [ ] Enforce a small pre-authentication IP rate limit and post-authentication
  per-principal token-bucket request limit. Use source address only when it is
  supplied by the trusted transport boundary.
- [ ] Enforce global and per-principal concurrent-session limits. Use a bounded
  queue only when configured; its default is no queue and immediate rejection.
- [ ] Charge or reserve the declared output-token bound at admission and release
  unused reservation at a terminal outcome. Do not let a retry create an
  untracked additional reservation.
- [ ] Return `429 Too Many Requests` and `Retry-After` for rate/quota
  rejection. Return a safe overload outcome for global capacity exhaustion.
- [ ] Derive the initial numerical settings from the load-test environment and
  document them as deployment configuration, not as universal constants.

**Done when:** concurrency, queue, rate, deadline, and reservation tests prove
that no request can exceed its configured bound or leave capacity reserved
after completion, failure, timeout, or cancellation.

## 7. Add health, metrics, traces, and shutdown behavior

- [ ] Expose `/livez`, `/readyz`, and `/metrics` only on the management
  listener. Protect that listener through private network placement or an
  explicit operator policy.
- [ ] Make `/livez` report only process/event-loop health. Make `/readyz`
  require validated configuration, usable Keycloak verification keys, a writable
  audit sink, completed startup, and no shutdown drain. Normal capacity
  saturation is a metric and admission outcome, not readiness flapping.
- [ ] Emit OpenTelemetry traces for request admission, identity verification,
  authorization, session lifetime, model acquisition, execution, cancellation,
  and terminal cleanup. Propagate only validated safe trace context.
- [ ] Emit bounded-cardinality metrics for HTTP request latency/error rate,
  time-to-first-token, generation duration, output tokens, active sessions,
  queue depth, admission rejection, cancellation latency, audit failures, CPU,
  and memory. Never use session ID, principal, prompt, or model reference as a
  metric label.
- [ ] Implement graceful shutdown: become unready, stop new admission, drain
  for the configured interval, cancel remaining sessions, persist terminal
  audits, flush telemetry, then exit.
- [ ] Provide versioned dashboard definitions or documented operator queries for
  admission pressure, request/error latency, active sessions, cancellation,
  audit failures, Keycloak/JWKS health, CPU, and memory.

**Done when:** health semantics, trace correlation, metric cardinality,
telemetry failure, audit privacy, and drain/cancellation behavior are tested
under normal and faulted conditions.

## 8. Add operational documentation and runbooks

- [ ] Document Keycloak client/realm setup, scopes, audience mapper, service
  accounts, token lifetime, key rotation, and expected JWKS outage behavior.
- [ ] Add runbooks for authentication failure, authorization denial, quota
  saturation, Keycloak/JWKS outage, Keycloak signing-key rotation, stale token,
  audit persistence failure, telemetry exporter failure, readiness failure,
  SSE disconnect, cancellation, and graceful drain.
- [ ] Update the API, configuration, compatibility, architecture, and
  operational documents. State clearly that public client authentication is
  separate from Milestone 5 worker mutual TLS.
- [ ] Add migration guidance for users of any retained local-only API.

**Done when:** an operator unfamiliar with the implementation can configure a
least-privilege Keycloak client, distinguish node health from capacity, rotate
keys safely, and restore service without weakening verification or exposing
sensitive data.

## 9. Produce acceptance evidence

- [ ] Add hermetic unit and contract tests for every domain, port, kernel, and
  application rule introduced above.
- [ ] Add node integration tests using controlled Keycloak/JWKS fixtures for
  authentication, authorization, audience, key rotation, denial, and
  cancellation paths.
- [ ] Add load scenarios covering normal service, concurrent authenticated
  callers, per-principal limits, global saturation, slow consumers, SSE
  disconnects, cancellation during execution, audit failure, and Keycloak
  unavailability with a valid cached key.
- [ ] Record platform, hardware, toolchain, configuration limits, request mix,
  concurrency, p50/p95 latency, error/rejection rate, queue depth,
  cancellation latency, CPU, memory, and metric/audit completeness. The record
  contains no prompts, raw outputs, credentials, weights, or activations.
- [ ] Run the repository quality gate and the required compatibility tests. For
  dependency changes, obtain and record the project-owner-run dependency audit
  result before closing the milestone.
- [ ] Run provisioned runtime validation through the production node composition
  root on every Tier-1 platform before making a cross-platform operable-node
  claim. Record any platform limitation separately rather than generalizing a
  single-platform result.
- [ ] Update the implementation-gap and release evidence only after contracts,
  implementation, automated tests, runbooks, and load evidence are complete.

**Milestone acceptance:** the recorded local load test demonstrates observable
latency, error, and resource behavior, and automated integration evidence
covers both authorization and cancellation paths.
