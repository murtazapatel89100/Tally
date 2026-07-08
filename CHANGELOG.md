# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

_Nothing yet. See the [ROADMAP](ROADMAP.md) for planned work._

## [0.1.0]

Initial release. Tally is a plain-text, double-entry accounting tool for the
terminal that reads standard `ledger` / `hledger` journal files.

### Added

- **Data model & workspace** — `tally-core` library (pure domain logic, no TUI
  dependencies) and the `tally` binary, sharing a Cargo workspace.
- **Journal parser** — reads ledger/hledger `.journal` files (transactions,
  postings, `account`/`commodity`/`alias`/`include` directives, tags, comments,
  `*` cleared / `!` pending statuses) with friendly `miette` error diagnostics.
- **Reports & CLI** — non-interactive commands:
  - `bal` — hierarchical account balances (with account/date filters).
  - `reg` — posting register with running totals.
  - `accounts` — list all known accounts.
  - `print` — canonical, ledger-compatible re-serialization.
  - `completions <shell>` — bash/zsh/fish shell completions.
  - `-f/--file` flag plus `TALLY_FILE` / `LEDGER_FILE` environment variables.
- **Interactive TUI** — Dashboard, Balances (collapsible tree), Register, and
  Accounts views with vim-style navigation, filtering, mouse and page scrolling.
- **In-TUI transaction entry & edit** — modal form with fuzzy account
  autocomplete, live balance checking, and auto-balancing of a single blank posting.
- **Dashboard, budgets & config** — net-worth sparkline, top-expense bar charts,
  and monthly budget gauges driven by `tally.toml`; three built-in themes
  (Tokyo Night / dark, Nord, light).
- **Documentation & landing site** — Astro + Starlight docs at https://tally.rs.
- **Release & hardening** — cross-platform CI (fmt, clippy, tests, property tests),
  property-based invariant tests, benchmarks, and a tagged multi-target release pipeline.

[Unreleased]: https://github.com/murtazapatel89100/Tally/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/murtazapatel89100/Tally/releases/tag/v0.1.0
