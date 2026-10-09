# proto-plugins

Plugins for [proto](https://moonrepo.dev/proto), managed as a monorepo with [moon](https://moonrepo.dev/moon).

Plugins that only download prebuilt releases use the proto TOML plugin format v2 (proto 0.63.0+). Plugins that need custom install logic are Rust WASM plugins.

## Plugins

| Plugin | Kind | Locator |
| --- | --- | --- |
| [HK](plugins/hk) | TOML | `https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/hk/hk.toml` |
| [Nushell](plugins/nu) | TOML | `https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/nu/nu.toml` |
| [OCaml](plugins/ocaml) | WASM | `github://hebilicious/proto-plugins/ocaml` |

## Installation

Add one or more plugins to `.prototools`:

```toml
[plugins.tools]
hk = "https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/hk/hk.toml"
nu = "https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/nu/nu.toml"
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
proto plugin add hk https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/hk/hk.toml
proto plugin add nu https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/nu/nu.toml
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

TOML plugins are not released: proto loads them straight from `main` via their raw URL, so changes ship on merge.

WASM plugins are released by [`release-plz`](https://release-plz.dev/). Each is versioned independently and uses monorepo tags matching proto's GitHub locator rules:

- `ocaml-vX.Y.Z`

Merging normal changes into `main` opens or updates the release PR when package versions need to change. The release workflow publishes any package version that does not have a matching monorepo tag, builds the matching WASM plugin, attaches the `.wasm` and `.sha256` assets to the GitHub release, and leaves Cargo publishing disabled.
