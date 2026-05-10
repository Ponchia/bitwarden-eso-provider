# ESO Webhook Install

Install External Secrets Operator first. This project provides the Bitwarden
Password Manager and Vaultwarden resolver behind ESO's generic webhook provider.

Create a namespace and credential Secret:

```bash
kubectl create namespace bweso-system
kubectl -n bweso-system create secret generic bweso-credentials \
  --from-literal=client-id='user.<uuid>' \
  --from-literal=client-secret='...' \
  --from-literal=master-password='...' \
  --from-literal=webhook-token='generate-a-long-random-token'
kubectl -n bweso-system label secret bweso-credentials \
  external-secrets.io/type=webhook
```

The provider rejects `/v1/resolve` calls without `Authorization: Bearer
<webhook-token>` by default. The label lets ESO's webhook provider expose the
Secret data as template variables for the authorization header.

Install the webhook for Vaultwarden or single-origin self-hosted Bitwarden:

```bash
helm upgrade --install bweso ./deploy/helm/bitwarden-eso-provider \
  --namespace bweso-system \
  --set-string image.tag='0.1.0' \
  --set-string config.singleOriginUrl='https://vaultwarden.example.com' \
  --set-string credentials.existingSecret.name=bweso-credentials \
  --set-string selectorPolicy.allowedKeys[0]='id:00000000-0000-0000-0000-000000000000'
```

Install the webhook for Bitwarden Cloud US:

```bash
helm upgrade --install bweso ./deploy/helm/bitwarden-eso-provider \
  --namespace bweso-system \
  --set-string image.tag='0.1.0' \
  --set-string config.identityUrl='https://identity.bitwarden.com' \
  --set-string config.apiUrl='https://api.bitwarden.com' \
  --set-string credentials.existingSecret.name=bweso-credentials \
  --set-string selectorPolicy.allowedKeys[0]='id:00000000-0000-0000-0000-000000000000'
```

Use `https://identity.bitwarden.eu` and `https://api.bitwarden.eu` for
Bitwarden EU.

`networkPolicy.enabled` is false by default. Enable it only after tailoring the
ingress rules for ESO/Prometheus and the egress rules for DNS plus your
Bitwarden/Vaultwarden backend. If the provider must reach an in-cluster ingress
or private address while still using the public Vaultwarden hostname for TLS
and HTTP host routing, configure `hostAliases`.

`selectorPolicy.allowedKeys` and `selectorPolicy.allowedKeyPrefixes` are
provider-side allowlists for the raw `remoteRef.key`. Empty lists allow all
items visible to the configured account, which is acceptable only when the
Bitwarden/Vaultwarden account itself is already scoped to the trust boundary.
When either list is configured, every non-matching selector returns `403`
without echoing the requested key.

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
        Authorization: Bearer {{ index .auth "webhook-token" }}
      secrets:
        - name: auth
          secretRef:
            name: bweso-credentials
      body: |
        {
          "remoteRef": {
            "key": {{ .remoteRef.key | toJson }},
            "property": {{ .remoteRef.property | toJson }}
          }
        }
      result:
        jsonPath: "$.data.value"
      timeout: 10s
```

Then create `ExternalSecret` resources that select item IDs/names and
properties. Prefer `id:<item-id>` selectors. `name:<item-name>` is supported for
operator convenience, and bare selectors currently try ID first then item name
for pre-release compatibility. Single-property responses always expose the
selected value at `$.data.value`, so the `SecretStore` does not need JSONPath
templating for field names. See [`../../deploy/eso`](../../deploy/eso) for
Secret type, Reloader, `ClusterSecretStore`, and NetworkPolicy examples.

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
