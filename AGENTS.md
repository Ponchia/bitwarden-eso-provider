# Vaultwarden Secrets Operator Instructions

This is a standalone Rust repository inside the local multi-repo workspace.
Keep commits and generated artifacts local to this repository.

## Boundaries

- Reference repositories live in
  `../_docs/reference-map.mdbitwarden-eso-provider`.
- Do not vendor or copy implementation code from reference repositories.
- Treat Vaultwarden and Bitwarden source as reference material only unless a
  license review explicitly approves reuse.
- Do not put real Vaultwarden credentials, master passwords, API tokens, or
  Kubernetes kubeconfigs in this repository.

## Checks

Before handing off code changes, run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

If dependencies change, also run:

```bash
cargo generate-lockfile
cargo tree
```

## Security Defaults

- `unsafe` is forbidden.
- TLS verification is mandatory by default.
- Secret values must use redacting or zeroizing wrappers where practical.
- Logs may include item IDs, request IDs, namespace names, and key names, but not
  values.
- Deletion, rollout, and namespace-scoped behavior must be explicit in the
  Kubernetes-facing API.

