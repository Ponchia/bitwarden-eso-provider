# Contributing

This project is private during the design spike. Contributions should keep the
repository public-ready:

- Use small commits.
- Keep reference code out of the source tree.
- Add tests for behavior that touches auth, decryption, sync semantics, or
  Kubernetes-facing contracts.
- Update `docs/decisions` when architecture changes.
- Keep defaults conservative, especially around deletion, TLS, logging, and RBAC.

Run the standard checks before opening a pull request:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

