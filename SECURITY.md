# Security Policy

This project is not production-ready yet.

## Reporting

While the repository is private, report issues directly to the maintainer. Before
public release, replace this section with a public vulnerability disclosure
process and security contact.

## Security Invariants

- Never disable TLS verification by default.
- Never log decrypted secret values, master passwords, access tokens, refresh
  tokens, API keys, or derived keys.
- Never accept namespace targets from Vaultwarden item metadata as the primary
  authorization model.
- Never require cluster-admin for the default deployment path.
- Never delete Kubernetes Secrets unless Kubernetes ownership policy or an
  explicit user setting says deletion is allowed.

## Threat Model

The initial threat model lives in [docs/threat-model.md](docs/threat-model.md).

