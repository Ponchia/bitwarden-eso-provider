#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHART_DIR="${ROOT_DIR}/deploy/helm/bitwarden-eso-provider"

log() {
  printf '[bweso-smoke] %s\n' "$*"
}

fail() {
  printf '[bweso-smoke] error: %s\n' "$*" >&2
  exit 1
}

truthy() {
  case "${1:-}" in
    1 | true | TRUE | yes | YES | y | Y | on | ON) return 0 ;;
    *) return 1 ;;
  esac
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

force_sync_value() {
  printf '%s-%s' "$(date +%s)" "${RANDOM}"
}

env_or_empty() {
  local name="$1"
  printf '%s' "${!name:-}"
}

first_env() {
  local name value
  for name in "$@"; do
    value="$(env_or_empty "${name}")"
    if [[ -n "${value}" ]]; then
      printf '%s' "${value}"
      return 0
    fi
  done
  return 1
}

kubectl_cmd=(kubectl)
helm_cmd=(helm)
kube_context="$(first_env BWESO_E2E_KUBE_CONTEXT VWSO_E2E_KUBE_CONTEXT KUBE_CONTEXT || true)"
if [[ -n "${kube_context}" ]]; then
  kubectl_cmd+=(--context "${kube_context}")
  helm_cmd+=(--kube-context "${kube_context}")
fi

require_cmd kubectl
require_cmd helm
require_cmd jq
require_cmd cargo

namespace="$(first_env BWESO_E2E_NAMESPACE VWSO_E2E_NAMESPACE || true)"
namespace="${namespace:-bweso-live-smoke}"
release="$(first_env BWESO_E2E_RELEASE VWSO_E2E_RELEASE || true)"
release="${release:-bweso}"
image_repository="$(first_env BWESO_E2E_IMAGE_REPOSITORY VWSO_E2E_IMAGE_REPOSITORY || true)"
image_repository="${image_repository:-ghcr.io/ponchia/bitwarden-eso-provider}"
image_tag="$(first_env BWESO_E2E_IMAGE_TAG VWSO_E2E_IMAGE_TAG || true)"
credentials_secret="$(first_env BWESO_E2E_CREDENTIALS_SECRET VWSO_E2E_CREDENTIALS_SECRET || true)"
credentials_secret="${credentials_secret:-bweso-live-credentials}"
pull_secret="$(first_env BWESO_E2E_IMAGE_PULL_SECRET VWSO_E2E_IMAGE_PULL_SECRET || true)"
target_secret="bweso-smoke-secret"
selector_file="$(first_env BWESO_E2E_SELECTOR_FILE VWSO_E2E_SELECTOR_FILE || true)"
cleanup_namespace=true
if truthy "$(first_env BWESO_E2E_KEEP_NAMESPACE VWSO_E2E_KEEP_NAMESPACE || true)"; then
  cleanup_namespace=false
fi

[[ -n "${image_tag}" ]] || fail "set BWESO_E2E_IMAGE_TAG to the image tag to test"

single_origin_url="$(
  first_env \
    BWESO_TEST_SINGLE_ORIGIN_URL BWESO_SINGLE_ORIGIN_URL \
    VWSO_TEST_VAULTWARDEN_URL VWSO_VAULTWARDEN_URL || true
)"
identity_url="$(
  first_env \
    BWESO_TEST_IDENTITY_URL BWESO_IDENTITY_URL \
    VWSO_TEST_IDENTITY_URL VWSO_IDENTITY_URL || true
)"
api_url="$(first_env BWESO_TEST_API_URL BWESO_API_URL VWSO_TEST_API_URL VWSO_API_URL || true)"
client_id="$(first_env BWESO_TEST_CLIENT_ID BWESO_CLIENT_ID VWSO_TEST_CLIENT_ID VWSO_CLIENT_ID || true)"
client_secret="$(first_env BWESO_TEST_CLIENT_SECRET BWESO_CLIENT_SECRET VWSO_TEST_CLIENT_SECRET VWSO_CLIENT_SECRET || true)"
master_password="$(first_env BWESO_TEST_MASTER_PASSWORD BWESO_MASTER_PASSWORD VWSO_TEST_MASTER_PASSWORD VWSO_MASTER_PASSWORD || true)"

if [[ -n "${single_origin_url}" && ( -n "${identity_url}" || -n "${api_url}" ) ]]; then
  fail "use either BWESO_TEST_SINGLE_ORIGIN_URL/BWESO_SINGLE_ORIGIN_URL or split identity/api URLs, not both"
fi
if [[ -z "${single_origin_url}" && ( -z "${identity_url}" || -z "${api_url}" ) ]]; then
  fail "set a single-origin URL or both split endpoint URLs"
fi
[[ -n "${client_id}" ]] || fail "set BWESO_TEST_CLIENT_ID or BWESO_CLIENT_ID"
[[ -n "${client_secret}" ]] || fail "set BWESO_TEST_CLIENT_SECRET or BWESO_CLIENT_SECRET"
[[ -n "${master_password}" ]] || fail "set BWESO_TEST_MASTER_PASSWORD or BWESO_MASTER_PASSWORD"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "${tmp_dir}"
  if [[ "${cleanup_namespace}" == true ]]; then
    log "cleaning namespace ${namespace}"
    "${kubectl_cmd[@]}" delete namespace "${namespace}" --ignore-not-found=true --wait=false >/dev/null
    local deadline=$((SECONDS + 120))
    while (( SECONDS < deadline )); do
      if ! "${kubectl_cmd[@]}" get namespace "${namespace}" >/dev/null 2>&1; then
        return 0
      fi
      sleep 2
    done
    log "namespace ${namespace} is still terminating; inspect it manually if it does not disappear"
  else
    log "leaving namespace ${namespace} because BWESO_E2E_KEEP_NAMESPACE is set"
  fi
}
trap cleanup EXIT

selector_from_env() {
  local key property
  key="$(first_env BWESO_TEST_ITEM_KEY VWSO_TEST_ITEM_KEY || true)"
  property="$(first_env BWESO_TEST_PROPERTY VWSO_TEST_PROPERTY || true)"
  [[ -n "${key}" && -n "${property}" ]] || return 1
  jq -n --arg key "${key}" --arg property "${property}" \
    '{key: $key, property: $property}' >"${tmp_dir}/selector.json"
  selector_file="${tmp_dir}/selector.json"
}

selector_from_live_test() {
  export BWESO_TEST_CLIENT_ID="${client_id}"
  export BWESO_TEST_CLIENT_SECRET="${client_secret}"
  export BWESO_TEST_MASTER_PASSWORD="${master_password}"
  export BWESO_TEST_SELECTOR_OUTPUT="${tmp_dir}/selector.json"
  if [[ -n "${single_origin_url}" ]]; then
    export BWESO_TEST_SINGLE_ORIGIN_URL="${single_origin_url}"
    unset BWESO_TEST_IDENTITY_URL BWESO_TEST_API_URL
  else
    export BWESO_TEST_IDENTITY_URL="${identity_url}"
    export BWESO_TEST_API_URL="${api_url}"
    unset BWESO_TEST_SINGLE_ORIGIN_URL
  fi

  if [[ -z "$(first_env BWESO_TEST_ITEM_KEY VWSO_TEST_ITEM_KEY || true)" ]]; then
    export BWESO_TEST_ALLOW_ANY_ITEM="${BWESO_TEST_ALLOW_ANY_ITEM:-true}"
  fi

  log "selecting a live vault item without printing values"
  (cd "${ROOT_DIR}" && cargo test -p bweso-bitwarden --test live_bitwarden -- --nocapture)
  selector_file="${tmp_dir}/selector.json"
}

if [[ -n "${selector_file}" ]]; then
  [[ -f "${selector_file}" ]] || fail "selector file does not exist: ${selector_file}"
elif ! selector_from_env; then
  selector_from_live_test
fi

item_key="$(jq -r '.key // empty' "${selector_file}")"
property="$(jq -r '.property // empty' "${selector_file}")"
[[ -n "${item_key}" ]] || fail "selector must contain .key"
[[ -n "${property}" ]] || fail "selector must contain .property for ESO jsonPath extraction"

log "creating namespace ${namespace}"
"${kubectl_cmd[@]}" create namespace "${namespace}" --dry-run=client -o yaml | "${kubectl_cmd[@]}" apply -f - >/dev/null

ghcr_token="$(first_env BWESO_E2E_GHCR_TOKEN VWSO_E2E_GHCR_TOKEN || true)"
if [[ -n "${ghcr_token}" ]]; then
  pull_secret="${pull_secret:-ghcr-pull}"
  ghcr_user="$(first_env BWESO_E2E_GHCR_USER VWSO_E2E_GHCR_USER GITHUB_ACTOR || true)"
  ghcr_user="${ghcr_user:-bweso-smoke}"
  auth="$(printf '%s:%s' "${ghcr_user}" "${ghcr_token}" | base64 | tr -d '\n')"
  cat >"${tmp_dir}/dockerconfigjson" <<EOF
{"auths":{"ghcr.io":{"username":"${ghcr_user}","password":"${ghcr_token}","auth":"${auth}"}}}
EOF
  "${kubectl_cmd[@]}" -n "${namespace}" create secret generic "${pull_secret}" \
    --type=kubernetes.io/dockerconfigjson \
    --from-file=.dockerconfigjson="${tmp_dir}/dockerconfigjson" \
    --dry-run=client -o yaml | "${kubectl_cmd[@]}" apply -f - >/dev/null
fi

printf '%s' "${client_id}" >"${tmp_dir}/client-id"
printf '%s' "${client_secret}" >"${tmp_dir}/client-secret"
printf '%s' "${master_password}" >"${tmp_dir}/master-password"
chmod 0600 "${tmp_dir}/client-id" "${tmp_dir}/client-secret" "${tmp_dir}/master-password"

log "creating webhook credential Secret"
"${kubectl_cmd[@]}" -n "${namespace}" create secret generic "${credentials_secret}" \
  --from-file=client-id="${tmp_dir}/client-id" \
  --from-file=client-secret="${tmp_dir}/client-secret" \
  --from-file=master-password="${tmp_dir}/master-password" \
  --dry-run=client -o yaml | "${kubectl_cmd[@]}" apply -f - >/dev/null

helm_args=(
  upgrade --install "${release}" "${CHART_DIR}"
  --namespace "${namespace}"
  --set-string "image.repository=${image_repository}"
  --set-string "image.tag=${image_tag}"
  --set-string "credentials.existingSecret.name=${credentials_secret}"
  --set-string "config.cacheTtlSeconds=2"
)
if [[ -n "${single_origin_url}" ]]; then
  helm_args+=(--set-string "config.singleOriginUrl=${single_origin_url}")
else
  helm_args+=(--set-string "config.identityUrl=${identity_url}")
  helm_args+=(--set-string "config.apiUrl=${api_url}")
fi
if [[ -n "${pull_secret}" ]]; then
  helm_args+=(--set-string "imagePullSecrets[0].name=${pull_secret}")
fi

log "installing webhook chart ${image_repository}:${image_tag}"
"${helm_cmd[@]}" "${helm_args[@]}" >/dev/null

selector="app.kubernetes.io/instance=${release},app.kubernetes.io/name=bitwarden-eso-provider"
log "waiting for webhook rollout"
"${kubectl_cmd[@]}" -n "${namespace}" rollout status deployment -l "${selector}" --timeout=180s >/dev/null

service_name="$("${kubectl_cmd[@]}" -n "${namespace}" get svc -l "${selector}" -o jsonpath='{.items[0].metadata.name}')"
[[ -n "${service_name}" ]] || fail "could not find webhook Service"

cat >"${tmp_dir}/eso.yaml" <<EOF
apiVersion: external-secrets.io/v1
kind: SecretStore
metadata:
  name: bitwarden-live
  namespace: ${namespace}
spec:
  provider:
    webhook:
      url: "http://${service_name}.${namespace}.svc.cluster.local:8080/v1/resolve"
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
---
apiVersion: external-secrets.io/v1
kind: ExternalSecret
metadata:
  name: bweso-smoke
  namespace: ${namespace}
spec:
  refreshPolicy: Periodic
  refreshInterval: 10s
  secretStoreRef:
    name: bitwarden-live
    kind: SecretStore
  target:
    name: ${target_secret}
    creationPolicy: Owner
    deletionPolicy: Delete
  data:
    - secretKey: resolved
      remoteRef:
        key: "${item_key}"
        property: "${property}"
---
apiVersion: external-secrets.io/v1
kind: ExternalSecret
metadata:
  name: bweso-missing-property
  namespace: ${namespace}
spec:
  refreshPolicy: Periodic
  refreshInterval: 10s
  secretStoreRef:
    name: bitwarden-live
    kind: SecretStore
  target:
    name: bweso-missing-property-secret
    creationPolicy: Owner
    deletionPolicy: Delete
  data:
    - secretKey: resolved
      remoteRef:
        key: "${item_key}"
        property: "__bweso_missing_property_$(date +%s)"
---
apiVersion: external-secrets.io/v1
kind: ExternalSecret
metadata:
  name: bweso-missing-item
  namespace: ${namespace}
spec:
  refreshPolicy: Periodic
  refreshInterval: 10s
  secretStoreRef:
    name: bitwarden-live
    kind: SecretStore
  target:
    name: bweso-missing-item-secret
    creationPolicy: Owner
    deletionPolicy: Delete
  data:
    - secretKey: resolved
      remoteRef:
        key: "__bweso_missing_item_$(date +%s)"
        property: "${property}"
EOF

log "applying ESO smoke resources"
"${kubectl_cmd[@]}" apply -f "${tmp_dir}/eso.yaml" >/dev/null

log "waiting for successful sync"
"${kubectl_cmd[@]}" -n "${namespace}" wait externalsecret/bweso-smoke \
  --for=condition=Ready --timeout=180s >/dev/null

wait_secret_nonempty() {
  local name="$1"
  local key="$2"
  local deadline=$((SECONDS + 120))
  local encoded_value_len
  while (( SECONDS < deadline )); do
    encoded_value_len="$("${kubectl_cmd[@]}" -n "${namespace}" get secret "${name}" \
      -o "jsonpath={.data.${key}}" 2>/dev/null | wc -c | tr -d ' ')"
    if [[ "${encoded_value_len}" -gt 0 ]]; then
      return 0
    fi
    sleep 2
  done
  fail "Secret ${name} did not contain non-empty key ${key}"
}

wait_secret_nonempty "${target_secret}" resolved

log "checking Secret recreation"
"${kubectl_cmd[@]}" -n "${namespace}" delete secret "${target_secret}" >/dev/null
"${kubectl_cmd[@]}" -n "${namespace}" annotate externalsecret/bweso-smoke \
  "force-sync=$(force_sync_value)" --overwrite >/dev/null
"${kubectl_cmd[@]}" -n "${namespace}" wait externalsecret/bweso-smoke \
  --for=condition=Ready --timeout=180s >/dev/null
wait_secret_nonempty "${target_secret}" resolved

log "checking webhook restart plus resync"
"${kubectl_cmd[@]}" -n "${namespace}" rollout restart deployment -l "${selector}" >/dev/null
"${kubectl_cmd[@]}" -n "${namespace}" rollout status deployment -l "${selector}" --timeout=180s >/dev/null
"${kubectl_cmd[@]}" -n "${namespace}" annotate externalsecret/bweso-smoke \
  "force-sync=$(force_sync_value)" --overwrite >/dev/null
"${kubectl_cmd[@]}" -n "${namespace}" wait externalsecret/bweso-smoke \
  --for=condition=Ready --timeout=180s >/dev/null
wait_secret_nonempty "${target_secret}" resolved

wait_negative_absent() {
  local name="$1"
  local deadline=$((SECONDS + 120))
  local status
  while (( SECONDS < deadline )); do
    status="$("${kubectl_cmd[@]}" -n "${namespace}" get externalsecret "${name}" -o json \
      | jq -r '.status.conditions[]? | select(.type == "Ready") | "\(.status) \(.reason)"' \
      | tail -n 1)"
    if [[ "${status}" == "False SecretSyncedError" || "${status}" == "True SecretDeleted" ]]; then
      return 0
    fi
    sleep 2
  done
  fail "${name} did not reach a negative no-target state"
}

log "checking expected negative cases"
wait_negative_absent bweso-missing-property
wait_negative_absent bweso-missing-item
if "${kubectl_cmd[@]}" -n "${namespace}" get secret bweso-missing-property-secret >/dev/null 2>&1; then
  fail "missing-property ExternalSecret unexpectedly created a target Secret"
fi
if "${kubectl_cmd[@]}" -n "${namespace}" get secret bweso-missing-item-secret >/dev/null 2>&1; then
  fail "missing-item ExternalSecret unexpectedly created a target Secret"
fi

log "live ESO smoke test passed"
