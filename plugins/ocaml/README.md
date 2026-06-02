# OCaml plugin

[OCaml](https://ocaml.org/) WASM plugin for [proto](https://moonrepo.dev/proto).

This plugin installs a realistic OCaml baseline:

- `opam`
- `ocaml-base-compiler`
- `dune`

It manages an isolated opam root inside each proto installation and activates the
switch environment automatically when invoked through proto.

## Installation

Add the following to `.prototools`:

```toml
[plugins]
ocaml = "github://hebilicious/proto-plugins/ocaml"

[tools.ocaml]
version = "5.4.1"
```

Or add it explicitly:

```shell
proto plugin add ocaml github://hebilicious/proto-plugins/ocaml
```

## Usage

```shell
# install latest stable compiler
proto install ocaml

# install a specific compiler version
proto install ocaml 5.4.1

# run tools from the managed switch
proto run ocaml -- ocaml -version
proto run ocaml -- dune --version
```

## Version Detection

The plugin checks version files in this order:

1. `.ocaml-version`
2. `dune-project`

Supported `.ocaml-version` formats:

```text
5.4.1
5.4
ocaml-base-compiler.5.4.1
stable
```

The `dune-project` parser supports explicit compiler constraints in package
dependency stanzas, for example:

```lisp
(package
 (name app)
 (depends
  (ocaml (>= 5.1) (< 5.5))
  dune))
```

## Supported Platforms

- Linux x64
- Linux arm64
- macOS x64
- macOS arm64
- Windows x64

## Notes

- The plugin installs `dune` by default.
- It does not install additional tools such as `ocaml-lsp-server`,
  `ocamlformat`, `utop`, or `odoc`.
- `*.opam` auto-detection is intentionally not implemented because current proto
  plugin version detection only supports fixed filenames.

## Contributing

```shell
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1
cargo test
moon run ocaml:e2e
```

## Releases

This plugin is released from
[`Hebilicious/proto-plugins`](https://github.com/Hebilicious/proto-plugins)
with [`release-plz`](https://release-plz.dev/).

The monorepo tag for this plugin is `ocaml-v{{version}}`.
