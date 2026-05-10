# Public Release Checklist

Required before calling a release generally usable:

- Tag and publish a multi-arch image from GitHub Actions.
- Run `scripts/live-eso-smoke.sh` against a real Vaultwarden instance and k3s or
  kind cluster with ESO installed.
- Publish the Helm chart artifact or document chart install from source for the
  first pre-release.
- Review the release image SBOM/provenance output.
- Add local kind integration coverage with a disposable Vaultwarden fixture.
- Review logs for secret-value redaction under success and failure paths.
- Decide whether unsupported organization/shared item decryption should be
  hard-fail documented behavior or implemented before `v1.0.0`.

Nice to have before `v1.0.0`:

- Native ESO provider assessment if the webhook contract becomes limiting.
- Reloader example manifests.
- NetworkPolicy examples for in-cluster Vaultwarden and public Bitwarden Cloud
  egress.
- Chart schema validation.
