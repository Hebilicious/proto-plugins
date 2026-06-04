---
name: proto-plugin-authoring
description: Use when adding or updating a proto tool plugin in Hebilicious/proto-plugins, including WASM validation, release-plz release flow, and the follow-up moonrepo/proto registry PR.
---

# Proto Plugin Authoring

Use for `proto-plugins` work and any follow-up `moonrepo/proto` registry change.

## Hard rules

- Default to a `branch/` feature branch and PR unless the user explicitly instructs otherwise.
- Plugins must build as WASM and have Rust tests plus Docker e2e coverage.
- Do not create tags or GitHub releases manually. Release only through the repo's `release-plz` workflows.
- A human must review the `proto-plugins` PR before merge, release, or publish.
- Open the `moonrepo/proto` PR only after the `proto-plugins` change is validated, merged, released, and tested from the released locator.

## Add a Plugin

1. Copy the closest existing plugin under `plugins/<name>` and keep the repo patterns.
2. Update `Cargo.toml`, `.moon/workspace.yml`, `release-plz.toml`, `README.md`, and `.github/workflows/ci.yml` e2e matrix as needed.
3. Add public contract tests and `plugins/<name>/tests/e2e/{Dockerfile,run.sh,verify.sh}`.
4. Keep `.prototools` pinned to explicit versions when touching release/toolchain automation, and ensure Rust has `wasm32-wasip1`.

## Validate Before PR

Run and report results:

```shell
cargo test --workspace
cargo build --workspace --target wasm32-wasip1
moon run proto-plugins:fmt proto-plugins:test proto-plugins:build
moon run <plugin>:e2e
```

Open a clean `proto-plugins` PR with only plugin/release-support changes. Wait for CI and human review before merge. Merge with squash only.

## Release Gate

After merge to `main`, let `release-plz` open/update the release PR. Merge that release PR only after CI and human review. Confirm the GitHub release contains the `.wasm` and `.wasm.sha256` assets. Test installing/using the released plugin locator before touching `moonrepo/proto`.

## Upstream Registry PR

Only after the release gate passes, open a separate clean PR in `moonrepo/proto` that changes only the relevant plugin registry entry. Keep the description maintainer-friendly and minimal:

```markdown
Updates the <tool> plugin locator to the released `Hebilicious/proto-plugins` monorepo artifact.

Validated in `Hebilicious/proto-plugins` via CI, release-plz release, and install test from the released locator.
```
