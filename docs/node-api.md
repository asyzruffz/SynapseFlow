# Node API v1

This document defines the first public SynapseFlow node API. It is implemented
by the `synapseflow-node` library and started only through `synapseflow serve`.
All endpoints are served over HTTPS or through a configured trusted TLS proxy.

## Authentication and authorization

Every public endpoint requires an OAuth 2.0 bearer access token issued by the
configured Keycloak realm. The node validates the token before it acquires a
model or starts a session.

`synapseflow:generate` permits a model only when the configured model-access
policy permits that immutable reference. A session owner may read, observe, and
cancel their own session. `synapseflow:cancel:any` permits cross-owner
cancellation. `synapseflow:observe` is reserved for explicitly configured
operator observation surfaces; it does not grant generation access.

## Session endpoints

| Method and path | Behavior |
|---|---|
| `POST /v1/sessions` | Validates, authorizes, admits, and starts a generation. Returns `202 Accepted`, `Location`, and a safe session representation. |
| `GET /v1/sessions/{session_id}` | Returns the safe state for the owning principal. |
| `GET /v1/sessions/{session_id}/events` | Delivers a live Server-Sent Events stream for the owning principal. |
| `DELETE /v1/sessions/{session_id}` | Requests idempotent cancellation for the owner or a caller with `synapseflow:cancel:any`. |

The create request contains an immutable `model` reference, `prompt`, sampling
policy, and optional bounded `deadline_ms`. It never accepts a model URL,
backend selection, worker identity, cache path, route, checkpoint, or runtime
profile.

`Idempotency-Key` is optional but recommended for `POST /v1/sessions`. It is
bound to the authenticated principal and canonical request identity. Equivalent
replays return the original session result; conflicting reuse is rejected.

## Streaming events

The stream emits `started`, zero or more ordered `token` events, then exactly
one terminal event: `completed`, `cancelled`, or `failed`. Terminal events carry
only a session identifier and safe outcome metadata. `token` events carry the
ordered token ID and decoded text needed by the caller; the node must not copy
that content into telemetry or audit records.

The stream is live-only. A disconnected subscriber may reconnect to the status
endpoint and a new live stream when the session is still active, but event
replay is not a v1 guarantee. Process restart follows the durable session
recovery/interruption policy rather than replaying prior SSE events.

## Error and admission behavior

Errors are JSON objects containing only a stable SynapseFlow error code and a
safe message. Authentication and authorization failures never reveal token,
scope, model-policy, cache, or worker details.

Request body, prompt, context, output-token, deadline, rate, concurrency, and
queue limits are checked before execution. Rate/quota rejection returns `429`
with `Retry-After`. Global overload returns a safe bounded failure. Repeated
`DELETE` calls are idempotent and do not create additional work.

## Management endpoints

`/livez`, `/readyz`, and `/metrics` are available only through the configured
private management listener. They are not part of the public client API.
