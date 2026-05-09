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

Do not run this test against a personal daily-use account. Use a dedicated
Vaultwarden or Bitwarden user with only the fixture items needed by this
project.
