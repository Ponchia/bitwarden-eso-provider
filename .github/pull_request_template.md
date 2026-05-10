## Summary

-

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --locked --workspace --all-targets -- -D warnings`
- [ ] `cargo test --locked --workspace --all-targets`
- [ ] `helm lint deploy/helm/bitwarden-eso-provider -f deploy/helm/lint-values.yaml`
- [ ] `helm template bweso deploy/helm/bitwarden-eso-provider -f deploy/helm/lint-values.yaml --namespace bweso-system`

## Security And Compatibility

- [ ] No secrets, vault item names, vault item IDs, requested properties, API tokens, master passwords, or derived keys are logged or exposed in metrics/errors.
- [ ] TLS, authentication, RBAC, NetworkPolicy, and deletion defaults stay conservative.
- [ ] Bitwarden Cloud and Vaultwarden compatibility is unchanged or documented.
- [ ] User-facing docs, chart values, examples, and schema were updated when behavior changed.
