# Compatibility

This project targets the Bitwarden Password Manager vault protocol implemented
by Vaultwarden and Bitwarden. It does not target Bitwarden Secrets Manager
(`bws`) APIs.

## Endpoint Modes

Two endpoint layouts are supported.

Single-origin servers use one URL and derive the API roots from it:

```bash
BWESO_SINGLE_ORIGIN_URL="https://vaultwarden.example.com"
```

Requests are built as:

- Identity: `https://vaultwarden.example.com/identity/...`
- API: `https://vaultwarden.example.com/api/...`

This is the normal Vaultwarden layout and also matches self-hosted Bitwarden
when exposed through one origin.

Split endpoint servers use explicit identity and API URLs:

```bash
BWESO_IDENTITY_URL="https://identity.bitwarden.com"
BWESO_API_URL="https://api.bitwarden.com"
```

For Bitwarden EU, use:

```bash
BWESO_IDENTITY_URL="https://identity.bitwarden.eu"
BWESO_API_URL="https://api.bitwarden.eu"
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

- `BWESO_TEST_SINGLE_ORIGIN_URL` for single-origin servers.
- `BWESO_TEST_IDENTITY_URL` plus `BWESO_TEST_API_URL` for Bitwarden Cloud or
  explicit split deployments.

Live verification status:

- Vaultwarden single-origin: verified against the exact `v0.1.2` OCI release
  chart and image on a real k3s cluster on 2026-05-11, including selector
  policy, single-field and whole-item sync, target Secret recreation, webhook
  restart, negative cases, and redacted metrics.
- Bitwarden Cloud US split endpoints: verified against a dedicated live account
  and the exact `v0.1.1` release chart and image on a real k3s cluster on
  2026-05-11, including selector policy, single-field and whole-item sync,
  target Secret recreation, webhook restart, negative cases, and redacted
  metrics. The `v0.1.2` release changes packaging/version metadata and keeps the
  same split-endpoint provider protocol implementation covered by fake-server
  tests.

The latest Vaultwarden live verification environment used:

- k3s server `v1.34.5+k3s1`.
- External Secrets Operator `v2.4.1`.
- Helm `v4.1.4`.
- Vaultwarden `1.35.4`.

## Current Scope

Implemented:

- User API-key login with master-password unlock data.
- PBKDF2-SHA256 and Argon2id account KDFs.
- Authenticated Bitwarden encrypted strings.
- Login, secure-note notes, custom fields, TOTP fields, and SSH key fields.
- Individual vault item sync against single-origin and split endpoint layouts.
- Single-field ESO sync through `remoteRef` and whole-item ESO sync through
  `dataFrom.extract`.
- `id:<item-id>` and `name:<item-name>` selectors. Bare selectors currently try
  ID first and then decrypted item name for pre-release compatibility.
- Provider-side selector policy based on exact raw keys and raw key prefixes.
  This policy gates item keys, not individual item properties.

Not yet implemented:

- Bitwarden Secrets Manager (`bws`) machine-account/project secret APIs.
- Shared organization item decryption that requires organization key handling.
  Selected shared items fail explicitly instead of silently returning partial
  results.
- Attachment metadata lookup, download, decryption, and mapping. Properties
  beginning with `attachment.` or `attachments.` fail explicitly.
- Interactive two-factor or new-device challenge handling for API-key login.
