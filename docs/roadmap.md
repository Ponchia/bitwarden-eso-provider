# Roadmap

This roadmap separates what is already in the pre-release codebase from work
that should wait until after `v0.1.0`.

## v0.1.0 Release Candidate

The current release-candidate scope is:

- Bitwarden Password Manager and Vaultwarden vault-item sync through External
  Secrets Operator's generic webhook provider.
- Bitwarden-compatible user API-key login, master-password unlock, vault sync,
  and local item decryption.
- PBKDF2-SHA256 and Argon2id account KDF support.
- Vaultwarden/single-origin endpoints and Bitwarden Cloud split endpoints.
- Single-field `remoteRef` resolution and whole-item `dataFrom.extract`
  resolution.
- `id:<item-id>` and `name:<item-name>` selectors, with bare selector lookup
  kept only for pre-release compatibility.
- Provider-side selector allowlists for exact keys and key prefixes.
- Explicit hard failures for unsupported shared organization items and
  attachments.
- Redacted logs, public error bodies, and low-cardinality Prometheus metrics.
- Health probes, graceful shutdown readiness behavior, and cache metrics.
- Helm chart, ESO examples, NetworkPolicy examples, Reloader example,
  PrometheusRule example, Grafana dashboard, threat model, release checklist,
  and live smoke-test script.
- Multi-arch release workflow with GHCR image publishing, SBOM/provenance, and
  Helm chart attachment to GitHub Releases.

Before the repository is made public, the remaining work is operational:

- Keep docs aligned with the actual release state.
- Push a clean branch and confirm GitHub Actions is green.
- Run the release workflow or publish a tag to produce the exact image that
  will be smoke-tested.
- Run `scripts/live-eso-smoke.sh` against Vaultwarden and Bitwarden Cloud with
  selector policy enabled.
- Make the repository public only after the release docs, CI, and live smoke
  checks are coherent.
- Enable the documented `main` branch protection and security settings as soon
  as GitHub exposes them for the public repository.

## After v0.1.0

High-value follow-up work:

- Add disposable kind integration coverage that does not require private
  credentials.
- Assess whether a native ESO provider would remove webhook operational
  friction without duplicating ESO's lifecycle semantics.
- Add organization/shared item decryption after fixture coverage, key-handling
  review, and live verification.
- Add attachment metadata lookup, download, decryption, and Kubernetes mapping
  only after the UX and security model are clear.
- Evaluate stale-cache-on-upstream-outage behavior. The first release keeps
  upstream failures explicit.
- Decide whether OCI Helm chart publishing or a GitHub Pages chart repository is
  worth the extra release surface.
- Revisit a native Kubernetes controller only if ESO cannot cover important
  workflows cleanly.

## Not Planned For v0.1.0

- Bitwarden Secrets Manager (`bws`) support. Use Bitwarden's official Secrets
  Manager integrations for that product surface.
- Built-in application workload restarts. Use ESO refreshes, Reloader, checksum
  annotations, or GitOps rollout mechanisms.
- Property-level selector policy. The current selector policy gates item keys;
  isolate stricter trust boundaries with dedicated provider credentials and
  exact `id:` allowlists.
