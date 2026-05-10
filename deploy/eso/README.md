# ESO Examples

These examples use placeholders only. Replace item IDs, namespaces, service
names, and image names before applying them.

Recommended order:

- `secretstore-webhook.example.yaml`: namespace-local `SecretStore`.
- `externalsecret.example.yaml`: single-field sync using `id:<item-id>`.
- `secret-types.example.yaml`: docker config JSON, basic auth, SSH auth, and
  multiline files.
- `reloader.example.yaml`: Stakater Reloader annotation pattern.
- `clustersecretstore.warning.example.yaml`: shared store pattern with the
  security warning that should accompany it.
- `networkpolicy-vaultwarden-in-cluster.example.yaml`: narrow in-cluster
  Vaultwarden egress starting point.
- `networkpolicy-bitwarden-cloud.example.yaml`: Bitwarden Cloud egress starting
  point. Native Kubernetes NetworkPolicy cannot restrict by DNS name; use a CNI
  or egress gateway with FQDN policy if strict hostname enforcement is required.

The Helm chart leaves NetworkPolicy disabled by default. Enable
`networkPolicy.enabled` only after adapting one of these examples to the exact
DNS, ingress, Vaultwarden, ESO, and Prometheus paths in your cluster.

Prefer one dedicated Bitwarden/Vaultwarden user and one namespace-local
`SecretStore` per trust boundary. Namespace-local `SecretStore` resources read
webhook auth from a same-namespace token Secret such as `bweso-webhook-auth`;
the provider runtime credentials in `bweso-system` should not be reused across
namespaces as the ESO auth Secret. Configure the Helm chart's
`selectorPolicy.allowedKeys` or `selectorPolicy.allowedKeyPrefixes` whenever
the provider credentials can see more vault items than the namespace should
read.

The ExternalSecret examples use `creationPolicy: Orphan`,
`deletionPolicy: Retain`, and template `mergePolicy: Merge`. That combination
lets ESO recreate a missing target Secret, avoids deleting target Secrets when
an ExternalSecret is removed, and prevents template-only keys from replacing
provider-sourced keys.

For migrated Kubernetes Secret keys, prefer `field.<key>` properties. Bare
`username` and `password` mean Bitwarden login fields, while `field.username`
and `field.password` mean custom fields with those names.
