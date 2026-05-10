# Live Testing

The normal CI suite uses deterministic fixtures and local fake servers for both
single-origin and split endpoint layouts. A live smoke test is available for a
real Bitwarden-compatible Password Manager instance, but it is skipped unless all
required environment variables are set.

For Vaultwarden or a self-hosted single-origin server:

```bash
VWSO_TEST_VAULTWARDEN_URL="https://vaultwarden.example.com" \
VWSO_TEST_CLIENT_ID="user.<uuid>" \
VWSO_TEST_CLIENT_SECRET="..." \
VWSO_TEST_MASTER_PASSWORD="..." \
VWSO_TEST_ITEM_KEY="app/database" \
VWSO_TEST_PROPERTY="DATABASE_URL" \
cargo test -p vwso-vaultwarden --test live_vaultwarden -- --nocapture
```

For Bitwarden Cloud US:

```bash
VWSO_TEST_IDENTITY_URL="https://identity.bitwarden.com" \
VWSO_TEST_API_URL="https://api.bitwarden.com" \
VWSO_TEST_CLIENT_ID="user.<uuid>" \
VWSO_TEST_CLIENT_SECRET="..." \
VWSO_TEST_MASTER_PASSWORD="..." \
VWSO_TEST_ITEM_KEY="app/database" \
VWSO_TEST_PROPERTY="DATABASE_URL" \
cargo test -p vwso-vaultwarden --test live_vaultwarden -- --nocapture
```

For Bitwarden Cloud EU, use `https://identity.bitwarden.eu` and
`https://api.bitwarden.eu`.

`VWSO_TEST_PROPERTY` is optional. When omitted, the test resolves the whole
item and asserts that at least one secret field is returned.

For a secret-safe smoke test against an existing dedicated test vault, omit the
item key and let the test choose the first decryptable item with extractable
secret fields:

```bash
VWSO_TEST_VAULTWARDEN_URL="https://vaultwarden.example.com" \
VWSO_TEST_CLIENT_ID="user.<uuid>" \
VWSO_TEST_CLIENT_SECRET="..." \
VWSO_TEST_MASTER_PASSWORD="..." \
VWSO_TEST_ALLOW_ANY_ITEM=true \
cargo test -p vwso-vaultwarden --test live_vaultwarden -- --nocapture
```

This mode does not print decrypted values or selected item names. If
`VWSO_TEST_PROPERTY` is also set, the test searches for the first decryptable
item containing that property.

Set `VWSO_TEST_SELECTOR_OUTPUT=/tmp/vwso-selector.json` to write the selected
item ID and property name for follow-up Kubernetes tests. The file does not
contain decrypted values.

Do not run this test against a personal daily-use account. Use a dedicated
Vaultwarden or Bitwarden user with only the fixture items needed by this
project.

## Live ESO Smoke Test

`scripts/live-eso-smoke.sh` deploys the Helm chart into a temporary namespace,
creates a namespace-local `SecretStore`, syncs an `ExternalSecret`, verifies
target Secret recreation, restarts the webhook Deployment, forces another sync,
and checks expected error cases for missing items/properties. It does not print
decrypted values.

Required:

- `kubectl`, `helm`, `jq`, and `cargo`.
- External Secrets Operator already installed in the target cluster.
- A pushed image tag for the webhook.
- Live test credentials through the `VWSO_TEST_*` variables above, or the
  equivalent runtime `VWSO_*` variables.

Example:

```bash
export VWSO_E2E_KUBE_CONTEXT=<your-cluster-context>
export VWSO_E2E_IMAGE_TAG="$(git rev-parse --short=12 HEAD)"
export VWSO_TEST_VAULTWARDEN_URL="https://vaultwarden.example.com"
export VWSO_TEST_CLIENT_ID="user.<uuid>"
export VWSO_TEST_CLIENT_SECRET="..."
export VWSO_TEST_MASTER_PASSWORD="..."
export VWSO_TEST_ALLOW_ANY_ITEM=true

scripts/live-eso-smoke.sh
```

For private GHCR images, set `VWSO_E2E_GHCR_TOKEN` and optionally
`VWSO_E2E_GHCR_USER`; the script creates a temporary namespace-local
image-pull Secret without printing the token.
