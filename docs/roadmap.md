# Roadmap

## Phase 0: Design Spike

- Map Vaultwarden auth and cipher endpoints.
- Map Bitwarden client-side decryption flow.
- Document 1Password Operator use cases and restart semantics.
- Validate ESO webhook request and response contracts.
- Decide field addressing syntax.

Initial notes on authenticated encrypted string handling live in
[`crypto-notes.md`](crypto-notes.md).

## Phase 1: Local Provider

- Implement Vaultwarden login with user API key. Initial fake-server coverage is
  in place.
- Implement vault unlock and item decryption with tests from deterministic
  fixtures.
- Add field extraction for login, secure note, SSH key, and custom fields.
- Wire runtime configuration into `vwso-eso-webhook`.
- Add cache with explicit TTL and single-flight refresh.
- Add redacted metrics.
- Add a local CLI smoke-test command.

## Phase 2: ESO Webhook

- Implement `/v1/resolve` contract.
- Add SecretStore and ExternalSecret examples.
- Add Helm chart with namespace-scoped default RBAC.
- Add Docker image and SBOM generation.
- Add integration tests with local Vaultwarden and kind.

## Phase 3: Kubernetes Ergonomics

- Document rollout/restart options with Reloader and GitOps annotations.
- Add examples for TLS, docker config, basic auth, and multiline files.
- Decide whether a native controller or native ESO provider is needed.
