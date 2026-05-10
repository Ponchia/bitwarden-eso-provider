# Repository Governance

This document captures the repository settings expected before the project is
made public.

## Publication Gate

Before publication, verify that the repository settings below are either already
enabled or scheduled for the visibility change. Some GitHub security and branch
protection controls are plan-dependent for private repositories; enable them
immediately when they become available.

Dependabot vulnerability alerts, issue tracking, squash-only merging, branch
cleanup after merge, and repository topics should be enabled before publication.
Enable secret scanning, push protection, and private vulnerability reporting as
soon as GitHub exposes those controls for the repository.

Do not make the repository public until the working tree is clean, all CI checks
pass, and the ignored local `.env.*` files have been removed or kept outside the
repository directory.

## Recommended Repository Settings

- Visibility: public after the first public-ready commit has passed CI.
- Issues: enabled.
- Discussions: optional; enable only if issue traffic becomes noisy.
- Wiki: disabled. Keep docs in Git.
- Projects: disabled unless there is an active public roadmap board.
- Merge strategy: squash merge enabled, merge commits disabled, rebase merge
  disabled.
- Squash commit message: pull request title and description.
- Delete branch on merge: enabled.
- Auto-merge: keep manual for normal pull requests. Dependabot patch-only PRs
  may be merged automatically by the repository workflow after CI succeeds.
- Allow update branch: enabled.
- Topics: `bitwarden`, `vaultwarden`, `external-secrets`, `external-secrets-operator`,
  `kubernetes`, `rust`, `helm`, `secrets-management`.
- Security features: enable secret scanning, push protection, Dependabot alerts,
  Dependabot security updates, and private vulnerability reporting when
  available for the repository.

## Main Branch Protection

Protect `main` with these rules once GitHub allows branch protection:

- Require a pull request before merging.
- Require at least one approving review.
- Require review from Code Owners.
- Dismiss stale approvals when new commits are pushed.
- Require conversation resolution before merging.
- Require status checks to pass before merging.
- Require branches to be up to date before merging.
- Required checks:
  - `Rust`
  - `Helm`
  - `Security`
  - `Docker`
- Include administrators.
- Block force pushes.
- Block deletions.

Do not require signed commits or a merge queue for the first public release.
Those can be added later once external contribution volume justifies them.

## Release Permissions

Only maintainers should be able to push tags matching `v*`. Release tags publish
multi-arch images and Helm chart artifacts, so tag protection should be enabled
before the first public stable release.

## Dependabot Policy

Dependabot patch-only PRs are safe to merge automatically after CI succeeds.
Minor and major updates stay manual because Rust `0.x` crates can contain API
changes in semver-minor updates, and Docker/toolchain updates can change clippy
behavior. Related RustCrypto KDF/MAC/hash crates are grouped so Dependabot does
not open incompatible one-crate-at-a-time updates.
