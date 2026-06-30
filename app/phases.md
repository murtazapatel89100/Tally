# Tally — Phased Build Plan

A step-by-step roadmap so you can build and learn one slice at a time. Each phase
is independently runnable and ends with a concrete **"done when"** check. Crates
are introduced only when a phase actually needs them (see **[`stack.md`](stack.md)**).

Legend: 🎯 goal · 🧱 build · 📦 new crates · ✅ done when

---

## Phase 0 — Scaffold *(complete)*

- 🎯 Project skeleton and docs.
- 🧱 `tally/` with `app/` + `web/`, `README.md`, `prd.md`, `stack.md`, `phases.md`.
- ✅ Folder structure exists; PRD reads as a complete spec.

---

## Phase 1 — Workspace & "hello journal"

- 🎯 A compiling Cargo workspace and the data model, with a sample journal in the repo.
- 🧱
  - `app/Cargo.toml` workspace: members `core` (`tally-core` lib) + `tally` (binary).
  - Edition 2024. Set up `.gitignore`, basic `cargo fmt`/`clippy` config.
  - `tally-core::model`: `Commodity`, `Amount`, `Account`, `Posting`, `Transaction`, `Status`.
  - A hand-written `examples/sample.journal` fixture to test against.
- 📦 `rust_decimal`, `jiff`, `indexmap`, `thiserror`.
- ✅ `cargo build` succeeds; `cargo test` runs a model unit test (e.g. an `Amount` adds correctly).

---

## Phase 2 — Parser (the heart)

- 🎯 Turn journal text into the model, with friendly errors.
- 🧱
  - `tally-core::parser` (winnow): transactions, postings, amounts (prefixed/suffixed
    commodities, thousands separators, negatives), comments, inline tags, **amount
    inference** for one blank posting, and directives (`include`, `alias`, `account`).
  - `tally-core::journal`: assemble parsed entries into a `Journal` with the account index.
  - miette diagnostics: errors show file, line/column, and a caret.
- 📦 `winnow`, `miette`.
- ✅ `sample.journal` parses; unit tests cover amount inference & each syntax form;
  a deliberately broken file produces a readable caret error.

---

## Phase 3 — Reports & non-interactive CLI

- 🎯 Useful output from the terminal — no TUI yet.
- 🧱
  - `tally-core::report`: balance tree (aggregated subtotals/totals) and register
    (chronological postings + running total). `tally-core::query` filters by
    account / date range / payee / tag.
  - `tally` binary: clap subcommands `bal`, `reg`, `accounts`, `print`; `-f/--file`
    with `$TALLY_FILE` / `$LEDGER_FILE` fallback; `color-eyre` for top-level errors.
  - `print` re-serializes canonically (round-trip).
- 📦 `clap`, `clap_complete`, `color-eyre`, `tracing(+subscriber, +appender)`.
- ✅ `cargo run -p tally -- bal -f examples/sample.journal` matches a known fixture;
  snapshot tests (`insta`) lock report output; `parse(print(x)) == x` holds.

---

## Phase 4 — TUI: explore

- 🎯 Browse the data interactively.
- 🧱
  - `tally/src/tui`: app state + synchronous event loop (`ratatui::run()` + crossterm).
  - Views: **Register**, **Balances** (collapsible tree), **Accounts**.
  - Global incremental search/filter; vim-ish keys; help overlay; view switching.
  - `color-eyre` panic hook restores the terminal on crash.
- 📦 `ratatui`, `crossterm`.
- ✅ `cargo run -p tally` opens the sample journal; you can navigate, search, and
  drill from an account into its register.

---

## Phase 5 — TUI: entry & edit

- 🎯 Add/edit transactions without leaving the app; write back to the file.
- 🧱
  - Modal Entry/Edit form: date, payee, status, postings (`tui-textarea` fields).
  - **Fuzzy account autocomplete** (`nucleo-matcher`) + live "balances to zero" check.
  - On save: serialize and append/update the journal file, then re-parse.
  - Optional: `notify` to live-reload on external edits.
- 📦 `tui-textarea`, `nucleo-matcher`, (optional `notify`).
- ✅ A transaction entered in the TUI is written to the journal, re-parses cleanly,
  and an unbalanced/invalid entry can't be saved.

---

## Phase 6 — Charts, budgets & config

- 🎯 The "modern, visual" payoff.
- 🧱
  - **Dashboard**: net-worth `Sparkline` over time, expense-by-category `BarChart`,
    budget `Gauge`s.
  - `tally.toml` via `serde`/`toml`/`directories`: theme, default journal, budgets.
  - Theme system (dark/light) wired to config.
- 📦 `serde`, `toml`, `directories`, (`strum`/`itertools` as helpful).
- ✅ Dashboard renders charts from the sample journal; budgets defined in
  `tally.toml` show progress; theme switches.

---

## Phase 7 — Web (docs + landing)

- 🎯 A site to explain and install Tally.
- 🧱 `web/` with **Astro + Starlight**: landing page (features, install, screenshots)
  + docs (format spec, command reference, keybindings) sourced from the PRD.
- ✅ `npm run build` produces a static site; landing + the format-spec page render.

---

## Phase 8 — Release & hardening

- 🎯 Ship it.
- 🧱 `proptest` invariants, `criterion` benchmarks on a large journal, CI
  (fmt/clippy/test), versioned release binaries, install instructions.
- ✅ Tagged release with a single self-contained binary; CI green.

---

## Suggested build order

`P1 → P2 → P3` gives you a working ledger-compatible CLI fast (and is the most
valuable, lowest-risk core). `P4 → P5 → P6` layer the TUI experience on top.
`P7 → P8` are presentation and release. Each arrow is a natural stopping point.
