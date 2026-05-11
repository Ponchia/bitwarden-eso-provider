# Release Readiness Plan

This file records the release-readiness decisions for Bitwarden ESO Provider.
`v0.1.0` is public, so this document now describes the current baseline and the
rules for future releases.

## Product Shape

The first public release remains an External Secrets Operator webhook provider
for Bitwarden Password Manager and Vaultwarden vault items.

Do not build a native Kubernetes operator before `v0.1.0`. ESO already owns
refresh intervals, target `Secret` lifecycle, deletion behavior, status
conditions, and GitOps integration. A native operator, native ESO provider,
Secrets Store CSI provider, and PushSecret support are later roadmap items.

## Required For Public Releases

- Keep the repository name `bitwarden-eso-provider`, but consistently describe
  the project as a Bitwarden Password Manager / Vaultwarden vault-item ESO
  provider.
- Support explicit `id:<item-id>` and `name:<item-name>` selectors. Recommend
  `id:` selectors in production.
- Keep bare selector lookup only as pre-release compatibility: item ID first,
  then decrypted item name.
- Enforce optional provider-side selector policy with exact raw keys and raw key
  prefixes. Empty policy allows all keys visible to the configured account;
  configured policy denies every non-matching key with a redacted `403`.
  Document that the policy is item-key scoped, not property scoped.
- Fail selected shared organization items explicitly until organization-key
  decryption is implemented and live-tested.
- Fail `attachment.` and `attachments.` properties explicitly until attachment
  metadata lookup, download, decryption, and mapping are implemented.
- Expose low-cardinality metrics for HTTP requests, resolve outcomes, cache
  hits, cache refreshes, and last successful cache refresh age/timestamp.
- Ship Helm values and schema for selector policy and pod `hostAliases`.
- Provide examples for namespace-local `SecretStore`, warned
  `ClusterSecretStore`, common Kubernetes Secret types, Reloader, and
  NetworkPolicy starting points.
- Provide optional Grafana dashboard and PrometheusRule examples for the
  exported low-cardinality metrics.
- Document the tested migration target policy:
  `creationPolicy: Orphan`, `deletionPolicy: Retain`, and template
  `mergePolicy: Merge`.
- Recommend `field.<key>` for migrated Kubernetes keys to avoid collisions with
  Bitwarden login fields such as `username` and `password`.
- Document intentional empty target keys as ESO template data with
  `mergePolicy: Merge`.
- Keep chart NetworkPolicy opt-in for `v0.1.0`; backend, DNS, ESO, and
  Prometheus reachability is cluster-specific and a too-generic default can
  break first installs.
- Attach the packaged Helm chart to tagged GitHub Releases after the release
  image manifest has been built and scanned.

## Current Validation Baseline

The public `v0.1.0` baseline has been validated with:

- CI gates for formatting, clippy, tests with coverage, Helm rendering,
  markdown linting, observability examples, Gitleaks, Trivy filesystem scanning,
  cargo-deny, Checkov, and Dockerfile build checks.
- CodeQL code scanning for Rust and GitHub Actions workflow files.
- Local advisory scans used during release review, including Semgrep and
  SonarQube where available. These are review tools unless they are present in
  the GitHub workflow for that commit.
- `scripts/live-eso-smoke.sh` against Vaultwarden on a k3s cluster with
  selector policy enabled.
- `scripts/live-eso-smoke.sh` against Bitwarden Cloud with selector policy
  enabled.
- A tagged GitHub Release that publishes a multi-arch image and a packaged Helm
  chart from the release commit.
- Public repository controls for branch protection, tag protection, secret
  scanning, Dependabot alerts, security policy, issue templates, and CODEOWNERS.

## Future Release Gate

For each release:

- Run the GitHub CI workflow to green.
- Run the release workflow from the exact tag or commit being released.
- Confirm the release chart artifact is attached only after the image manifest
  and release image scan succeed.
- Run live smoke tests against Vaultwarden and Bitwarden Cloud with selector
  policy enabled.
- Record the image index digest and chart checksum in the release notes or
  release checklist.

## Deferred

- Organization/shared item decryption.
- Attachment support.
- Stale-cache-on-upstream-outage behavior.
- Disposable kind integration that does not require private credentials.
- OCI Helm chart publishing or GitHub Pages chart repository.
- Native operator or native ESO provider.
