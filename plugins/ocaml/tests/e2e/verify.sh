#!/usr/bin/env bash

set -euo pipefail

workspace_root="/tmp/proto-ocaml-e2e"
project_dir="${workspace_root}/project"
plugin_path="/workspace/target/wasm32-wasip1/debug/ocaml.wasm"
version="5.4.1"
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
  "${proto_bin}" exec "ocaml@${version}" -- bash -lc "$1"
}

echo "building plugin wasm"
cd /workspace
"${cargo_bin}" build --target wasm32-wasip1 --package ocaml

echo "preparing proto workspace"
rm -rf "${workspace_root}"
mkdir -p "${project_dir}"

cat > "${project_dir}/.prototools" <<EOF
[plugins]
ocaml = "file://${plugin_path}"
EOF

echo "${version}" > "${project_dir}/.ocaml-version"

cd "${project_dir}"

echo "installing tool through proto"
"${proto_bin}" --log trace install ocaml "${version}"

echo "resolving installed paths"
ocaml_path="$("${proto_bin}" bin ocaml "${version}")"
bin_path="$("${proto_bin}" bin ocaml "${version}" --bin)"
shim_path="$("${proto_bin}" bin ocaml "${version}" --shim)"
exes_dir="$("${proto_bin}" bin ocaml "${version}" --dir exes)"
tool_dir="$(cd "$(dirname "${ocaml_path}")/../.." && pwd)"

opam_path="${tool_dir}/bin/opam"
ocamlc_path="${tool_dir}/_opam/bin/ocamlc"
ocamlopt_path="${tool_dir}/_opam/bin/ocamlopt"
ocamldep_path="${tool_dir}/_opam/bin/ocamldep"
dune_path="${tool_dir}/_opam/bin/dune"

assert_eq "${ocaml_path}" "${tool_dir}/_opam/bin/ocaml" "primary executable path"
assert_eq "${exes_dir}" "${tool_dir}/bin" "primary executable directory"
assert_eq "${bin_path}" "/root/.proto/bin/ocaml" "linked binary path"
assert_eq "${shim_path}" "/root/.proto/shims/ocaml" "shim path"
assert_eq "$(readlink "${bin_path}")" "${ocaml_path}" "ocaml bin symlink target"
assert_eq "$(readlink /root/.proto/bin/opam)" "${opam_path}" "opam bin symlink target"
assert_eq "$(readlink /root/.proto/bin/ocamlc)" "${ocamlc_path}" "ocamlc bin symlink target"
assert_eq "$(readlink /root/.proto/bin/ocamlopt)" "${ocamlopt_path}" "ocamlopt bin symlink target"
assert_eq "$(readlink /root/.proto/bin/ocamldep)" "${ocamldep_path}" "ocamldep bin symlink target"
assert_eq "$(readlink /root/.proto/bin/dune)" "${dune_path}" "dune bin symlink target"

for path in \
  "${opam_path}" \
  "${ocaml_path}" \
  "${ocamlc_path}" \
  "${ocamlopt_path}" \
  "${ocamldep_path}" \
  "${dune_path}" \
  "${bin_path}" \
  "${shim_path}"
do
  test -x "${path}"
done

echo "verifying shim registry"
grep -F '"ocaml"' /root/.proto/shims/registry.json >/dev/null
grep -F '"opam"' /root/.proto/shims/registry.json >/dev/null
grep -F '"ocamlc"' /root/.proto/shims/registry.json >/dev/null
grep -F '"ocamlopt"' /root/.proto/shims/registry.json >/dev/null
grep -F '"ocamldep"' /root/.proto/shims/registry.json >/dev/null
grep -F '"dune"' /root/.proto/shims/registry.json >/dev/null

echo "verifying command execution through proto exec"
assert_contains "$(exec_in_tool 'command -v opam')" "/root/.proto/" "opam resolves within proto-managed paths"
assert_contains "$(exec_in_tool 'command -v ocaml')" "/root/.proto/" "ocaml resolves within proto-managed paths"
assert_contains "$(exec_in_tool 'command -v ocamlc')" "/root/.proto/" "ocamlc resolves within proto-managed paths"
assert_contains "$(exec_in_tool 'command -v ocamlopt')" "/root/.proto/" "ocamlopt resolves within proto-managed paths"
assert_contains "$(exec_in_tool 'command -v ocamldep')" "/root/.proto/" "ocamldep resolves within proto-managed paths"
assert_contains "$(exec_in_tool 'command -v dune')" "/root/.proto/" "dune resolves within proto-managed paths"
assert_eq "$(exec_in_tool 'opam --version')" "2.5.0" "opam reports expected version"
assert_eq "$(exec_in_tool 'ocaml -version')" "The OCaml toplevel, version ${version}" "ocaml reports expected version"
assert_eq "$(exec_in_tool 'ocamlc -version')" "${version}" "ocamlc reports expected version"
assert_eq "$(exec_in_tool 'ocamlopt -version')" "${version}" "ocamlopt reports expected version"
assert_eq "$(exec_in_tool 'ocamldep -version')" "ocamldep, version ${version}" "ocamldep reports expected version"
dune_version="$(exec_in_tool 'dune --version')"
[[ "${dune_version}" =~ ^[0-9]+(\.[0-9]+)*$ ]] || {
  echo "assertion failed: dune reports a semantic version" >&2
  echo "actual: ${dune_version}" >&2
  exit 1
}

echo "verifying detection through proto run"
"${proto_bin}" run ocaml -- -version | grep -F "The OCaml toplevel, version ${version}"
"${proto_bin}" run --exe dune ocaml -- --version | grep -E '^[0-9]+'

echo "docker e2e passed"
