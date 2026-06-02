# proto-hk

HK WASM plugin for [proto](https://moonrepo.dev/proto).

## Installation

```toml
[plugins]
hk = "github://hebilicious/proto-plugins/hk"

[tools.hk]
version = "1.46.0"
```

Or add it explicitly:

```shell
proto plugin add hk github://hebilicious/proto-plugins/hk
proto install hk
```

## Version Files

The plugin detects `.hk-version`.

```text
1.46.0
```

Aliases supported by proto include `latest` and `stable`.
