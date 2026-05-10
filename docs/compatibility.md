# Compatibility

This project targets the Bitwarden Password Manager vault protocol implemented
by Vaultwarden and Bitwarden. It does not target Bitwarden Secrets Manager
(`bws`) APIs.

## Endpoint Modes

Two endpoint layouts are supported.

Single-origin servers use one URL and derive the API roots from it:

```bash
VWSO_VAULTWARDEN_URL="https://vaultwarden.example.com"
```

Requests are built as:

- Identity: `https://vaultwarden.example.com/identity/...`
- API: `https://vaultwarden.example.com/api/...`

This is the normal Vaultwarden layout and also matches self-hosted Bitwarden
when exposed through one origin.

Split endpoint servers use explicit identity and API URLs:

```bash
VWSO_IDENTITY_URL="https://identity.bitwarden.com"
VWSO_API_URL="https://api.bitwarden.com"
```

For Bitwarden EU, use:

```bash
VWSO_IDENTITY_URL="https://identity.bitwarden.eu"
VWSO_API_URL="https://api.bitwarden.eu"
```

The provider appends only the endpoint-specific paths in split mode:

- Prelogin: `{identity_url}/accounts/prelogin/password`
- Token: `{identity_url}/connect/token`
- Sync: `{api_url}/sync?excludeDomains=true`

## Verified Contract

The automated suite has fake-server coverage for both endpoint layouts:

- Vaultwarden/self-hosted single-origin routes under `/identity` and `/api`.
- Bitwarden-style split identity and API servers with root-relative
  `/connect/token`, `/accounts/prelogin/password`, and `/sync` routes.
- Token form fields use Bitwarden client casing for device metadata:
  `deviceIdentifier`, `deviceName`, and `deviceType`.

The live smoke test can be aimed at either layout:

- `VWSO_TEST_VAULTWARDEN_URL` for single-origin servers.
- `VWSO_TEST_IDENTITY_URL` plus `VWSO_TEST_API_URL` for Bitwarden Cloud or
  explicit split deployments.

Live verification status:

- Vaultwarden single-origin: verified against a real k3s cluster with External
  Secrets Operator on 2026-05-09.
- Bitwarden Cloud split endpoints: fake-server covered, still pending a live
  validation account.

## Current Scope

Implemented:

- User API-key login with master-password unlock data.
- PBKDF2-SHA256 and Argon2id account KDFs.
- Authenticated Bitwarden encrypted strings.
- Login, secure-note notes, custom fields, TOTP fields, and SSH key fields.
- Personal vault item sync against single-origin and split endpoint layouts.

Not yet implemented:

- Bitwarden Secrets Manager (`bws`) machine-account/project secret APIs.
- Shared organization item decryption that requires asymmetric organization key
  decapsulation.
- Interactive two-factor or new-device challenge handling for API-key login.
