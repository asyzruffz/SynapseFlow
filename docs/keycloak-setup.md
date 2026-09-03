# Keycloak setup for an operable SynapseFlow node

This procedure configures the Milestone 4 Keycloak boundary. It creates the
single resource-server client accepted by a SynapseFlow node and a
least-privilege service-account caller. It neither configures remote workers
nor enables password grants, token exchange, or browser clients.

The procedure follows Keycloak's current guidance for [service
accounts](https://www.keycloak.org/docs/latest/server_admin/#_service_accounts)
and its [Audience protocol mapper](https://www.keycloak.org/docs/latest/server_admin/#_audience).
Record the resulting issuer URL and client secret in deployment secret storage;
do not commit them, include them in command history, or put them in node TOML.

## 1. Create the realm and resource server

1. Create a production realm named `synapseflow` (or record the deployment's
   different realm name).
2. Create an OpenID Connect client whose **Client ID** is
   `synapseflow-node`.
3. Enable client authentication and service accounts. Disable Standard flow,
   Direct access grants, Implicit flow, device authorization, and token
   exchange. Do not configure redirect URIs or web origins on this
   machine-to-machine client.
4. Disable **Full Scope Allowed**. Token roles and scopes must be assigned
   explicitly.

The node is configured with the exact issuer
`https://<keycloak-host>/realms/synapseflow`, expected audience
`synapseflow-node`, and `RS256`. Its issuer metadata and JSON Web Key Set
(JWKS) URLs are discovered from that issuer; the node does not receive a
Keycloak administrator credential and never introspects each request.

## 2. Publish the required access-token claims

Create these three OpenID Connect client scopes, configure each scope to be
included in the access-token `scope` claim, and assign them only to callers
that need the capability:

| Client scope | Grants |
|---|---|
| `synapseflow:generate` | Create a session for a model permitted by node model policy. |
| `synapseflow:cancel:any` | Cancel a session owned by another principal. |
| `synapseflow:observe` | Access explicitly configured operator observation surfaces; it does not grant generation. |

On the `synapseflow-node` client's dedicated client scope, add an **Audience**
protocol mapper with **Included Client Audience** set to `synapseflow-node` and
**Add to access token** enabled. Attach the mapper's scope as a default scope
for this client. The accepted `aud` claim must contain that exact value; the
node never treats a client ID, `azp`, or a token's presence as a substitute for
the audience check.

## 3. Provision a least-privilege machine caller

1. Create a separate confidential OpenID Connect client for each machine
   caller, enable only client authentication and service accounts, and disable
   Full Scope Allowed.
2. Attach `synapseflow:generate` only when the caller needs generation.
   Attach `synapseflow:cancel:any` and `synapseflow:observe` separately and
   only for an operator client with that responsibility.
3. Grant the matching scope/role mappings to the service account. Use a
   client secret or a stronger approved client-authentication method from the
   deployment secret store.
4. Obtain tokens through the realm token endpoint using `grant_type=client_credentials`.

The configured model policy is a separate SynapseFlow deployment setting. It
maps `synapseflow:generate` to explicitly allowed immutable model references;
neither a Keycloak client nor a request can choose an artifact URL, runtime
backend, cache entry, route, or worker.

If a browser client is added in a later change, create a separate client that
uses authorization-code flow with PKCE. Do not enable direct password grants
to make this procedure work.

## 4. Verify and operate the token profile

Before pointing a node at the realm, inspect a token from every caller class.
It must contain one non-empty subject, the exact issuer, an `aud` value of
`synapseflow-node`, `exp`, `nbf`, and the intended scopes. The node accepts
only `RS256` initially; it rejects unsigned, symmetric, unexpected,
ambiguous, expired, or not-yet-valid tokens.

Key rotation keeps previous public signing keys published until every issued
token under the old key has expired and the node's configured JWKS maximum
staleness has elapsed. The node caches successful JWKS keys for its bounded
staleness interval and makes one coordinated refresh for an unknown `kid`. A
transient Keycloak outage is tolerated only while a suitable cached key remains
fresh; otherwise authentication fails closed.

To reproduce the realm on another Keycloak installation, repeat the steps
above or export the realm with Keycloak's supported JSON import/export
procedure. Realm exports are configuration artifacts, not backups: Keycloak
documents that exports do not include persisted sessions, workflow state, or
revoked tokens. See [Importing and exporting realms](https://www.keycloak.org/server/importExport).
