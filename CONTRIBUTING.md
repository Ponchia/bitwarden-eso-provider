# Contributing

Thanks for helping improve Bitwarden ESO Provider. This project sits between a
password manager, External Secrets Operator, and Kubernetes Secrets, so changes
must keep security, compatibility, and operational behavior clear.

## Ground Rules

- Follow the [Code of Conduct](CODE_OF_CONDUCT.md).
- Do not include real secrets, master passwords, API tokens, vault item IDs,
  vault item names, Kubernetes Secret values, kubeconfigs, or private
  infrastructure details in issues, pull requests, tests, examples, screenshots,
  logs, or commit history.
- Keep reference code out of the source tree. Notes and source links belong
  in documentation.
- Prefer small pull requests with a focused behavior change.
- Add tests for behavior that touches auth, decryption, sync semantics,
  redaction, metrics, Helm rendering, or Kubernetes-facing contracts.
- Update docs, examples, chart values, and schema when user-facing behavior
  changes.
- Update `docs/decisions` when architecture changes.
- Keep defaults conservative, especially around deletion, TLS, logging, auth,
  NetworkPolicy, and RBAC.

## Development

Install a stable Rust toolchain and Helm. Then run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
helm lint deploy/helm/bitwarden-eso-provider -f deploy/helm/lint-values.yaml
helm template bweso deploy/helm/bitwarden-eso-provider -f deploy/helm/lint-values.yaml --namespace bweso-system
```

If dependencies change, also run:

```bash
cargo generate-lockfile
cargo tree
```

## Pull Requests

Pull requests should explain:

- What changed and why.
- Which backend was affected: Bitwarden Cloud, Vaultwarden, self-hosted
  Bitwarden, ESO, Helm, or packaging.
- How the change was validated.
- Whether there are compatibility or security implications.

The default merge path is expected to be squash merge after required checks and
review pass.
