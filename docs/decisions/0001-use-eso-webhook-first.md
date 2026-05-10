# 0001: Use External Secrets Operator Webhook First

## Status

Accepted for initial implementation.

## Context

A standalone Bitwarden-to-Kubernetes sync loop is easy to build but tends to mix
responsibilities:

- It fetches and decrypts source secrets.
- It owns Kubernetes Secret lifecycle.
- It invents refresh and deletion semantics.
- It often needs broad RBAC.

External Secrets Operator already solves the Kubernetes-side lifecycle and has a
generic webhook provider. Using that provider lets this project focus on the
Bitwarden-compatible work first.

## Decision

Implement a Rust HTTP provider for ESO before implementing a native Kubernetes
operator.

## Consequences

- Kubernetes users declare desired state with `SecretStore` and
  `ExternalSecret`.
- ESO owns Secret creation, updates, deletion, templating, and status.
- This project can remain a smaller Rust service.
- Advanced Bitwarden-specific UX may require a native controller later.
