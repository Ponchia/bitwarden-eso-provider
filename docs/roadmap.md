# Roadmap

This roadmap separates what shipped in the `v0.1.x` release line from
follow-up work.

## v0.1.x

The first public release includes:

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
- Multi-arch release workflow with GHCR image publishing, SBOM/provenance,
  release image scanning, GHCR OCI Helm chart publishing, and Helm chart
  attachment to GitHub Releases.
- Live smoke verification against Vaultwarden and Bitwarden Cloud using the
  exact release chart and image.

## v0.2 (Rename + Vaultwarden-First Repositioning)

`v0.2` is the rename and repositioning boundary. `v0.1.x` had no real users,
so this is a hard rename: the old GHCR image package and OCI chart are
deleted, old tags are yanked, no compatibility shim is kept. Targeted scope:

- Rename the GitHub repository, binary crate, container image, and Helm chart
  from `bitwarden-eso-provider` to `vaultwarden-eso-provider`. The crate
  prefix `bweso-` (which refers to the Bitwarden-compatible API surface) can
  stay; only the binary crate and the chart move.
- Add **custom CA bundle support** (`BWESO_CA_BUNDLE_FILE` plus a Helm value)
  for Vaultwarden installs on a private CA. This is the most common gap for
  the self-hosted homelab target.
- Add per-source **rate limiting / concurrency cap** on `/v1/resolve` beyond
  the current bearer-token + body-size + single-flight-refresh mitigations.
- Collapse the six `BitwardenApiClient` constructors into a single
  `with_options(BitwardenApiClientOptions)` form.
- Replace the hand-rolled `constant_time_eq` with the `subtle` crate (or the
  `constant_time_eq` crate) so the project does not ship its own
  constant-time comparison.
- Replace the hand-rolled Prometheus text-format emitter with
  `metrics-exporter-prometheus`. Existing redaction tests cover regression.
- Add cargo-fuzz coverage on `EncryptedString::from_str` — the parser eating
  untrusted upstream bytes.
- Adopt the Bitwarden Password Manager SDK Rust crates: **no** (see
  [`decisions/0001-bitwarden-sdk-adoption.md`](decisions/0001-bitwarden-sdk-adoption.md)).

## After v0.2

Higher-effort follow-up work:

- Add disposable kind integration coverage that does not require private
  credentials.
- Add **organization / shared item decryption** after fixture coverage,
  key-handling review, and live verification against both Vaultwarden and
  Bitwarden Cloud. This is the gap that excludes most team-scale users today.
- Add attachment metadata lookup, download, decryption, and Kubernetes
  mapping only after the UX and security model are clear.
- Assess whether a native ESO provider would remove webhook operational
  friction without duplicating ESO's lifecycle semantics.
- Evaluate stale-cache-on-upstream-outage behavior. The first release keeps
  upstream failures explicit.
- Decide whether a GitHub Pages chart repository is worth the extra release
  surface for users who cannot consume OCI charts.
- Revisit a native Kubernetes controller only if ESO cannot cover important
  workflows cleanly.
- Add fuzzing on the Bitwarden encrypted-string parser and property tests on
  URL/path construction; the current CI gates verify hygiene, not protocol
  parser robustness.
- Consolidate the governance / readiness / release-checklist documents,
  which currently overlap.

## Not Planned For v0.1.x

- Bitwarden Secrets Manager (`bws`) support. Use Bitwarden's official Secrets
  Manager integrations for that product surface.
- Built-in application workload restarts. Use ESO refreshes, Reloader, checksum
  annotations, or GitOps rollout mechanisms.
- Property-level selector policy. The current selector policy gates item keys;
  isolate stricter trust boundaries with dedicated provider credentials and
  exact `id:` allowlists.
