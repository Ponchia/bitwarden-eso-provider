# Public Readiness Plan

This file records the pre-`v0.1.0` public-readiness decisions for Bitwarden ESO
Provider.

## Product Shape

The first public release remains an External Secrets Operator webhook provider
for Bitwarden Password Manager and Vaultwarden vault items.

Do not build a native Kubernetes operator before `v0.1.0`. ESO already owns
refresh intervals, target `Secret` lifecycle, deletion behavior, status
conditions, and GitOps integration. A native operator, native ESO provider,
Secrets Store CSI provider, and PushSecret support are later roadmap items.

## Required For Public Release

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
- Fail selected shared organization items explicitly until organization-key
  decryption is implemented and live-tested.
- Fail `attachment.` and `attachments.` properties explicitly until attachment
  metadata lookup, download, decryption, and mapping are implemented.
- Expose low-cardinality metrics for HTTP requests, resolve outcomes, cache
  hits, cache refreshes, and last successful cache refresh age/timestamp.
- Ship Helm values and schema for selector policy.
- Provide examples for namespace-local `SecretStore`, warned
  `ClusterSecretStore`, common Kubernetes Secret types, Reloader, and
  NetworkPolicy starting points.
- Attach the packaged Helm chart to tagged GitHub Releases.

## Validation Gate

Before making the repository public:

- Run the full Rust and Helm checks from `AGENTS.md`.
- Run `scripts/live-eso-smoke.sh` against Vaultwarden on the k3s cluster with
  selector policy enabled.
- Run `scripts/live-eso-smoke.sh` against Bitwarden Cloud with selector policy
  enabled.
- Confirm CI is green after pushing.
- Keep the repo private until the branch is clean, pushed, smoke-tested, and
  release docs are coherent.

After making the repository public:

- Enable documented `main` branch protection immediately.
- Confirm secret scanning, Dependabot alerts, security policy, issue templates,
  and CODEOWNERS are active.

## Deferred

- Organization/shared item decryption.
- Attachment support.
- Stale-cache-on-upstream-outage behavior.
- Disposable kind integration that does not require private credentials.
- OCI Helm chart publishing or GitHub Pages chart repository.
- Native operator or native ESO provider.
