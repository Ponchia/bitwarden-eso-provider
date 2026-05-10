# Live Testing

The normal CI suite uses deterministic fixtures and local fake servers for both
single-origin and split endpoint layouts. Live tests are skipped unless all
required environment variables are set.

## Direct Provider Test

For Vaultwarden or a self-hosted single-origin Bitwarden server:

```bash
BWESO_TEST_SINGLE_ORIGIN_URL="https://vaultwarden.example.com" \
BWESO_TEST_CLIENT_ID="user.<uuid>" \
BWESO_TEST_CLIENT_SECRET="..." \
BWESO_TEST_MASTER_PASSWORD="..." \
BWESO_TEST_ITEM_KEY="id:00000000-0000-0000-0000-000000000000" \
BWESO_TEST_PROPERTY="DATABASE_URL" \
cargo test -p bweso-bitwarden --test live_bitwarden -- --nocapture
```

For Bitwarden Cloud US:

```bash
BWESO_TEST_IDENTITY_URL="https://identity.bitwarden.com" \
BWESO_TEST_API_URL="https://api.bitwarden.com" \
BWESO_TEST_CLIENT_ID="user.<uuid>" \
BWESO_TEST_CLIENT_SECRET="..." \
BWESO_TEST_MASTER_PASSWORD="..." \
BWESO_TEST_ITEM_KEY="id:00000000-0000-0000-0000-000000000000" \
BWESO_TEST_PROPERTY="DATABASE_URL" \
cargo test -p bweso-bitwarden --test live_bitwarden -- --nocapture
```

For Bitwarden Cloud EU, use `https://identity.bitwarden.eu` and
`https://api.bitwarden.eu`.

`BWESO_TEST_PROPERTY` is optional. When omitted, the test resolves the whole
item and asserts that at least one secret field is returned.

For a secret-safe smoke test against an existing dedicated test vault, omit the
item key and let the test choose the first decryptable item with extractable
secret fields:

```bash
BWESO_TEST_SINGLE_ORIGIN_URL="https://vaultwarden.example.com" \
BWESO_TEST_CLIENT_ID="user.<uuid>" \
BWESO_TEST_CLIENT_SECRET="..." \
BWESO_TEST_MASTER_PASSWORD="..." \
BWESO_TEST_ALLOW_ANY_ITEM=true \
cargo test -p bweso-bitwarden --test live_bitwarden -- --nocapture
```

This mode does not print decrypted values or selected item names. If
`BWESO_TEST_PROPERTY` is also set, the test searches for the first decryptable
item containing that property.

Set `BWESO_TEST_SELECTOR_OUTPUT=/tmp/bweso-selector.json` to write the selected
item ID and property name for follow-up Kubernetes tests. The file does not
contain decrypted values.

Use a dedicated Vaultwarden or Bitwarden user with only the fixture items needed
by this project. Do not run live tests against a personal daily-use account.

## Live ESO Smoke Test

`scripts/live-eso-smoke.sh` deploys the Helm chart into a temporary namespace,
creates a namespace-local `SecretStore`, syncs an `ExternalSecret`, verifies
target Secret recreation, restarts the webhook Deployment, forces another sync,
checks expected error cases for missing items/properties and selector-policy
denial, and verifies `/livez`, `/readyz`, `/metrics`, successful/error/cache
metrics, and metric redaction. It does not print decrypted values.

Required:

- `kubectl`, `helm`, `jq`, `curl`, and `cargo`.
- External Secrets Operator already installed in the target cluster.
- A pushed image tag for the webhook.
- Live test credentials through the `BWESO_TEST_*` variables above, or the
  equivalent runtime `BWESO_*` variables.

The smoke test installs the chart with `selectorPolicy.allowedKeys` containing
only the selected item and one deliberate missing item. That proves allowed
selectors still sync and disallowed selectors fail with redacted `403`
responses.

The chart leaves NetworkPolicy disabled by default because egress to
Vaultwarden, Bitwarden Cloud, DNS, ESO, and Prometheus is cluster-specific. Set
`BWESO_E2E_NETWORK_POLICY_ENABLED=true` only when the chart values or cluster
defaults already allow the selected backend path. For private ingress or
split-horizon DNS, set `BWESO_E2E_HOST_ALIAS_IP` and optionally
`BWESO_E2E_HOST_ALIAS_HOSTNAME`; when omitted, the hostname is inferred from the
single-origin URL.

The normal path is to push to `main`, let GitHub Actions build and publish the
commit-tagged amd64 image, then run the smoke test with the 12-character commit
tag. This avoids slow local `linux/amd64` Docker builds on Apple Silicon or
other non-amd64 workstations.

If the GHCR package is private and was created manually before the workflow was
in place, grant the repository `Write` access under the package settings'
`Manage Actions access` section. Otherwise GitHub Actions can authenticate with
`GITHUB_TOKEN` but GHCR will still reject the push with `403 Forbidden`.

Example with a private GHCR image:

```bash
export BWESO_E2E_KUBE_CONTEXT="<your-cluster-context>"
export BWESO_E2E_IMAGE_TAG="$(git rev-parse --short=12 HEAD)"
export BWESO_E2E_GHCR_TOKEN="$(gh auth token)"
export BWESO_TEST_SINGLE_ORIGIN_URL="https://vaultwarden.example.com"
export BWESO_TEST_CLIENT_ID="user.<uuid>"
export BWESO_TEST_CLIENT_SECRET="..."
export BWESO_TEST_MASTER_PASSWORD="..."
export BWESO_TEST_ALLOW_ANY_ITEM=true
# Optional for private ingress/DNS paths:
# export BWESO_E2E_HOST_ALIAS_IP="10.43.186.117"
# Optional, inferred from BWESO_TEST_SINGLE_ORIGIN_URL when omitted:
# export BWESO_E2E_HOST_ALIAS_HOSTNAME="vaultwarden.example.com"

scripts/live-eso-smoke.sh
```

For public GHCR images, omit `BWESO_E2E_GHCR_TOKEN`. For private GHCR images,
set `BWESO_E2E_GHCR_TOKEN` and optionally `BWESO_E2E_GHCR_USER`; the script
creates a temporary namespace-local image-pull Secret without printing the
token.
