# proto-plugins

WASM plugins for [proto](https://moonrepo.dev/proto), managed as a Rust monorepo with [moon](https://moonrepo.dev/moon).

## Plugins

| Plugin | Package | Locator |
| --- | --- | --- |
| [HK](plugins/hk) | `hk` | `github://hebilicious/proto-plugins/hk` |
| [Nushell](plugins/nu) | `nu` | `github://hebilicious/proto-plugins/nu` |
| [OCaml](plugins/ocaml) | `ocaml` | `github://hebilicious/proto-plugins/ocaml` |

## Installation

Add one or more plugins to `.prototools`:

```toml
[plugins]
hk = "github://hebilicious/proto-plugins/hk"
nu = "github://hebilicious/proto-plugins/nu"
ocaml = "github://hebilicious/proto-plugins/ocaml"

[tools.hk]
version = "1.46.0"

[tools.nu]
version = "0.112.2"

[tools.ocaml]
version = "5.4.1"
```

Or add them explicitly:

```shell
proto plugin add hk github://hebilicious/proto-plugins/hk
proto plugin add nu github://hebilicious/proto-plugins/nu
proto plugin add ocaml github://hebilicious/proto-plugins/ocaml
```

## Development

```shell
proto install
moon run proto-plugins:fmt proto-plugins:test proto-plugins:build
```

Run a plugin-specific Docker e2e test:

```shell
moon run hk:e2e
moon run nu:e2e
moon run ocaml:e2e
```

## Hooks

This repository includes [`hk`](https://hk.jdx.dev/getting_started.html) configuration in `hk.pkl`.

```shell
cargo install hk
hk install --global
```

After the global install, `hk` is a no-op outside repositories that contain an `hk.pkl`.

## Releases

Releases are handled by [`release-plz`](https://release-plz.dev/).

Each plugin is versioned independently and uses monorepo tags matching proto's GitHub locator rules:

- `nu-vX.Y.Z`
- `ocaml-vX.Y.Z`
- `hk-vX.Y.Z`

Merging normal changes into `main` opens or updates the release PR. Merging the release PR creates the package tags, builds the matching WASM plugin, attaches the `.wasm` and `.sha256` assets to the GitHub release, and leaves Cargo publishing disabled.
