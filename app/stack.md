# Tally — Tech Stack & Crate Selection

Why each dependency was chosen. Everything here is picked to be **modern *and*
production-proven**. Rust **edition 2024**. The project is deliberately
**synchronous** — no `tokio`/async runtime — because a thread-based input loop is
the ratatui norm and keeps the code approachable.

> Versions are indicative; Cargo resolves the latest compatible release and
> `Cargo.lock` pins them so builds are reproducible.

---

## Terminal UI

| Crate | ~Ver | Why it's the right choice |
|-------|------|---------------------------|
| **ratatui** | 0.30 | The de-facto modern Rust TUI framework (maintained successor to tui-rs). 0.30 added the simplified `ratatui::run()` setup and ships `BarChart`, `Chart`, `Sparkline`, `Gauge`, `Table`, `List` — exactly the widgets the Dashboard / Register / Balances views need. Large community, excellent docs. |
| **crossterm** | 0.28 | ratatui's default cross-platform backend (Linux/macOS/Windows). Handles raw mode, key/mouse events, colors. Re-exported by ratatui so versions stay aligned. |
| **tui-textarea** | 0.7 | Drop-in multi-line text-editor widget for ratatui. Powers the Entry/Edit form fields with cursor, selection, and editing built in — no need to write an editor from scratch. |

## CLI

| Crate | ~Ver | Why |
|-------|------|-----|
| **clap** | 4 | The standard argument parser. We use the **derive** API: subcommands (`bal`, `reg`, …) and flags become annotated structs — type-safe, auto `--help`, minimal boilerplate. |
| **clap_complete** | 4 | Generates shell completions (bash/zsh/fish) from the same clap definition — expected polish for a serious CLI. |

## Domain core (money, dates, accounts)

| Crate | ~Ver | Why |
|-------|------|-----|
| **rust_decimal** + **rust_decimal_macros** | 1 | **Never use `f64` for money.** Fixed-precision 128-bit decimal with exact arithmetic, serde support, and a `dec!()` literal macro. Correct rounding for accounting. |
| **jiff** | 0.2 | The modern, correctness-first datetime library (by the author of `regex`/`ripgrep`). Best-in-class calendar handling; `jiff::civil::Date` models posting dates exactly. *Caveat: pre-1.0, so minor API churn is possible — `Cargo.lock` insulates us. If you'd rather have a 1.0-stable option, `time` 0.3 is the conservative swap.* |
| **indexmap** | 2 | Hash map that preserves **insertion order** — used for the account index so accounts/commodities report in first-seen order (matches ledger) without re-sorting. |

## Parsing & diagnostics

| Crate | ~Ver | Why |
|-------|------|-----|
| **winnow** | 0.6 | Modern parser-combinator library (maintained fork of nom by the clap/toml maintainer). Fast, clean ergonomics, ideal for the line-oriented ledger grammar. Combinators keep the parser readable and testable vs. a hand-rolled state machine. |
| **miette** | 7 | Turns parse failures into **friendly diagnostics** — file, line/column, and a caret underlining the bad token (the "never a bare parse error" goal). Pairs with `thiserror` for rich, source-spanned errors. |

## Fuzzy matching

| Crate | ~Ver | Why |
|-------|------|-----|
| **nucleo-matcher** | 0.3 | The fuzzy matcher from the Helix editor — ~6× faster than `fuzzy-matcher`/skim, superior Unicode handling, battle-tested. Powers fast account autocomplete in the Entry form and search ranking. |

## Config, paths, errors, logging

| Crate | ~Ver | Why |
|-------|------|-----|
| **serde** (+ derive) | 1 | The serialization standard; derive `Deserialize` for `tally.toml`. |
| **toml** | 0.8 | Parses `tally.toml` (theme, budgets, default journal path) into serde structs. |
| **directories** | 5 | Resolves OS-correct config/data paths (XDG on Linux, the right spots on macOS/Windows) so `tally.toml` is found in the conventional location. |
| **thiserror** | 2 | Ergonomic derived error enums for the **library** (`tally-core`). Pairs with miette for diagnostics. |
| **color-eyre** | 0.6 | Rich error/panic reports for the **binary**, with a panic hook that restores the terminal on crash — critical for a TUI, otherwise a panic leaves the terminal garbled. |
| **tracing** + **tracing-subscriber** + **tracing-appender** | — | Structured logging to a **file** (stdout belongs to the TUI). Invaluable for debugging the event loop and parser without disrupting the UI. |

## Optional / later phases

| Crate | Phase | Why |
|-------|-------|-----|
| **notify** | P5+ | Watch the journal file and live-reload when it changes on disk (e.g. edited in another editor). |
| **itertools**, **strum** | any | Quality-of-life: richer iterators; deriving `Display`/`FromStr` for view & status enums. |

## Dev-dependencies (testing & quality)

| Crate | Why |
|-------|-----|
| **insta** | Snapshot tests for report output and `print` round-tripping — review diffs instead of brittle string asserts. |
| **rstest** | Table-driven / fixture-based parametrized tests for the parser's many input cases. |
| **assert_cmd** + **predicates** | Black-box integration tests that run the real `tally` binary and assert on its output. |
| **proptest** | Property tests for invariants: every parsed transaction balances to zero; `parse(print(x)) == x`. |
| **criterion** | Benchmarks to keep parsing/reporting fast on large journals. |

---

## Decisions worth remembering (for a Rust newcomer)

- **Decimal, not float, for money** — floating point can't represent `0.10`
  exactly; accounting must be exact. `rust_decimal` is non-negotiable here.
- **Workspace split** (`tally-core` vs `tally` binary) keeps domain logic free of
  terminal dependencies, so it's unit-testable and reusable (future WASM web app).
- **Library uses `thiserror`, binary uses `color-eyre`** — a common idiom: typed
  recoverable errors in the lib, human-friendly reporting at the top level.
- **Synchronous by choice** — async adds real complexity; we don't need it.
- See **[`phases.md`](phases.md)** for which crates get introduced in which build phase.
