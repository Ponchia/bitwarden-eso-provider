# 0004 — Hot-Reloadable Selector Policy

## Status

Accepted, 2026-05-17.

## Context

The selector allow-list (`BWESO_ALLOWED_KEYS` /
`BWESO_ALLOWED_KEY_PREFIXES`) was parsed once in `main()` via
`SelectorPolicy::from_args` and stored in the cloned `AppState`. There was
no runtime reload path.

Operators driving the allow-list from a Kubernetes ConfigMap (the natural
GitOps pattern: regenerate the policy, commit, let the cluster apply it)
hit a sharp edge. Mounting the ConfigMap value as the `BWESO_ALLOWED_KEYS`
env var via `configMapKeyRef` snapshots it at pod start. Changing the
ConfigMap does not change the Deployment pod spec, so the pod is never
rolled and the in-memory policy keeps denying the new key. The provider
correctly answers `403 policy_denied`, but ESO's webhook provider masks it
as a generic `SecretSyncedError`, so the root cause (a stale in-memory
policy that needed a manual `kubectl rollout restart`) is non-obvious.
This was reported from production use as
[issue #3](https://github.com/Ponchia/vaultwarden-eso-provider/issues/3).

Existing mitigations were insufficient: prefix entries reduce churn but
weaken per-item scoping; the Reloader example reloads consumer workloads,
not the provider's own policy; Reloader on the provider Deployment still
means a pod restart per onboarding and pushes a provider-specific
operational concern onto every adopter.

## Decision

Add **optional file-backed policy sources that are re-read at runtime**:

- New args `BWESO_ALLOWED_KEYS_FILE` / `BWESO_ALLOWED_KEY_PREFIXES_FILE`
  (consistent with the existing `*_FILE` credential pattern) and
  `BWESO_POLICY_RELOAD_INTERVAL_SECONDS` (default `30`, `0` disables
  reloading).
- File entries are unioned with the inline flag/env entries. File format:
  one entry per line, commas also split, surrounding whitespace trimmed,
  blank lines and `#` comment lines ignored, remaining entries validated
  non-empty.
- `SelectorPolicy` becomes a hot-swappable holder
  (`RwLock<Arc<PolicyRules>>`, no new dependency). A background task
  re-evaluates the file sources on the interval and atomically swaps the
  active rules only when they change.
- The task is spawned only when a file source is configured; otherwise
  behavior is byte-for-byte unchanged (single startup evaluation). The
  task exits promptly on shutdown via a `Lifecycle` notification
  (`tokio::select!` on the tick vs. a shutdown signal), not only on its
  next tick.
- Configured-but-unreadable/invalid files fail fast at startup; a
  transient failure during a later reload is logged and the last
  known-good policy keeps serving (fail to last-good, not open/closed).
- **Configured-empty is not allow-all.** The legacy "empty rule set ⇒
  allow all" applies only when *no* policy source is configured. When any
  file source is configured, an evaluation that yields zero entries
  (empty, comment-only, or a ConfigMap accidentally emptied by a bad
  GitOps render) is an error: it fails fast at startup and fails to
  last-good on reload, rather than silently widening to allow-all on the
  no-restart path.
- Helm chart gains `selectorPolicy.configMap` (mounted read-only at
  `/etc/bweso/policy`) and `selectorPolicy.reloadIntervalSeconds`.

## Rationale

- Removes the missed-restart failure mode: ConfigMap-driven onboarding
  becomes pure GitOps, applied within one interval with no restart.
- Conservative by default: no file ⇒ no task ⇒ identical behavior, so
  existing deployments are unaffected and the security posture is unchanged.
- No new dependency. The read path stays a cheap uncontended `Arc` clone;
  swaps are rare (reload interval).
- Redaction preserved: reload logs counts only, never selector keys, and
  errors carry file paths, not vault keys.

## Consequences

- A file-backed policy widens/narrows access on the reload interval
  without an audited pod rollout. `reloadIntervalSeconds: 0` (or omitting
  the ConfigMap entirely) keeps the stricter rollout-gated behavior for
  high-assurance boundaries.
- Mounted-ConfigMap propagation latency (kubelet sync) adds to the
  configured interval before a change takes effect.
