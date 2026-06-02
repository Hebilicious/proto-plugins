# Repository Instructions

## Branches and PRs

- Do not push directly to `main`.
- Create all changes on a feature branch using the `branch/` prefix.
- Open a pull request for every change and wait for CI to pass before merging.
- Merge pull requests with squash merge only.

## Releases

- Do not create tags or GitHub releases manually.
- Use the existing `release-plz` workflow:
  - merge feature PRs into `main`
  - let `release-plz` open or update the automated release PR
  - merge the release PR to publish the next release

## Validation

- Run `cargo test --workspace` for Rust/unit and integration coverage.
- Run `cargo build --workspace --target wasm32-wasip1` before sending changes for review.
- Run `moon run proto-plugins:fmt proto-plugins:test proto-plugins:build` for the monorepo task graph.
- When changing release automation, keep `rust-toolchain.toml` pinned to an explicit Rust version so release builds continue to select `wasm32-wasip1`.
