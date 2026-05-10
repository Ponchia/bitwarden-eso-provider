# Bitwarden ESO Provider

Bitwarden ESO Provider lets External Secrets Operator resolve Kubernetes Secrets
from Bitwarden Password Manager and Vaultwarden vault items.

It uses the Bitwarden Password Manager vault protocol: user API-key login,
master-password unlock, sync, and local item decryption. It is not a Bitwarden
Secrets Manager (`bws`) provider.

The production path is deliberately narrow:

```text
Bitwarden Cloud / Vaultwarden -> bitwarden-eso-provider -> External Secrets Operator -> Kubernetes Secret
```

External Secrets Operator owns refresh policy, deletion behavior, status
conditions, templating, and GitOps integration. This project owns only the
Bitwarden-compatible authentication, decryption, and webhook resolution layer.

## Status

Pre-1.0, but already live-tested.

Verified paths:

- Vaultwarden/self-hosted single-origin endpoints.
- Bitwarden Cloud split endpoints.
- Direct Rust live resolution against real vault data.
- k3s + External Secrets Operator smoke tests covering initial sync, forced
  refresh, target Secret recreation, webhook restart, and expected not-found
  errors.

Treat crate APIs, chart values, and metadata keys as unstable until the first
tagged release.

## Features

- Rust workspace with a small ESO webhook binary and reusable client crates.
- API-key login and sync for Bitwarden-compatible Password Manager servers.
- Master-password user-key unlock for PBKDF2-SHA256 and Argon2id accounts.
- Local encrypted string and vault item decryption.
- Whole-item extraction or one-field extraction through ESO `remoteRef`.
- In-memory sync cache with explicit TTL and single-flight refresh behavior.
- Dedicated `/livez`, `/readyz`, and `/metrics` endpoints with redacted
  Prometheus-format runtime, HTTP, and resolution metrics.
- Helm chart, ESO manifests, live smoke test script, architecture notes, threat
  model, and release checklist.

## Design Principles

- Kubernetes manifests declare what is synced. Vault item metadata never decides
  target namespaces or Secret names.
- No Bitwarden, Vaultwarden, 1Password, or Kubernetes source code is vendored.
- TLS verification is on by default. HTTP is accepted only for localhost tests.
- Logs must not contain secret values, master passwords, API tokens, or derived
  keys.
- Restarts and target Secret recreation use Kubernetes-native behavior.
- The webhook itself does not need Kubernetes API RBAC.

## Layout

```text
crates/bweso-core              Shared request/response and secret document types
crates/bweso-bitwarden         Bitwarden-compatible API, crypto, and resolver
crates/bitwarden-eso-provider  HTTP adapter for ESO's webhook provider
deploy/eso                    Example SecretStore and ExternalSecret manifests
deploy/helm                   Helm chart
docs                          Architecture, install, compatibility, testing
references                    Notes pointing to local reference checkouts
```

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

Run locally against Vaultwarden or a self-hosted single-origin Bitwarden server:

```bash
BWESO_SINGLE_ORIGIN_URL="https://vaultwarden.example.com" \
BWESO_CLIENT_ID="user.<uuid>" \
BWESO_CLIENT_SECRET="..." \
BWESO_MASTER_PASSWORD="..." \
BWESO_CACHE_TTL_SECONDS=60 \
cargo run -p bitwarden-eso-provider -- --listen 127.0.0.1:8080
```

Run locally against Bitwarden Cloud:

```bash
BWESO_IDENTITY_URL="https://identity.bitwarden.com" \
BWESO_API_URL="https://api.bitwarden.com" \
BWESO_CLIENT_ID="user.<uuid>" \
BWESO_CLIENT_SECRET="..." \
BWESO_MASTER_PASSWORD="..." \
BWESO_CACHE_TTL_SECONDS=60 \
cargo run -p bitwarden-eso-provider -- --listen 127.0.0.1:8080
```

## Helm Install

```bash
kubectl create namespace bweso-system
kubectl -n bweso-system create secret generic bweso-credentials \
  --from-literal=client-id='user.<uuid>' \
  --from-literal=client-secret='...' \
  --from-literal=master-password='...'

helm upgrade --install bweso ./deploy/helm/bitwarden-eso-provider \
  --namespace bweso-system \
  --set-string config.singleOriginUrl='https://vaultwarden.example.com' \
  --set-string credentials.existingSecret.name=bweso-credentials
```

For Bitwarden Cloud, use `config.identityUrl=https://identity.bitwarden.com` and
`config.apiUrl=https://api.bitwarden.com` instead of `config.singleOriginUrl`.

Then create an ESO `SecretStore` and `ExternalSecret` using the examples under
[`deploy/eso`](deploy/eso).

Compatibility details are in [`docs/compatibility.md`](docs/compatibility.md).
Operational metrics and probe details are in
[`docs/operations/observability.md`](docs/operations/observability.md).
Live smoke-test instructions are in [`docs/live-testing.md`](docs/live-testing.md).

## License

Apache-2.0. Keep this repo free of copied code from reference projects unless a
license review explicitly approves it.
