# Architecture

## Goal

Provide a serious, public-ready path for syncing Vaultwarden-backed Bitwarden
Password Manager items into Kubernetes without making Vaultwarden item metadata
the Kubernetes control plane.

## Initial Shape

The first target is an External Secrets Operator webhook provider:

```text
ExternalSecret
  -> SecretStore(provider.webhook)
  -> vwso-eso-webhook
  -> Vaultwarden-compatible API
  -> decrypted item fields
  -> ESO-managed Kubernetes Secret
```

This project should own:

- Vaultwarden-compatible authentication.
- Local decryption and field extraction.
- Master-password user-key unlock.
- Vaultwarden API-key login and sync.
- Provider-level caching and rate limiting.
- Redacted logs, metrics, and health checks.
- A small HTTP contract usable by ESO's generic webhook provider.

This project should not own initially:

- Kubernetes Secret ownership.
- Refresh policy.
- Deletion policy.
- Target Secret templating.
- Deployment restarts.

Those are already modeled by External Secrets Operator.

## Why ESO First

External Secrets Operator already provides the API surface Kubernetes users
expect: `SecretStore`, `ExternalSecret`, refresh intervals, creation policies,
deletion policies, status conditions, and templating. Reusing it avoids a
premature CRD and gives the project a narrow, testable first milestone.

## Later Options

- Native External Secrets Operator provider if the webhook API becomes too
  limiting.
- Native Rust controller with `kube-rs` if we need Vaultwarden-specific CRDs.
- Secrets Store CSI provider only if file mount semantics become a first-class
  target.
