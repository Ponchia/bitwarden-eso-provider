# Threat Model

## Assets

- Vaultwarden or Bitwarden user API key client ID and client secret.
- Vaultwarden or Bitwarden user master password.
- Derived vault encryption keys.
- Decrypted item field values.
- Kubernetes Secrets created by External Secrets Operator.
- Provider logs, metrics, and caches.

## Trust Boundaries

- Kubernetes API server to provider pod.
- ESO controller to provider webhook.
- Provider pod to Bitwarden-compatible HTTPS endpoint.
- Provider memory and local cache.
- Kubernetes Secret storage and etcd encryption.
- ESO to provider webhook traffic. The default Helm chart exposes a ClusterIP
  HTTP service for ESO; this hop carries resolved secret values and relies on
  Kubernetes network isolation, bearer-token auth, and optional NetworkPolicy.

## Initial Attacker Capabilities

- Read provider logs.
- Read Kubernetes objects in namespaces where RBAC allows it.
- Compromise an application namespace.
- Intercept network traffic if TLS is misconfigured.
- Submit or modify `ExternalSecret` manifests if GitOps or RBAC allows it.

## Security Requirements

- TLS verification is mandatory by default.
- The provider Service must stay cluster-internal unless it is placed behind a
  TLS or mTLS terminating mesh, ingress, or gateway. Use that protected HTTPS
  endpoint for the ESO webhook URL when pod-network traffic is not trusted.
- The provider must not expose vault item IDs, item names, requested properties,
  secret values, decrypted vault content, API tokens, master passwords, or
  derived keys through logs, metrics, or public error responses.
- Vaultwarden/Bitwarden credentials must come from a Kubernetes Secret or external
  workload identity mechanism, not command-line args.
- The default deployment must not need Kubernetes API RBAC. Namespace access is
  controlled by ESO `SecretStore`/`ClusterSecretStore` placement, Kubernetes
  RBAC, and optional NetworkPolicy.
- A compromised application namespace must not allow arbitrary vault item
  reads unless its `SecretStore` credentials and provider selector policy
  explicitly allow that.
- Deletion must be controlled by ESO policies, not hidden provider behavior.
- Provider-side selector policy must deny by default when configured and must
  return redacted `403` responses for disallowed `remoteRef.key` values.

## Recommended Isolation Model

- Use one dedicated Bitwarden/Vaultwarden user API key per namespace or trust
  boundary.
- Use namespace-local `SecretStore` resources by default.
- Put only the webhook bearer token in workload namespaces. Keep client ID,
  client secret, and master password in the provider namespace or an equivalent
  runtime secret boundary.
- Configure `selectorPolicy.allowedKeys` or `selectorPolicy.allowedKeyPrefixes`
  on the provider Deployment when the credential can see more vault items than
  the namespace should consume.
- Treat provider selector policy as item-key scoped. It does not enforce
  per-property authorization inside an allowed item, so use exact `id:`
  allowlists and separate provider credentials when different namespaces should
  see different fields.
- Treat `ClusterSecretStore` as a deliberate shared trust boundary. Kubernetes
  RBAC can control who may reference it, but the Bitwarden/Vaultwarden account
  and selector policy still define the data that can be read.

## Unsupported Surfaces For v0.1.0

- Bitwarden Secrets Manager (`bws`) is a separate product surface and is not
  handled by this provider.
- Shared organization vault items fail explicitly until organization-key
  decryption is implemented and live-tested for Vaultwarden and Bitwarden
  Cloud.
- Attachment extraction fails explicitly. Use notes or custom fields for
  multiline material until attachment download/decryption is implemented.

## Open Questions

- Whether Bitwarden Password Manager SDK internals can be reused legally and
  practically.
- Whether item revision metadata is sufficient for efficient cache invalidation.
