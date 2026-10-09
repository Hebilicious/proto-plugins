# Nushell plugin

[Nushell](https://www.nushell.sh/) TOML plugin for [proto](https://moonrepo.dev/proto).

This plugin installs the official prebuilt Nushell release archives from
[`nushell/nushell`](https://github.com/nushell/nushell/releases). It uses the
proto TOML plugin format v2 and requires proto 0.63.0 or newer.

## Installation

Add the following to `.prototools`:

```toml
[plugins.tools]
nu = "https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/nu/nu.toml"

[tools.nu]
version = "0.112.2"
```

Or add it explicitly:

```shell
proto plugin add nu https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/nu/nu.toml
```

## Usage

```shell
# install latest stable release
proto install nu

# install a specific version
proto install nu 0.112.2

# run Nushell
proto run nu -- --version
proto run nu -- -c 'version | get version'
```

## Version Detection

The plugin checks version files in this order:

1. `.nu-version`
2. `.nushell-version`

Supported formats:

```text
0.112.2
stable
```

## Supported Platforms

- Linux x64, glibc and musl
- Linux arm64, glibc and musl
- macOS x64
- macOS arm64
- Windows x64
- Windows arm64

## Notes

- Nushell release archives include several `nu_plugin_*` binaries. The plugin
  exposes these binaries through proto alongside the primary `nu` executable.
- Windows installs use the release `.zip` archive, not the `.msi` installer.
- Downloads are verified against the release `SHA256SUMS` file.

## Contributing

```shell
moon run nu:e2e
```
