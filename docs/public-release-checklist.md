# Public Release Checklist

Required before calling a release generally usable:

- Commit the public collaboration files: `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, `SUPPORT.md`, issue templates, pull request template,
  and `CODEOWNERS`.
- Apply the GitHub repository settings documented in
  [`repository-governance.md`](repository-governance.md).
- Make the repository public only after the branch is clean, pushed, and CI is
  green.
- Enable `main` branch protection immediately after GitHub allows it for the
  repository.
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
- Verify `/livez`, `/readyz`, `/metrics`, default probes, and optional
  `ServiceMonitor` rendering.
- Keep Helm chart schema validation in sync with supported values.
- Verify provider-side selector policy returns redacted `403` failures.
- Verify unsupported organization/shared items and attachment properties fail
  explicitly and are documented.

Nice to have before `v1.0.0`:

- Native ESO provider assessment if the webhook contract becomes limiting.
- Disposable local kind integration coverage that does not need private
  credentials.
