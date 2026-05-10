# Helm Chart

The installable chart lives in
[`vaultwarden-secrets-operator`](vaultwarden-secrets-operator).

The default chart shape is intentionally small:

- Namespace-scoped deployment.
- No Kubernetes API RBAC; the webhook does not watch or write Kubernetes
  objects.
- No dashboard by default.
- Existing Kubernetes Secret for credentials by default.
- Optional NetworkPolicy template.

Render it locally with non-secret lint values:

```bash
helm lint deploy/helm/vaultwarden-secrets-operator -f deploy/helm/lint-values.yaml
helm template vwso deploy/helm/vaultwarden-secrets-operator \
  -f deploy/helm/lint-values.yaml \
  --namespace vwso-system
```
