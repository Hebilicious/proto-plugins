# proto-hk

HK TOML plugin for [proto](https://moonrepo.dev/proto).

This plugin uses the proto TOML plugin format v2 and requires proto 0.63.0 or newer.

## Installation

```toml
[plugins.tools]
hk = "https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/hk/hk.toml"

[tools.hk]
version = "1.46.0"
```

Or add it explicitly:

```shell
proto plugin add hk https://raw.githubusercontent.com/Hebilicious/proto-plugins/main/plugins/hk/hk.toml
proto install hk
```

## Version Files

The plugin detects `.hk-version`.

```text
1.46.0
```

Aliases supported by proto include `latest` and `stable`.

## Contributing

```shell
moon run hk:e2e
```
