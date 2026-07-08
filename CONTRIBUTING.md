# Contributing to Tally

Thanks for your interest in improving Tally! This document explains how to get
set up, the checks your change needs to pass, and how to get it merged.

By participating you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

- **Report a bug** — open an issue with the [Bug report](https://github.com/murtazapatel89100/Tally/issues/new?template=bug_report.yml) template.
- **Request a feature** — check the [ROADMAP](ROADMAP.md) first, then open a [Feature request](https://github.com/murtazapatel89100/Tally/issues/new?template=feature_request.yml).
- **Send a pull request** — bug fixes, docs, and small features are welcome. For large
  changes, please open an issue to discuss the design first so we don't waste your time.

## Repository layout

| Path | Description |
|------|-------------|
| `app/` | Rust workspace root — `tally-core` library + `tally` binary |
| `app/core/` | `tally-core`: pure domain logic (parser, model, reports, printer). **No TUI/terminal dependencies.** |
| `app/tally/` | `tally`: the CLI + ratatui TUI |
| `app/examples/` | Sample journal (`sample.journal`) and config (`tally.toml`) |
| `web/docs/` | Astro + Starlight documentation & landing site (https://tally.rs) |
| `web/app/` | Web GUI version of the CLI (planned, WASM-powered) |
| `.github/workflows/` | CI (fmt, clippy, tests, proptest, docs build) + release pipeline |

## Prerequisites

- **Rust** (stable, edition 2024 — MSRV **1.85**). Install via [rustup](https://rustup.rs/).
  Make sure the `rustfmt` and `clippy` components are present:
  ```sh
  rustup component add rustfmt clippy
  ```
- **Node 20 + [pnpm](https://pnpm.io/)** — only needed if you touch the `web/docs` site.

The workspace manifest lives at `app/Cargo.toml`, so every cargo command below passes
`--manifest-path app/Cargo.toml` (or you can `cd app` first and drop the flag).

## Build & run

```sh
# Build everything
cargo build --manifest-path app/Cargo.toml

# Run the TUI against the bundled sample journal
cargo run --manifest-path app/Cargo.toml -p tally -- -f app/examples/sample.journal

# Run a non-interactive report
cargo run --manifest-path app/Cargo.toml -p tally -- -f app/examples/sample.journal bal
```

## Checks to run before pushing

CI runs the following — run them locally first so your PR goes green on the first try.
These mirror [`.github/workflows/ci.yml`](.github/workflows/ci.yml):

```sh
# 1. Formatting (config in app/rustfmt.toml)
cargo fmt --manifest-path app/Cargo.toml --all --check

# 2. Lints — clippy must be clean
cargo clippy --manifest-path app/Cargo.toml --all-targets --all-features

# 3. Tests across the workspace
cargo test --manifest-path app/Cargo.toml --all

# 4. Property tests (tally-core invariants)
cargo test --manifest-path app/Cargo.toml -p tally-core proptest

# 5. Docs build (only if you changed web/docs)
cd web/docs && pnpm install --frozen-lockfile && pnpm run build
```

### Git pre-commit hook

The repo ships a versioned pre-commit hook (in [`.githooks/`](.githooks/)) that runs the
same clippy gate as CI and **blocks the commit if lint fails** — so nothing broken gets
committed or pushed. Enable it once per clone:

```sh
git config core.hooksPath .githooks
```

Bypass it in an emergency with `git commit --no-verify`.

## Coding conventions

- **Safety:** `unsafe_code` is `forbid`den workspace-wide. Don't introduce it.
- **No debugging leftovers:** `dbg!` and `todo!` are lint warnings — remove them before committing.
- **Keep the core pure:** `tally-core` must not depend on `ratatui`, `crossterm`, or any
  terminal/UI crate. Domain logic goes in `core`; presentation goes in `tally`.
- **Formatting:** governed by `app/rustfmt.toml` (max width 100, crate-granularity imports,
  `StdExternalCrate` grouping). Just run `cargo fmt`.
- **Snapshot tests** use [`insta`](https://insta.rs/). If a change intentionally alters
  report output, review and accept the new snapshots:
  ```sh
  cargo insta review
  ```
- **Property tests** use [`proptest`](https://proptest-rs.github.io/proptest/). Add
  invariants for new parser/report behavior in `app/core/tests/proptest_invariants.rs`.

## Commit & PR conventions

- Follow [Conventional Commits](https://www.conventionalcommits.org/) — matching the
  existing history, e.g. `feat(tui): ...`, `fix(parser): ...`, `docs: ...`, `chore: ...`.
- Branch off `main`; keep PRs focused on a single concern.
- Update [`CHANGELOG.md`](CHANGELOG.md) under **[Unreleased]** for any user-facing change.
- Fill in the pull request template checklist. A PR is ready to review when CI is green.

## Proposing large features

The [ROADMAP](ROADMAP.md) tracks planned work. If you want to tackle a roadmap item or
propose something new and substantial, open an issue describing the design first — it's
the fastest path to an approved, mergeable change.

Thanks again for contributing! 💚
