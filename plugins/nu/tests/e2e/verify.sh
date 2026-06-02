#!/usr/bin/env bash

set -euo pipefail

workspace_root="/tmp/proto-nu-e2e"
project_dir="${workspace_root}/project"
plugin_path="/workspace/target/wasm32-wasip1/debug/nu.wasm"
version="0.112.2"
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
  "${proto_bin}" exec "nu@${version}" -- bash -lc "$1"
}

echo "building plugin wasm"
cd /workspace
"${cargo_bin}" build --target wasm32-wasip1 --package nu

echo "preparing proto workspace"
rm -rf "${workspace_root}"
mkdir -p "${project_dir}"

cat > "${project_dir}/.prototools" <<EOF
[plugins]
nu = "file://${plugin_path}"
EOF

echo "${version}" > "${project_dir}/.nu-version"

cd "${project_dir}"

echo "installing tool through proto"
"${proto_bin}" --log trace install nu "${version}"

echo "resolving installed paths"
nu_path="$("${proto_bin}" bin nu "${version}")"
bin_path="$("${proto_bin}" bin nu "${version}" --bin)"
shim_path="$("${proto_bin}" bin nu "${version}" --shim)"
exes_dir="$("${proto_bin}" bin nu "${version}" --dir exes)"
tool_dir="$(cd "$(dirname "${nu_path}")" && pwd)"

assert_eq "${nu_path}" "${tool_dir}/nu" "primary executable path"
assert_eq "${exes_dir}" "${tool_dir}" "primary executable directory"
assert_eq "${bin_path}" "/root/.proto/bin/nu" "linked binary path"
assert_eq "${shim_path}" "/root/.proto/shims/nu" "shim path"
assert_eq "$(readlink "${bin_path}")" "${nu_path}" "nu bin symlink target"
assert_eq "$(readlink /root/.proto/bin/nu_plugin_query)" "${tool_dir}/nu_plugin_query" "query plugin bin symlink target"

for path in \
  "${nu_path}" \
  "${tool_dir}/nu_plugin_query" \
  "${tool_dir}/nu_plugin_formats" \
  "${bin_path}" \
  "${shim_path}"
do
  test -x "${path}"
done

echo "verifying shim registry"
grep -F '"nu"' /root/.proto/shims/registry.json >/dev/null
grep -F '"nu_plugin_query"' /root/.proto/shims/registry.json >/dev/null
grep -F '"nu_plugin_formats"' /root/.proto/shims/registry.json >/dev/null

echo "verifying command execution through proto exec"
assert_contains "$(exec_in_tool 'command -v nu')" "/root/.proto/" "nu resolves within proto-managed paths"
assert_eq "$(exec_in_tool 'nu --version')" "${version}" "nu reports expected version"

echo "verifying detection through proto run"
"${proto_bin}" run nu -- --version | grep -F "${version}"
"${proto_bin}" run nu -- -c 'version | get version' | grep -F "${version}"

echo "docker e2e passed"
