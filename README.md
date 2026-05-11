# Bitwarden ESO Provider

[![CI](https://github.com/Ponchia/bitwarden-eso-provider/actions/workflows/ci.yml/badge.svg)](https://github.com/Ponchia/bitwarden-eso-provider/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Ponchia/bitwarden-eso-provider?include_prereleases&sort=semver)](https://github.com/Ponchia/bitwarden-eso-provider/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Unofficial [External Secrets Operator](https://external-secrets.io/) webhook
provider for syncing **Bitwarden Password Manager** and **Vaultwarden** vault
items into Kubernetes `Secret` resources.

This project is for the Password Manager vault-item API surface, not
[Bitwarden Secrets Manager](https://bitwarden.com/products/secrets-manager/).
It exists for teams and homelabs that already keep operational secrets in
Bitwarden Password Manager or Vaultwarden and want to consume them through
standard ESO-managed Kubernetes Secrets.

```text
Bitwarden Cloud or Vaultwarden
        |
        | user API key + master-password unlock + local item decryption
        v
bitwarden-eso-provider
        |
        | ESO webhook provider
        v
External Secrets Operator
        |
        v
Kubernetes Secret
```

This repository is not affiliated with Bitwarden, Vaultwarden, 1Password, or
the External Secrets Operator project.

## Status

Pre-release. The provider is functional and live-tested, but chart values,
image tags, and crate APIs may still change before the first public tag.

Verified so far:

- Vaultwarden and self-hosted single-origin endpoint layout.
- Bitwarden Cloud US split identity/API endpoint layout.
- PBKDF2-SHA256 and Argon2id vault unlock.
- ESO sync through `remoteRef` and `dataFrom.extract`.
- Target Secret recreation, webhook restart, expected not-found failures,
  selector-policy denial, health probes, and redacted metrics.
- Prometheus Operator `ServiceMonitor` / `PrometheusRule` compatibility, both
  through Helm rendering and server-side Kubernetes validation.

Use a dedicated test account first. Do not aim this at a primary day-to-day
vault until you have reviewed the [threat model](docs/threat-model.md) and
understand the namespace isolation model.

## When To Use It

Use this project when:

- Your source of truth is Bitwarden Password Manager or Vaultwarden vault
  items.
- You want ESO to own refresh intervals, target Secret policies, templating,
  status, and GitOps-friendly manifests.
- You can dedicate a Bitwarden/Vaultwarden user or API key to each Kubernetes
  trust boundary.
- You want a small webhook service with no Kubernetes API permissions.

Use something else when:

- Your secrets already live in Bitwarden Secrets Manager. Use Bitwarden's
  official operator or ESO's Bitwarden Secrets Manager provider.
- You need dynamic infrastructure secrets, leases, or database credentials.
  Use Vault, a cloud secret manager, Infisical, or another purpose-built
  backend.
- You need shared organization item decryption or attachment extraction today.
  Those are explicit non-features for the first release.
- You cannot accept storing a Bitwarden/Vaultwarden user API key and master
  password in the provider runtime namespace.

## Features

- Rust HTTP webhook implementing ESO's generic webhook contract.
- Bitwarden-compatible user API-key login, vault sync, and local decryption.
- PBKDF2-SHA256 and Argon2id account KDF support.
- Bitwarden Cloud split endpoints and Vaultwarden single-origin endpoints.
- Single-field sync through ESO `remoteRef`.
- Whole-item sync through ESO `dataFrom.extract`.
- Explicit `id:<item-id>` and `name:<item-name>` selectors.
- Optional provider-side selector policy with exact key and prefix allowlists.
- In-memory sync cache with TTL and single-flight refresh behavior.
- Bearer-token authentication on `/v1/resolve` by default.
- `/livez`, `/readyz`, and `/metrics` endpoints.
- Helm chart, ESO examples, NetworkPolicy examples, Reloader examples,
  Grafana dashboard, PrometheusRule example, live smoke-test script, threat
  model, and release checklist.

## Current Limits

- Bitwarden Secrets Manager (`bws`) APIs are not supported.
- Shared organization items fail with `unsupported_shared_item` until
  organization-key decryption is implemented and live-tested.
- Attachment properties fail with `unsupported_attachment`. For `v0.1.0`, store
  certificates, kubeconfigs, SSH keys, and multiline config in secure notes or
  custom fields.
- Interactive two-factor and new-device challenge flows are not supported for
  API-key login.
- The provider does not restart application workloads. Use ESO refreshes,
  Stakater Reloader, checksum annotations, or your GitOps rollout mechanism.

## Quick Start

Prerequisites:

- Kubernetes cluster.
- [External Secrets Operator](https://external-secrets.io/latest/) installed.
- Helm 3.
- A dedicated Bitwarden or Vaultwarden user API key.
- The user's master password.
- A provider image tag or digest. Released chart archives are attached to GitHub
  Releases and default to the matching provider image version.

Create the provider namespace and runtime credentials:

```bash
kubectl create namespace bweso-system

kubectl -n bweso-system create secret generic bweso-credentials \
  --from-literal=client-id='user.<uuid>' \
  --from-literal=client-secret='...' \
  --from-literal=master-password='...' \
  --from-literal=webhook-token='generate-a-long-random-token'
```

Set the release chart reference:

```bash
CHART_VERSION=0.1.0
CHART_REF="https://github.com/ponchia/bitwarden-eso-provider/releases/download/v${CHART_VERSION}/bitwarden-eso-provider-${CHART_VERSION}.tgz"
```

Install the provider for Vaultwarden or another single-origin Bitwarden server:

```bash
helm upgrade --install bweso "${CHART_REF}" \
  --namespace bweso-system \
  --set-string image.repository='ghcr.io/ponchia/bitwarden-eso-provider' \
  --set-string image.tag="${CHART_VERSION}" \
  --set-string config.singleOriginUrl='https://vaultwarden.example.com' \
  --set-string credentials.existingSecret.name='bweso-credentials' \
  --set-string selectorPolicy.allowedKeys[0]='id:00000000-0000-0000-0000-000000000000'
```

For Bitwarden Cloud US, use split endpoints instead:

```bash
helm upgrade --install bweso "${CHART_REF}" \
  --namespace bweso-system \
  --set-string image.repository='ghcr.io/ponchia/bitwarden-eso-provider' \
  --set-string image.tag="${CHART_VERSION}" \
  --set-string config.identityUrl='https://identity.bitwarden.com' \
  --set-string config.apiUrl='https://api.bitwarden.com' \
  --set-string credentials.existingSecret.name='bweso-credentials' \
  --set-string selectorPolicy.allowedKeys[0]='id:00000000-0000-0000-0000-000000000000'
```

For Bitwarden Cloud EU, use `https://identity.bitwarden.eu` and
`https://api.bitwarden.eu`.

For unreleased `main` builds, clone the repository and replace `"${CHART_REF}"`
with `./deploy/helm/bitwarden-eso-provider`.

Create a token-only ESO auth Secret in each workload namespace that will use a
namespace-local `SecretStore`:

```bash
kubectl create namespace app

kubectl -n app create secret generic bweso-webhook-auth \
  --from-literal=webhook-token='same-webhook-token-as-above'

kubectl -n app label secret bweso-webhook-auth \
  external-secrets.io/type=webhook
```

Create a namespace-local `SecretStore`:

```yaml
apiVersion: external-secrets.io/v1
kind: SecretStore
metadata:
  name: bitwarden
  namespace: app
spec:
  provider:
    webhook:
      url: "http://bweso-bitwarden-eso-provider.bweso-system.svc.cluster.local:8080/v1/resolve"
      method: POST
      headers:
        Content-Type: application/json
        Authorization: 'Bearer {{ index .auth "webhook-token" }}'
      secrets:
        - name: auth
          secretRef:
            name: bweso-webhook-auth
            key: webhook-token
      body: |
        {
          "remoteRef": {
            "key": {{ .remoteRef.key | toJson }},
            "property": {{ .remoteRef.property | toJson }}
          }
        }
      result:
        jsonPath: "$.data.value"
      timeout: 10s
```

Create an `ExternalSecret`:

```yaml
apiVersion: external-secrets.io/v1
kind: ExternalSecret
metadata:
  name: app-database
  namespace: app
spec:
  refreshPolicy: Periodic
  refreshInterval: 1h
  secretStoreRef:
    name: bitwarden
    kind: SecretStore
  target:
    name: app-database
    creationPolicy: Orphan
    deletionPolicy: Retain
    template:
      mergePolicy: Merge
  data:
    - secretKey: DATABASE_URL
      remoteRef:
        key: id:00000000-0000-0000-0000-000000000000
        property: field.DATABASE_URL
```

More examples are in [deploy/eso](deploy/eso), including whole-item extraction,
Docker config JSON, basic auth, SSH auth, multiline files, Reloader, warned
`ClusterSecretStore`, and NetworkPolicy starting points.

## Selectors And Properties

Use `id:<item-id>` selectors in production. Item IDs are stable across renames.
`name:<item-name>` is supported for convenience, but duplicate item names are
rejected as ambiguous. Bare selectors currently try item ID first and then item
name for pre-release compatibility.

Common properties:

| Property | Meaning |
| --- | --- |
| `username` or `login.username` | Login username field. |
| `password` or `login.password` | Login password field. |
| `totp` or `login.totp` | Login TOTP field. |
| `notes` | Item notes or secure-note content. |
| `field.<name>` | Custom field with the exact name. |
| `custom.<name>` | Custom field alias. |
| `<name>` | Custom field fallback when no conventional property matches. |
| `sshKey.privateKey` | SSH private key field. |
| `sshKey.publicKey` | SSH public key field. |
| `sshKey.keyFingerprint` | SSH key fingerprint field. |

Prefer `field.<key>` for migrated Kubernetes Secret keys. Plain `username` and
`password` select Bitwarden login fields; `field.username` and
`field.password` select custom fields with those names.

## Security Model

The provider is intentionally narrow:

- Kubernetes manifests decide target namespaces and Secret names. Vault item
  metadata never decides where data is written.
- The provider needs no Kubernetes API RBAC.
- The provider's Bitwarden/Vaultwarden client ID, client secret, and master
  password stay in the provider namespace.
- Workload namespaces need only the webhook bearer token used by ESO.
- Logs, metrics, and public error responses redact secret values, item IDs,
  item names, requested properties, API tokens, master passwords, and derived
  keys.
- TLS verification is required for non-local Bitwarden/Vaultwarden endpoints.
- The default Service is cluster-internal HTTP. Keep it private, require the
  bearer token, use NetworkPolicy, and put it behind TLS or mTLS when the pod
  network is not a trusted boundary.

Selector policy is item-key scoped. If a namespace can request an allowed
`remoteRef.key`, it can request any property on that item and can use whole-item
extraction unless ESO manifests, RBAC, and review prevent it. For strict
isolation, use dedicated provider credentials per namespace or trust boundary
plus exact `id:` allowlists.

Kubernetes Secrets are still Kubernetes Secrets. Enable encryption at rest,
restrict RBAC, and avoid granting broad read access to generated Secrets.

Read the full [threat model](docs/threat-model.md) before production use.

## Observability

The provider exposes:

- `/livez`: process liveness.
- `/readyz`: readiness, including graceful shutdown behavior.
- `/metrics`: Prometheus text exposition.

Metrics are low-cardinality and redacted. They cover HTTP requests, resolve
outcomes, error classes, latency, cache hits, cache refreshes, and last
successful cache refresh age.

The Helm chart can render a `ServiceMonitor` when Prometheus Operator CRDs are
installed:

```bash
helm upgrade --install bweso "${CHART_REF}" \
  --namespace bweso-system \
  --reuse-values \
  --set metrics.serviceMonitor.enabled=true
```

Grafana and alerting examples are in [examples](examples). Operational details
are in [docs/operations/observability.md](docs/operations/observability.md).

## Comparison

Bitwarden ESO Provider is deliberately narrow: it syncs Bitwarden Password
Manager and Vaultwarden vault items through ESO. That makes it useful when the
vault item is already the source of truth, but it is not a replacement for a
dedicated infrastructure secret manager.

Use **Bitwarden ESO Provider** when you need:

- Bitwarden Password Manager or Vaultwarden vault-item support.
- ESO-managed `SecretStore`, `ExternalSecret`, refresh intervals, target
  policies, and templating.
- A webhook service with no Kubernetes API permissions.

Use **Bitwarden Secrets Manager integrations** when your source of truth is
Bitwarden Secrets Manager (`bws`), not Password Manager vault items:

- [Bitwarden Secrets Manager Kubernetes Operator](https://bitwarden.com/help/secrets-manager-kubernetes-operator/)
  is the first-party operator with its own `BitwardenSecret` CRD.
- [ESO Bitwarden Secrets Manager provider](https://external-secrets.io/latest/provider/bitwarden-secrets-manager/)
  is the ESO-native path for `bws`, with the Bitwarden SDK server and its
  certificate setup.

Use **[1Password Kubernetes Operator](https://developer.1password.com/docs/k8s/operator/)**
when your organization already stores secrets in 1Password and wants the
first-party Kubernetes integration, including documented automatic redeploy
annotations.

Use **ESO providers for Vault, cloud secret managers, Infisical, and similar
systems** when you want a dedicated infrastructure secret-management platform,
dynamic secrets, native identity integration, leases, audit workflows, or
provider-specific rotation behavior.

Use **[Secrets Store CSI Driver](https://secrets-store-csi-driver.sigs.k8s.io/getting-started/usage)**
when workloads should consume mounted files from external stores instead of
ESO-managed Kubernetes Secret manifests.

Use **`bw` CLI scripts or custom cron jobs** only for small personal
automations. They can work, but they are weaker as a public, tested,
observable Kubernetes integration.

## Repository Layout

```text
crates/bweso-core
  Shared request/response and secret document types
crates/bweso-bitwarden
  Bitwarden-compatible API, crypto, and resolver
crates/bitwarden-eso-provider
  HTTP adapter for ESO's webhook provider
deploy/eso
  SecretStore, ExternalSecret, Reloader, and NetworkPolicy examples
deploy/helm
  Helm chart
docs
  Architecture, install, compatibility, operations, testing, security
examples
  Grafana dashboard and PrometheusRule examples
references
  Reference repository manifest and notes
```

## Development

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
```

Coverage is tracked as a regression signal, not as a release goal by itself:

```bash
cargo install cargo-llvm-cov --locked
cargo llvm-cov --locked --workspace --all-targets --fail-under-lines 80 --summary-only
cargo llvm-cov --locked --workspace --all-targets \
  --lcov --output-path coverage/lcov.info
```

`coverage/lcov.info` is ignored by Git and imported by SonarQube through
`sonar-project.properties` when present.

Run the provider locally against Vaultwarden or a single-origin self-hosted
Bitwarden server:

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

Live test instructions are in [docs/live-testing.md](docs/live-testing.md).

## Documentation

- [Install guide](docs/install/eso-webhook.md)
- [Compatibility](docs/compatibility.md)
- [Architecture](docs/architecture.md)
- [Threat model](docs/threat-model.md)
- [Observability](docs/operations/observability.md)
- [Restarts and rollouts](docs/operations/restarts.md)
- [Migration runbook](docs/operations/migration-runbook.md)
- [Release checklist](docs/public-release-checklist.md)
- [Roadmap](docs/roadmap.md)

## Contributing And Security

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), keep
secrets out of issues and pull requests, and include the relevant validation
commands in PRs.

Report vulnerabilities privately through the process in
[SECURITY.md](SECURITY.md). Do not open public issues for credential leaks,
secret-value exposure, auth bypasses, or selector-redaction failures.

## License

Apache-2.0. Keep this repository free of copied implementation code from
reference projects unless a license review explicitly approves it.
