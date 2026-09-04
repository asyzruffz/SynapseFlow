# Provision the SynapseFlow Keycloak realm

`synapseflow-realm.json` is a reviewable import baseline for the Milestone 4
resource-server client. It contains no administrator account, client secret,
user, service-account role assignment, or model policy; those values are
deployment-specific and must not be committed.

## Import and configure

1. Review the JSON in a non-production Keycloak environment first. Import it
   as realm `synapseflow` with Keycloak's supported realm-import mechanism.
   Do not use a realm export as a backup/restore substitute.
2. Open `Clients` → `synapseflow-node`. Keep client authentication enabled and
   service accounts disabled: this is the resource-server audience, not a
   caller credential. Keep Standard flow, Direct access grants, Implicit flow,
   device authorization, token exchange, redirect URIs, web origins, and Full
   Scope Allowed disabled.
3. Under `Client scopes`, confirm that the imported
   `synapseflow-node-audience` Audience mapper adds `synapseflow-node` to the
   **access token** `aud` claim. It deliberately does not add itself to the
   token's `scope` claim.
4. Under `Client scopes`, confirm the three imported scopes exist and emit into
   the access-token `scope` claim:
   `synapseflow:generate`, `synapseflow:cancel:any`, and
   `synapseflow:observe`.
5. Create a separate confidential client for each machine caller. Enable only
   client authentication and service accounts, keep Full Scope Allowed off,
   attach `synapseflow-node-audience` as a default client scope, attach only
   the SynapseFlow authorization scopes it may request, and grant only the
   corresponding scope/role mappings it needs. Do not reuse the
   `synapseflow-node` client secret as a caller credential.
6. For each service account, assign `synapseflow:generate` only when it needs
   generation. Assign `synapseflow:cancel:any` and `synapseflow:observe` only
   to a separately controlled operator client.
7. Store all client secrets in the deployment secret manager. Configure the
   node with the exact HTTPS issuer
   `https://<keycloak-host>/realms/synapseflow`, audience `synapseflow-node`,
   and initial signing algorithm `RS256`.
8. Request a client-credentials token from the caller client. Inspect it in a
   secure environment: it must have the exact `iss`, include
   `synapseflow-node` in `aud`, have `exp` and `nbf`, a non-empty `sub`, and
   only the intended SynapseFlow scopes. Never paste the token into logs, an
   issue tracker, or this repository.

The procedure follows Keycloak's current guidance for [client scopes, audience
mappers, and service accounts](https://www.keycloak.org/docs/latest/server_admin/).
