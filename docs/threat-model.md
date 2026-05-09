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

## Initial Attacker Capabilities

- Read provider logs.
- Read Kubernetes objects in namespaces where RBAC allows it.
- Compromise an application namespace.
- Intercept network traffic if TLS is misconfigured.
- Submit or modify `ExternalSecret` manifests if GitOps or RBAC allows it.

## Security Requirements

- TLS verification is mandatory by default.
- The provider must not log secret values or decrypted vault content.
- Vaultwarden/Bitwarden credentials must come from a Kubernetes Secret or external
  workload identity mechanism, not command-line args.
- The default deployment must watch or serve only configured namespaces.
- A compromised application namespace must not allow arbitrary vault item
  reads unless its `SecretStore` credentials explicitly allow that.
- Deletion must be controlled by ESO policies, not hidden provider behavior.

## Open Questions

- Whether a dedicated Vaultwarden or Bitwarden user can be constrained tightly
  enough through organizations and collections for multi-namespace use.
- Whether Bitwarden Password Manager SDK internals can be reused legally and
  practically.
- Whether item revision metadata is sufficient for efficient cache invalidation.
- Whether ESO webhook responses should return one field at a time or whole item
  documents.
