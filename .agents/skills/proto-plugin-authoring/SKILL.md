---
name: proto-plugin-authoring
description: Use when adding or updating a proto tool plugin in Hebilicious/proto-plugins, including TOML/WASM validation, release-plz release flow, and the follow-up moonrepo/proto registry PR.
---

# Proto Plugin Authoring

Use for `proto-plugins` work and any follow-up `moonrepo/proto` registry change.

## Hard rules

- Default to a `branch/` feature branch and PR unless the user explicitly instructs otherwise.
- Prefer a proto TOML plugin (format v2, `format = "2"`) when the tool only needs prebuilt downloads. Use a Rust WASM plugin only when custom logic is required (for example running commands during install).
- TOML plugins need Docker e2e coverage. WASM plugins must build as WASM and have Rust tests plus Docker e2e coverage.
- Do not create tags or GitHub releases manually. Release only through the repo's `release-plz` workflows.
- A human must review the `proto-plugins` PR before merge, release, or publish.
- Open the `moonrepo/proto` PR only after the `proto-plugins` change is validated, merged, released, and tested from the released locator.

## Add a Plugin

1. Copy the closest existing plugin under `plugins/<name>` and keep the repo patterns (`hk`/`nu` for TOML, `ocaml` for WASM).
2. Update `.moon/workspace.yml`, `README.md`, and the `.github/workflows/ci.yml` e2e matrix. For WASM plugins also update `Cargo.toml` workspace members and `release-plz.toml`.
3. Add `plugins/<name>/tests/e2e/{Dockerfile,run.sh,verify.sh}`, plus public contract tests for WASM plugins.
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

TOML plugins have no release step; they are served from `main` by raw URL. Test the raw `main` URL after merge before touching `moonrepo/proto`.

For WASM plugins, after merge to `main`, let `release-plz` open/update the release PR. Merge that release PR only after CI and human review. Confirm the GitHub release contains the `.wasm` and `.wasm.sha256` assets. Test installing/using the released plugin locator before touching `moonrepo/proto`.

## Upstream Registry PR

Only after the release gate passes, open a separate clean PR in `moonrepo/proto` that changes only the relevant plugin registry entry. Keep the description maintainer-friendly and minimal:

```markdown
Updates the <tool> plugin locator to the released `Hebilicious/proto-plugins` monorepo artifact.

Validated in `Hebilicious/proto-plugins` via CI, release-plz release, and install test from the released locator.
```
