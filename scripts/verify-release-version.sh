#!/usr/bin/env bash
set -euo pipefail

tag="${1:-}"
if [[ -z "${tag}" || "${tag}" != v* ]]; then
  echo "usage: $0 v<version>" >&2
  exit 2
fi

version="${tag#v}"
chart_file="deploy/helm/vaultwarden-eso-provider/Chart.yaml"

chart_version="$(
  awk -F': *' '$1 == "version" { gsub(/"/, "", $2); print $2; exit }' "${chart_file}"
)"
app_version="$(
  awk -F': *' '$1 == "appVersion" { gsub(/"/, "", $2); print $2; exit }' "${chart_file}"
)"

if [[ "${chart_version}" != "${version}" ]]; then
  echo "Chart.yaml version ${chart_version} does not match tag ${tag}" >&2
  exit 1
fi

if [[ "${app_version}" != "${version}" ]]; then
  echo "Chart.yaml appVersion ${app_version} does not match tag ${tag}" >&2
  exit 1
fi

if ! grep -Eq "^## v?${version}([[:space:]-]|$)" CHANGELOG.md; then
  echo "CHANGELOG.md does not contain an entry for ${tag}" >&2
  exit 1
fi

python3 - "${version}" <<'PY'
import pathlib
import sys
import tomllib

expected = sys.argv[1]
paths = [
    pathlib.Path("crates/bweso-core/Cargo.toml"),
    pathlib.Path("crates/bweso-bitwarden/Cargo.toml"),
    pathlib.Path("crates/vaultwarden-eso-provider/Cargo.toml"),
]

errors = []
for path in paths:
    actual = tomllib.loads(path.read_text())["package"]["version"]
    if actual != expected:
        errors.append(f"{path}: package.version {actual} != {expected}")

if errors:
    print("\n".join(errors), file=sys.stderr)
    sys.exit(1)
PY

echo "release version gate passed for ${tag}"
