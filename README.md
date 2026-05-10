# Vaultwarden Secrets Operator

Vaultwarden Secrets Operator is an experimental Rust implementation for using a
Vaultwarden or Bitwarden Password Manager vault as a Kubernetes secret source.

The initial target is not a bespoke Kubernetes controller. The first production
path is an External Secrets Operator webhook provider:

```text
Vaultwarden/Bitwarden -> vwso-eso-webhook -> External Secrets Operator -> Kubernetes Secret
```

This keeps Kubernetes ownership, refresh policy, deletion behavior, templating,
status conditions, and GitOps integration in External Secrets Operator while this
project focuses on one narrow responsibility: safely authenticating to
Bitwarden-compatible Password Manager APIs and resolving encrypted vault items.

## Status

Experimental implementation. The core Vaultwarden + External Secrets Operator
path has been live-tested against a k3s cluster, including initial sync, forced
refresh, target Secret recreation, webhook restart, and expected not-found
errors. Treat the public API and chart values as pre-1.0 until the first tagged
release.

The current repository contains:

- A Rust workspace with core model, Bitwarden-compatible client boundary, and
  ESO webhook entrypoint crates.
- Bitwarden-compatible authenticated encrypted string decryption.
- Master-password user-key unlock for PBKDF2-SHA256 and Argon2id accounts.
- A tested API-key login and sync client path backed by local fake servers for
  both Vaultwarden-style single-origin and Bitwarden-style split endpoints,
  wired into the ESO webhook runtime through environment configuration.
- In-memory sync caching with explicit TTL and single-flight refresh behavior.
- Architecture, threat-model, and reference notes.
- Example External Secrets Operator manifests.
- A Helm chart for deploying the webhook.
- A repeatable live ESO smoke-test script.
- CI scaffolding for formatting, clippy, unit tests, fake-server tests, and an
  opt-in live Bitwarden-compatible smoke test.

Bitwarden Cloud split endpoints are covered by fake-server tests, but still need
a live Bitwarden Cloud validation account before this project should claim full
cloud compatibility.

## Design Principles

- Kubernetes manifests declare what is synced. Vaultwarden items do not decide
  target namespaces.
- No Vaultwarden, Bitwarden, 1Password, or Kubernetes source code is vendored.
  Reference repositories live outside this repo.
- TLS verification is on by default and must not be silently bypassed.
- The provider must not log secret values, decrypted item content, master
  passwords, API tokens, or derived keys.
- Deletes and restarts must use Kubernetes-native ownership or explicit opt-in
  policy.
- The first public version should be usable without cluster-admin permissions.

## Workspace

```text
crates/vwso-core          Shared request/response and secret document types
crates/vwso-vaultwarden   Bitwarden-compatible API and crypto boundary
crates/vwso-eso-webhook   HTTP adapter for External Secrets Operator webhook
deploy/eso                Example SecretStore and ExternalSecret manifests
docs                      Architecture, decisions, threat model, research
references                Notes pointing to local reference checkouts
```

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

Run the webhook locally:

```bash
VWSO_VAULTWARDEN_URL="https://vaultwarden.example.com" \
VWSO_CLIENT_ID="user.<uuid>" \
VWSO_CLIENT_SECRET="..." \
VWSO_MASTER_PASSWORD="..." \
VWSO_CACHE_TTL_SECONDS=60 \
cargo run -p vwso-eso-webhook -- --listen 127.0.0.1:8080
```

For Bitwarden Cloud, configure split endpoints instead of `VWSO_VAULTWARDEN_URL`:

```bash
VWSO_IDENTITY_URL="https://identity.bitwarden.com" \
VWSO_API_URL="https://api.bitwarden.com" \
VWSO_CLIENT_ID="user.<uuid>" \
VWSO_CLIENT_SECRET="..." \
VWSO_MASTER_PASSWORD="..." \
VWSO_CACHE_TTL_SECONDS=60 \
cargo run -p vwso-eso-webhook -- --listen 127.0.0.1:8080
```

Compatibility details are in [`docs/compatibility.md`](docs/compatibility.md).
Live smoke-test instructions are in [`docs/live-testing.md`](docs/live-testing.md).

Install the webhook with Helm:

```bash
kubectl create namespace vwso-system
kubectl -n vwso-system create secret generic vwso-credentials \
  --from-literal=client-id='user.<uuid>' \
  --from-literal=client-secret='...' \
  --from-literal=master-password='...'

helm upgrade --install vwso ./deploy/helm/vaultwarden-secrets-operator \
  --namespace vwso-system \
  --set-string config.vaultwardenUrl='https://vaultwarden.example.com' \
  --set-string credentials.existingSecret.name=vwso-credentials
```

Then create an ESO `SecretStore` and `ExternalSecret` using the examples under
[`deploy/eso`](deploy/eso).

## License

Apache-2.0. Keep this repo free of copied code from reference projects unless a
license review explicitly approves it.
