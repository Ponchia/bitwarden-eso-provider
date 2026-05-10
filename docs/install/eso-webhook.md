# ESO Webhook Install

Install External Secrets Operator first. This project provides the Bitwarden
Password Manager and Vaultwarden resolver behind ESO's generic webhook provider.

Create a namespace and credential Secret:

```bash
kubectl create namespace bweso-system
kubectl -n bweso-system create secret generic bweso-credentials \
  --from-literal=client-id='user.<uuid>' \
  --from-literal=client-secret='...' \
  --from-literal=master-password='...'
```

Install the webhook for Vaultwarden or single-origin self-hosted Bitwarden:

```bash
helm upgrade --install bweso ./deploy/helm/bitwarden-eso-provider \
  --namespace bweso-system \
  --set-string image.tag='0.1.0' \
  --set-string config.singleOriginUrl='https://vaultwarden.example.com' \
  --set-string credentials.existingSecret.name=bweso-credentials
```

Install the webhook for Bitwarden Cloud US:

```bash
helm upgrade --install bweso ./deploy/helm/bitwarden-eso-provider \
  --namespace bweso-system \
  --set-string image.tag='0.1.0' \
  --set-string config.identityUrl='https://identity.bitwarden.com' \
  --set-string config.apiUrl='https://api.bitwarden.com' \
  --set-string credentials.existingSecret.name=bweso-credentials
```

Use `https://identity.bitwarden.eu` and `https://api.bitwarden.eu` for
Bitwarden EU.

Point ESO at the webhook:

```yaml
apiVersion: external-secrets.io/v1
kind: SecretStore
metadata:
  name: bitwarden
spec:
  provider:
    webhook:
      url: "http://bweso-bitwarden-eso-provider.bweso-system.svc.cluster.local:8080/v1/resolve"
      method: POST
      headers:
        Content-Type: application/json
      body: |
        {
          "remoteRef": {
            "key": "{{ .remoteRef.key }}",
            "property": "{{ .remoteRef.property }}"
          }
        }
      result:
        jsonPath: "$.data.{{ .remoteRef.property }}"
      timeout: 10s
```

Then create `ExternalSecret` resources that select item IDs/names and
properties. See [`../../deploy/eso`](../../deploy/eso) for examples.

The chart configures startup, liveness, and readiness probes by default:

```yaml
probes:
  startup:
    httpGet:
      path: /livez
      port: http
  liveness:
    httpGet:
      path: /livez
      port: http
  readiness:
    httpGet:
      path: /readyz
      port: http
```

The provider always serves Prometheus-format metrics at `/metrics`. If the
Prometheus Operator CRDs are installed, enable a `ServiceMonitor`:

```bash
helm upgrade --install bweso ./deploy/helm/bitwarden-eso-provider \
  --namespace bweso-system \
  --reuse-values \
  --set metrics.serviceMonitor.enabled=true
```

See [`../operations/observability.md`](../operations/observability.md) for the
full metric list and operational notes.
