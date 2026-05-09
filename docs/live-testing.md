# Live Testing

The normal CI suite uses deterministic fixtures and a local fake Vaultwarden
server. A live smoke test is available for a real Vaultwarden-compatible
instance, but it is skipped unless all required environment variables are set.

```bash
VWSO_TEST_VAULTWARDEN_URL="https://vaultwarden.example.com" \
VWSO_TEST_CLIENT_ID="user.<uuid>" \
VWSO_TEST_CLIENT_SECRET="..." \
VWSO_TEST_MASTER_PASSWORD="..." \
VWSO_TEST_ITEM_KEY="app/database" \
VWSO_TEST_PROPERTY="DATABASE_URL" \
cargo test -p vwso-vaultwarden --test live_vaultwarden -- --nocapture
```

`VWSO_TEST_PROPERTY` is optional. When omitted, the test resolves the whole
item and asserts that at least one secret field is returned.

Do not run this test against a personal daily-use account. Use a dedicated
Vaultwarden user with only the fixture items needed by this project.
