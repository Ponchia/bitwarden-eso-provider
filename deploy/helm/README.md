# Helm Chart

The installable chart lives in
[`bitwarden-eso-provider`](bitwarden-eso-provider).

The default chart shape is intentionally small:

- Namespace-scoped deployment.
- No Kubernetes API RBAC; the webhook does not watch or write Kubernetes
  objects.
- No dashboard by default.
- Existing Kubernetes Secret for credentials by default.
- Optional NetworkPolicy template.

Render it locally with non-secret lint values:

```bash
helm lint deploy/helm/bitwarden-eso-provider -f deploy/helm/lint-values.yaml
helm template bweso deploy/helm/bitwarden-eso-provider \
  -f deploy/helm/lint-values.yaml \
  --namespace bweso-system
```
