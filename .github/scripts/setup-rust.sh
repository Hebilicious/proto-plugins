#!/usr/bin/env bash
set -euo pipefail

rust_version="$(awk -F'"' '/^rust[[:space:]]*=/{ print $2; exit }' .prototools)"

if [[ -z "${rust_version}" ]]; then
  echo "Unable to find a rust version in .prototools." >&2
  exit 1
fi

rustup toolchain install "${rust_version}" --profile default --component rustfmt --target wasm32-wasip1
rustup default "${rust_version}"
rustup component add rustfmt --toolchain "${rust_version}"
rustup target add wasm32-wasip1 --toolchain "${rust_version}"

rustc --version
cargo --version
