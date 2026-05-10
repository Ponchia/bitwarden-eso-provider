# Roadmap

## Phase 0: Design Spike

- Map Bitwarden/Vaultwarden auth and cipher endpoints.
- Map Bitwarden client-side decryption flow.
- Document 1Password Operator use cases and restart semantics.
- Validate ESO webhook request and response contracts.
- Decide field addressing syntax.

Initial notes on authenticated encrypted string handling live in
[`crypto-notes.md`](crypto-notes.md).

## Phase 1: Local Provider

- Implement Bitwarden-compatible login with user API key. Initial fake-server
  coverage is in place.
- Implement vault unlock and item decryption with tests from deterministic
  fixtures.
- Add field extraction for login, secure note, SSH key, and custom fields.
- Wire runtime configuration into `bitwarden-eso-provider`.
- Add cache with explicit TTL and single-flight refresh.
- Add split Bitwarden Cloud endpoint support with fake-server coverage.
- Add redacted metrics, health probes, and graceful shutdown readiness behavior.
- Add optional selector allowlists and explicit unsupported shared-item and
  attachment failures.
- Add a local CLI smoke-test command.
- Add opt-in live Bitwarden-compatible smoke test.

## Phase 2: ESO Webhook

- Implement `/v1/resolve` contract.
- Add SecretStore and ExternalSecret examples.
- Add Helm chart with namespace-scoped default deployment and no Kubernetes API
  permissions.
- Add Docker image and SBOM generation.
- Add integration tests with local Vaultwarden and kind.
- Add repeatable live k3s/ESO smoke script.
- Add optional Prometheus Operator `ServiceMonitor` support.

## Phase 3: Kubernetes Ergonomics

- Document rollout/restart options with Reloader and GitOps annotations.
- Add examples for TLS, docker config, basic auth, and multiline files.
- Add organization/shared item decryption after fixture and live coverage.
- Add attachment download/decryption only after a clear Kubernetes mapping is
  documented.
- Decide whether a native controller or native ESO provider is needed.
