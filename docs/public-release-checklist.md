# Public Release Checklist

Required before calling a release generally usable:

- Commit the public collaboration files: `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, `SUPPORT.md`, issue templates, pull request template,
  and `CODEOWNERS`.
- Apply the GitHub repository settings documented in
  [`repository-governance.md`](repository-governance.md).
- Confirm the release branch is clean, pushed, and CI is green.
- Confirm `main` branch protection and release tag protection are active.
- Tag and publish a multi-arch image from GitHub Actions.
- Run `scripts/live-eso-smoke.sh` against real Vaultwarden and Bitwarden Cloud
  accounts with k3s or a kind cluster with ESO installed, with selector policy
  enabled.
- Verify migration examples use `creationPolicy: Orphan`,
  `deletionPolicy: Retain`, and template `mergePolicy: Merge`, and that a
  deleted target Secret is recreated with identical data.
- Verify examples use `field.<key>` for migrated Kubernetes keys that collide
  with Bitwarden login field names such as `username` and `password`.
- Verify intentional empty target keys are documented and rendered through
  `target.template.data` with `mergePolicy: Merge`.
- Publish the Helm chart artifact on the GitHub Release for the first
  pre-release.
- Review the release image SBOM/provenance output.
- Review logs for secret-value redaction under success and failure paths.
- Review metrics for secret-value and vault-item metadata redaction under success
  and failure paths.
- Review public HTTP error bodies for selector redaction because ESO can surface
  provider errors in `ExternalSecret` status and events.
- Generate Rust coverage with `cargo llvm-cov`; keep total line coverage above
  the conservative CI floor and review uncovered lines in security-sensitive
  paths before tagging.
- Verify `/livez`, `/readyz`, `/metrics`, default probes, and optional
  `ServiceMonitor` rendering.
- Import or lint the example Grafana dashboard and validate the example
  PrometheusRule before release.
- Keep Helm chart schema validation in sync with supported values.
- Verify provider-side selector policy returns redacted `403` failures.
- Verify selector policy documentation is clear that policy is item-key scoped,
  not property scoped.
- Verify unsupported organization/shared items and attachment properties fail
  explicitly and are documented.

## v0.1.0 Artifact Evidence

- Git tag: `v0.1.0`.
- Source commit: `03cdd76e11c02799b379efd6cdd447216894a493`.
- Image: `ghcr.io/ponchia/bitwarden-eso-provider:0.1.0`.
- Image index digest:
  `sha256:4847296e24f38c25e690da922264ec1f90e9f9ebf99d93a8c9775f7681134e1f`.
- Helm chart:
  `https://github.com/Ponchia/bitwarden-eso-provider/releases/download/v0.1.0/bitwarden-eso-provider-0.1.0.tgz`.
- Helm chart SHA256:
  `335053bd73b03a66d136c5bfa8081e5c3356b3b06a6af8e1721ee602c192b17a`.

Nice to have before `v1.0.0`:

- Native ESO provider assessment if the webhook contract becomes limiting.
- Disposable local kind integration coverage that does not need private
  credentials.
