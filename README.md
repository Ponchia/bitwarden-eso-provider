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
- Explicit `id:<item-id>` and `name:<item-name>` selectors. Bare selectors keep
  the pre-release ID-then-name behavior, but `id:` is recommended for
  production.
- Optional provider-side selector policy that allows exact keys or key prefixes
  and denies every other `remoteRef.key`.
- In-memory sync cache with explicit TTL and single-flight refresh behavior.
- Bearer-token authentication on `/v1/resolve` by default; unauthenticated mode
  is an explicit local-test setting.
- Dedicated `/livez`, `/readyz`, and `/metrics` endpoints with redacted
  Prometheus-format runtime, HTTP, and resolution metrics.
- Helm chart, ESO manifests, live smoke test script, architecture notes, threat
  model, and release checklist.

Current intentional limits:

- Bitwarden Secrets Manager (`bws`) APIs are not supported.
- Shared organization vault items fail explicitly until organization-key
  decryption is implemented and live-tested.
- Attachment extraction fails explicitly. Store certificates, kubeconfigs, SSH
  keys, and multiline config in notes or custom fields for `v0.1.0`.

## Why This Exists

Bitwarden has two different product surfaces:

- Password Manager vault items, which are also the surface implemented by
  Vaultwarden through the Bitwarden-compatible client API.
- Secrets Manager (`bws`) secrets, projects, machine accounts, and access
  tokens.

If your application secrets already live in Bitwarden Secrets Manager, use the
official Bitwarden or External Secrets Operator integration. This project exists
for the other case: teams that already keep operational secrets in Bitwarden
Password Manager or Vaultwarden and want Kubernetes to consume them through
standard ESO-managed Kubernetes Secrets.

| Option | Secret Source | Kubernetes Model | Vaultwarden Fit | Restart Story | Best Fit |
| --- | --- | --- | --- | --- | --- |
| **Bitwarden ESO Provider** | Bitwarden Password Manager or Vaultwarden vault items | ESO generic webhook; ESO owns `SecretStore`, `ExternalSecret`, refresh, status, templating, and target `Secret` lifecycle | Yes. Single-origin Vaultwarden and Bitwarden Cloud split endpoints are live-tested | Provider is stateless; use ESO force-sync plus app reload, Stakater Reloader, checksum annotations, or GitOps rollouts | Reuse existing Password Manager or Vaultwarden items without adopting a second secrets backend |
| [Bitwarden Secrets Manager Kubernetes Operator](https://bitwarden.com/help/secrets-manager-kubernetes-operator/) | Bitwarden Secrets Manager secrets and projects | First-party `BitwardenSecret` CRD and controller sync to Kubernetes `Secret` | No. It targets Secrets Manager, not the Password Manager client API implemented by Vaultwarden | Operator syncs on its refresh interval; workload restart remains an explicit deployment concern | Best official Bitwarden path when secrets can live in Secrets Manager |
| [ESO Bitwarden Secrets Manager provider](https://external-secrets.io/latest/provider/bitwarden-secrets-manager/) | Bitwarden Secrets Manager | Native ESO provider plus the Bitwarden SDK server | No. It targets Secrets Manager, not Password Manager or Vaultwarden items | ESO owns refresh and target `Secret` behavior; the SDK server and TLS/certificate setup are additional operational pieces | Best ESO-native path for `bws` users |
| [1Password Kubernetes Operator](https://developer.1password.com/docs/k8s/operator/) | 1Password items | First-party operator using 1Password Connect or service-account authentication | No | Supports automatic deployment restarts when linked 1Password items change | Mature option when the organization already uses 1Password |
| ESO built-in providers for HashiCorp Vault, cloud secret managers, Infisical, and similar systems | Purpose-built infrastructure secret backends | Native ESO providers | No | ESO plus backend-specific audit, identity, rotation, or dynamic-secret behavior | Best for teams adopting a dedicated infrastructure secrets platform |
| [Secrets Store CSI Driver](https://secrets-store-csi-driver.sigs.k8s.io/getting-started/usage) | External secret stores with CSI providers | CSI volume mounts with optional Kubernetes `Secret` sync | Not for this Password Manager/Vaultwarden flow | Updates mounts and synced Secrets; it does not restart application pods | Best when applications should read secrets from mounted files |
| `bw` CLI scripts or ad-hoc webhooks | Bitwarden Password Manager, sometimes Vaultwarden | Script, sidecar, cron, or custom webhook | Often possible | Depends entirely on the script | Fine for personal automation; weaker for a public, tested, observable Kubernetes integration |

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
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
```

Run locally against Vaultwarden or a self-hosted single-origin Bitwarden server:

```bash
BWESO_SINGLE_ORIGIN_URL="https://vaultwarden.example.com" \
BWESO_CLIENT_ID="user.<uuid>" \
BWESO_CLIENT_SECRET="..." \
BWESO_MASTER_PASSWORD="..." \
BWESO_WEBHOOK_AUTH_TOKEN="..." \
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
BWESO_WEBHOOK_AUTH_TOKEN="..." \
BWESO_CACHE_TTL_SECONDS=60 \
cargo run -p bitwarden-eso-provider -- --listen 127.0.0.1:8080
```

## Helm Install

```bash
kubectl create namespace bweso-system
kubectl -n bweso-system create secret generic bweso-credentials \
  --from-literal=client-id='user.<uuid>' \
  --from-literal=client-secret='...' \
  --from-literal=master-password='...' \
  --from-literal=webhook-token='generate-a-long-random-token'

helm upgrade --install bweso ./deploy/helm/bitwarden-eso-provider \
  --namespace bweso-system \
  --set-string config.singleOriginUrl='https://vaultwarden.example.com' \
  --set-string credentials.existingSecret.name=bweso-credentials \
  --set-string selectorPolicy.allowedKeys[0]='id:00000000-0000-0000-0000-000000000000'
```

For Bitwarden Cloud, use `config.identityUrl=https://identity.bitwarden.com` and
`config.apiUrl=https://api.bitwarden.com` instead of `config.singleOriginUrl`.

Then create an ESO `SecretStore` and `ExternalSecret` using the examples under
[`deploy/eso`](deploy/eso). Prefer namespace-local `SecretStore` resources and
`id:<item-id>` selectors. Namespace-local `SecretStore` resources need a
same-namespace token-only Secret labeled `external-secrets.io/type=webhook`;
do not copy the Bitwarden/Vaultwarden client secret and master password into
workload namespaces:

```bash
kubectl -n app create secret generic bweso-webhook-auth \
  --from-literal=webhook-token='same-webhook-token-as-above'
kubectl -n app label secret bweso-webhook-auth external-secrets.io/type=webhook
```

For migrated Kubernetes Secret keys, use `field.<key>` properties so custom
fields named `username` or `password` do not collide with Bitwarden login
fields. Use a broad `ClusterSecretStore` only when every namespace allowed to
reference it is in the same trust boundary.

For target Secrets that should survive GitOps mistakes and be recreated after
manual deletion, use ESO's `creationPolicy: Orphan`, `deletionPolicy: Retain`,
and template `mergePolicy: Merge`; the examples use that policy. In particular,
`creationPolicy: Merge` does not recreate a missing target Secret.

Compatibility details are in [`docs/compatibility.md`](docs/compatibility.md).
Operational metrics and probe details are in
[`docs/operations/observability.md`](docs/operations/observability.md).
Existing Kubernetes Secret migration guidance is in
[`docs/operations/migration-runbook.md`](docs/operations/migration-runbook.md).
Live smoke-test instructions are in [`docs/live-testing.md`](docs/live-testing.md).
Public repository and contribution expectations are in
[`CONTRIBUTING.md`](CONTRIBUTING.md), [`SECURITY.md`](SECURITY.md), and
[`docs/repository-governance.md`](docs/repository-governance.md).

## License

Apache-2.0. Keep this repo free of copied code from reference projects unless a
license review explicitly approves it.
