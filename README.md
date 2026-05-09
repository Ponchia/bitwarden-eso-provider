# Vaultwarden Secrets Operator

Vaultwarden Secrets Operator is an experimental Rust implementation for using a
Vaultwarden-backed Bitwarden Password Manager vault as a Kubernetes secret
source.

The initial target is not a bespoke Kubernetes controller. The first production
path is an External Secrets Operator webhook provider:

```text
Vaultwarden -> vwso-eso-webhook -> External Secrets Operator -> Kubernetes Secret
```

This keeps Kubernetes ownership, refresh policy, deletion behavior, templating,
status conditions, and GitOps integration in External Secrets Operator while this
project focuses on one narrow responsibility: safely authenticating to
Vaultwarden-compatible APIs and resolving encrypted vault items.

## Status

Early implementation. Do not deploy yet.

The current repository contains:

- A Rust workspace with core model, Vaultwarden client boundary, and ESO webhook
  entrypoint crates.
- Bitwarden-compatible authenticated encrypted string decryption.
- Master-password user-key unlock for PBKDF2-SHA256 and Argon2id accounts.
- A tested Vaultwarden API-key login and sync client path backed by a local fake
  server, wired into the ESO webhook runtime through environment configuration.
- In-memory sync caching with explicit TTL and single-flight refresh behavior.
- Architecture, threat-model, and reference notes.
- Example External Secrets Operator manifests.
- CI scaffolding for formatting, clippy, and tests.

The webhook binary still needs deployment manifests, redacted metrics, and live
Vaultwarden/kind integration tests before it should be deployed.

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
crates/vwso-vaultwarden   Vaultwarden-compatible API and crypto boundary
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

## License

Apache-2.0. Keep this repo free of copied code from reference projects unless a
license review explicitly approves it.
