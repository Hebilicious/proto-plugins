#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../../../.." && pwd)"
image_name="proto-nu-e2e"

cleanup() {
  docker image rm -f "${image_name}" >/dev/null 2>&1 || true
}

trap cleanup EXIT

docker build \
  --progress=plain \
  --file "${script_dir}/Dockerfile" \
  --tag "${image_name}" \
  "${repo_root}"

docker run --rm "${image_name}" /workspace/plugins/nu/tests/e2e/verify.sh
