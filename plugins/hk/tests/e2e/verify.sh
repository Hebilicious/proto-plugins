#!/usr/bin/env bash

set -euo pipefail

workspace_root="/tmp/proto-hk-e2e"
project_dir="${workspace_root}/project"
plugin_path="/workspace/target/wasm32-wasip1/debug/hk.wasm"
version="1.46.0"
cargo_bin="/usr/local/cargo/bin/cargo"
proto_bin="/root/.proto/bin/proto"

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"

  if [[ "${actual}" != "${expected}" ]]; then
    echo "assertion failed: ${message}" >&2
    echo "expected: ${expected}" >&2
    echo "actual:   ${actual}" >&2
    exit 1
  fi
}

assert_contains() {
  local actual="$1"
  local needle="$2"
  local message="$3"

  if [[ "${actual}" != *"${needle}"* ]]; then
    echo "assertion failed: ${message}" >&2
    echo "expected to contain: ${needle}" >&2
    echo "actual: ${actual}" >&2
    exit 1
  fi
}

exec_in_tool() {
  "${proto_bin}" exec "hk@${version}" -- bash -lc "$1"
}

echo "building plugin wasm"
cd /workspace
"${cargo_bin}" build --target wasm32-wasip1 --package hk

echo "preparing proto workspace"
rm -rf "${workspace_root}"
mkdir -p "${project_dir}"

cat > "${project_dir}/.prototools" <<EOF
[plugins]
hk = "file://${plugin_path}"
EOF

echo "${version}" > "${project_dir}/.hk-version"

cd "${project_dir}"

echo "installing tool through proto"
"${proto_bin}" --log trace install hk "${version}"

echo "resolving installed paths"
hk_path="$("${proto_bin}" bin hk "${version}")"
bin_path="$("${proto_bin}" bin hk "${version}" --bin)"
shim_path="$("${proto_bin}" bin hk "${version}" --shim)"
exes_dir="$("${proto_bin}" bin hk "${version}" --dir exes)"
tool_dir="$(cd "$(dirname "${hk_path}")" && pwd)"

assert_eq "${hk_path}" "${tool_dir}/hk" "primary executable path"
assert_eq "${exes_dir}" "${tool_dir}" "primary executable directory"
assert_eq "${bin_path}" "/root/.proto/bin/hk" "linked binary path"
assert_eq "${shim_path}" "/root/.proto/shims/hk" "shim path"
assert_eq "$(readlink "${bin_path}")" "${hk_path}" "hk bin symlink target"

for path in \
  "${hk_path}" \
  "${bin_path}" \
  "${shim_path}"
do
  test -x "${path}"
done

echo "verifying shim registry"
grep -F '"hk"' /root/.proto/shims/registry.json >/dev/null

echo "verifying command execution through proto exec"
assert_contains "$(exec_in_tool 'command -v hk')" "/root/.proto/" "hk resolves within proto-managed paths"
assert_eq "$(exec_in_tool 'hk --version')" "hk ${version}" "hk reports expected version"

echo "verifying detection through proto run"
"${proto_bin}" run hk -- --version | grep -F "hk ${version}"

echo "docker e2e passed"
