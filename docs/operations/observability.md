# Observability

Bitwarden ESO Provider is a stateless HTTP webhook. It should be observable as a
normal Kubernetes service without exposing vault item names, properties, secret
values, API tokens, or derived keys.

Public error responses are intentionally coarse for the same reason. ESO may
copy webhook error bodies into `ExternalSecret` status or events, so selector
values are not echoed back on failures.

`/v1/resolve` requires a bearer token by default. Authentication failures are
reported as HTTP `401` with the `auth` resolution error class and no token or
selector detail.

## Health Endpoints

The provider exposes dedicated probe endpoints:

| Endpoint | Meaning |
| --- | --- |
| `/livez` | Process is alive and the HTTP server can respond. |
| `/readyz` | Pod is ready to receive webhook traffic. Returns `503` after shutdown starts. |
| `/metrics` | Prometheus text exposition. |

The Helm chart enables startup, liveness, and readiness probes by default.
Liveness intentionally does not call Bitwarden or Vaultwarden. Upstream outages
should surface as request failures and metrics, not as restart loops.

## Metrics

Metrics are exposed in Prometheus text format with this content type:

```text
text/plain; version=0.0.4; charset=utf-8
```

Current series:

| Metric | Type | Labels | Notes |
| --- | --- | --- | --- |
| `bweso_build_info` | gauge | `version` | Static build metadata. |
| `bweso_process_start_time_seconds` | gauge | none | Process start timestamp. |
| `bweso_uptime_seconds` | gauge | none | Process uptime. |
| `bweso_ready` | gauge | none | `1` while `/readyz` is healthy, `0` during shutdown. |
| `bweso_http_requests_total` | counter | `method`, `route`, `status` | Low-cardinality HTTP request count. |
| `bweso_http_request_duration_seconds` | histogram | `method`, `route`, `status` | HTTP request latency. |
| `bweso_resolve_requests_total` | counter | `outcome`, `error_kind`, `status` | Secret resolution count. |
| `bweso_resolve_duration_seconds` | histogram | `outcome`, `error_kind`, `status` | End-to-end resolution latency. |

Resolution labels are intentionally coarse. They expose classes like
`auth`, `validation`, `not_found`, `ambiguous_selector`, `upstream_http`,
`upstream_status`, `crypto`, `key_derivation`, `kdf_parameters`,
`sync_payload`, `endpoint`, and `unsupported_version`. They do not expose vault
item IDs, item names, requested properties, usernames, URLs, or secret values.

## Prometheus Operator

If the Prometheus Operator CRDs are installed, enable the chart's
`ServiceMonitor`:

```yaml
metrics:
  serviceMonitor:
    enabled: true
    interval: 30s
    scrapeTimeout: 10s
```

If `networkPolicy.enabled=true`, make sure the ingress rules allow traffic from
the Prometheus scraper namespace to the provider Service on the `http` port.

## Useful Alerts

Example PromQL starting points:

```promql
bweso_ready == 0
```

```promql
sum(rate(bweso_resolve_requests_total{outcome="error"}[5m])) > 0
```

```promql
histogram_quantile(
  0.95,
  sum(rate(bweso_resolve_duration_seconds_bucket[5m])) by (le)
)
```
