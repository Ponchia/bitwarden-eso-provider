# Repository Governance

This document captures the repository settings expected before the project is
made public.

## Current Publication Gate

The repository is still private. On the current GitHub plan, branch protection
for this private repository is not available; GitHub returns a plan-gating error
when reading or writing `main` protection. Enable the protection rules below
immediately after making the repository public, or upgrade the private
repository plan before publication.

Dependabot vulnerability alerts, issue tracking, squash-only merging, branch
cleanup after merge, and repository topics are already enabled. GitHub currently
rejects secret scanning, push protection, and private vulnerability reporting for
this private repository, so enable them after the repository becomes public if
they are available in the repository security settings.

Do not make the repository public until the working tree is clean, all CI checks
pass, and the ignored local `.env.*` files have been removed or kept outside the
repository directory.

## Recommended Repository Settings

- Visibility: private until the first public-ready commit has passed CI, then
  public.
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
