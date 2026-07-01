# App — Structure Guide (Phase 2: the parser)

This is for someone coming from Python, Go, or TypeScript who wants to understand
how this Rust project is laid out and why — including what actually breaks when
you change things.

Phase 2 is where Tally grew its **parser**: the code that turns a plain-text
journal file into typed data. This guide centers on that subsystem — the three
new modules (`error.rs`, `parser.rs`, `journal.rs`) and how a line of text
becomes a `Transaction`.

---

## Rust concepts you need first

### What is a crate?

A **crate** is Rust's word for a package — the same as:
- a `package` in Python (a folder with `__init__.py`)
- a `module` in Go (a folder with `go.mod`)
- a `package` in Node.js (a folder with `package.json`)

Every crate has a `Cargo.toml` at its root (Rust's equivalent of `package.json`
or `go.mod`). A crate is either a **library** (code other crates import) or a
**binary** (a runnable program).

### What is a workspace?

A **workspace** is a collection of crates that share one `Cargo.lock` and one
`target/` build output folder. Think of it like a monorepo.

In Go terms: one `go.work` file linking multiple modules.
In JS terms: an npm/yarn workspace.

This project has **one workspace** (`app/Cargo.toml`) containing **two crates**:
- `core/` — a library crate (no `main`, can't be run directly)
- `tally/` — a binary crate (has `main.rs`, produces the `tally` executable)

The parser lives entirely in `core/`, so it has **zero terminal/UI
dependencies** — it can be unit-tested without a terminal and could later be
compiled to WASM for a web app.

### How do crates link to each other?

You declare a dependency in `Cargo.toml`, exactly like `package.json`'s
`"dependencies"`. The `tally` binary depends on `tally-core` like this:

```toml
# tally/Cargo.toml
[dependencies]
tally-core = { path = "../core" }
```

Then in Rust code you bring it in with:

```rust
use tally_core::journal::Journal;
//   ^^^^^^^^^^ crate name (hyphens become underscores in code)
```

> **Gotcha:** Cargo uses hyphens in crate names (`tally-core`), but Rust code
> uses underscores (`tally_core`). This is a universal Rust convention.

### What is `mod`?

Inside a crate, code is split into **modules** — Rust's equivalent of files/
subpackages. A module is declared with `mod parser;` which tells Rust to look for
either `parser.rs` or `parser/mod.rs`.

```
src/
├── lib.rs           ← crate root; declares: pub mod parser;
└── parser.rs        ← the actual module code
```

This is like Python's `from . import parser` or Go's implicit package discovery.

---

## Directory layout

```
app/                        ← workspace root
├── Cargo.toml              ← workspace manifest (links the two crates)
├── Cargo.lock              ← pinned dependency versions (commit this)
├── rustfmt.toml            ← code formatter config
├── examples/
│   └── sample.journal      ← test data file (not a Rust file)
├── core/                   ← "tally-core" library crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs          ← crate root
│   │   ├── model.rs        ← data types (what the parser produces)
│   │   ├── error.rs        ← friendly parse diagnostics (miette)   ← Phase 2
│   │   ├── parser.rs       ← journal text → model (winnow)         ← Phase 2
│   │   └── journal.rs      ← assembles entries into a Journal      ← Phase 2
│   └── tests/
│       └── sample_journal.rs  ← end-to-end test against the fixture ← Phase 2
└── tally/                  ← "tally" binary crate
    ├── Cargo.toml
    └── src/
        └── main.rs         ← entry point (CLI arrives in Phase 3)
```

---

## The parsing pipeline

Everything Phase 2 added is one pipeline: **text in, typed `Journal` out.**

```
 "2026-01-05 * Grocery\n    Expenses:Food  $12\n    Assets:Checking\n"
        │
        ▼  parser::parse(text, name)          ← parser.rs
   Vec<Entry>        (Entry = Transaction | Directive, amounts inferred)
        │
        ▼  Journal::build(entries)            ← journal.rs
   Journal { transactions, accounts, commodities, aliases, warnings }

   …and on any syntax error, a ParseError that draws a caret ← error.rs
```

Two layers, deliberately separated:

- **`parser.rs`** answers *"what does this text say?"* — pure syntax. It produces
  a flat list of `Entry` values and knows nothing about aliases or files.
- **`journal.rs`** answers *"what does it mean together?"* — it folds those
  entries into a `Journal`, resolving `alias` rewrites, expanding `include`
  files, and building the account/commodity indexes.

Keeping them apart means the tricky, whitespace-sensitive tokenizing is testable
in isolation, and the semantic assembly (which touches the filesystem for
`include`) stays out of the hot parsing path.

---

## File-by-file breakdown

### Dependencies added in Phase 2

Two crates were added to `core/Cargo.toml`:

```toml
winnow = "1"    # parser-combinator library — powers amount/date parsing
miette = { version = "7", features = ["fancy"] }  # rich, caret-drawing diagnostics
```

- **winnow** is a *parser-combinator* library: you build big parsers by composing
  tiny ones (`sign`, `number`, `commodity`) instead of writing a state machine by
  hand. It's used for "the heart" — parsing amounts and dates.
- **miette** turns an error type into a graphical diagnostic (file, line/column,
  and a caret underlining the bad token). The `fancy` feature enables the
  box-drawing renderer.

**Behavioral examples:**

- Remove `winnow` → every `use winnow::...` line in `parser.rs` fails to compile
  with "use of undeclared crate `winnow`".
- Drop the `fancy` feature from miette → the code still compiles, but printed
  errors lose the colored box-and-caret rendering and fall back to plain text.

---

### `app/core/src/lib.rs` — library crate root

Declares the crate's public modules:

```rust
pub mod model;    // data types
pub mod error;    // parse diagnostics
pub mod parser;   // journal text → model
pub mod journal;  // model → assembled Journal
```

It also carries a crate-level `#![allow(clippy::result_large_err)]`: `ParseError`
deliberately embeds the whole source file (so it can draw a caret), which makes
it larger than clippy's `result_large_err` threshold. Boxing every `Result` on
the rare error path would only hurt readability, so we opt out of that lint here.

**Behavioral examples:**

- Add `pub mod query;` without creating `query.rs` → compile error: "file not
  found for module `query`". Rust expects the file to exist the moment you declare
  the module.
- Remove the `#![allow(...)]` line → `cargo clippy` starts warning that the
  `Err` variant of `parse`'s return type is "very large". Harmless, but noisy.

---

### `app/core/src/model.rs` — the types the parser fills in

The parser doesn't invent its own output shapes; it populates the data types that
already existed: `Transaction`, `Posting`, `Amount`, `Commodity`, `Account`,
`Status`, and `SourceSpan`. The one field Phase 2 finally puts to use is
`Transaction.source_span` — the byte range the transaction occupied in the file,
which the parser records so later phases (edit/write-back) can find it again.

`Transaction::is_balanced()` remains the double-entry check the parser relies on:
after inference runs, a well-formed transaction sums to zero per commodity.

**Behavioral examples:**

- Make `Posting.amount` required (`Amount` instead of `Option<Amount>`) → amount
  inference in `parser.rs` breaks, because it represents a blank posting as
  `amount: None`. The compiler flags every site that assumed it could be absent.
- Add a `Status` variant (e.g. `Disputed`) → the `marker()` match and the header
  parser's `*`/`!` handling both need updating; the compiler lists each spot.

---

### `app/core/src/error.rs` — parse diagnostics

Defines `ParseError`, which derives both `thiserror::Error` (so it's a normal
Rust error) and `miette::Diagnostic` (so a reporter can draw a caret). It bundles
the failure message, the full source text, a byte-offset span, the caret label,
and an optional help hint. Printed by a miette-aware reporter, a bad amount looks
like:

```
  × invalid amount
   ╭─[sample.journal:2:22]
 2 │     Expenses:Food    $1..2
   ·                      ──┬──
   ·                        ╰── not a valid amount
   ╰────
  help: Amounts look like `$1,234.56`, `-$3`, or `10 AAPL`.
```

The `#[source_code]`, `#[label]`, and `#[help]` attributes are how the derive
macro learns which field is the text, which is the span, and which is the hint.

**Behavioral examples:**

- Change `#[label("{}", self.label)]` to a fixed `#[label("here")]` → every error
  shows `here` under the caret instead of the specific message the parser chose.
  Still compiles.
- Remove the `#[source_code]` field → the derive still compiles, but the reporter
  has no text to render, so you get the headline with no caret or code frame.

---

### `app/core/src/parser.rs` — the journal parser *(the heart)*

Turns journal text into a `Vec<Entry>` (each `Entry` is a `Transaction` or a
`Directive`). It works in two registers:

1. **Line-oriented outer loop** (plain Rust). Ledger's grammar is defined by
   *lines* and *indentation*, so the loop walks the file a line at a time and
   classifies each one: blank, comment (`;` `#` `%` `|` `*`), a directive, a
   transaction header (starts with a digit), or an indented posting.
2. **winnow token parsers** for the fiddly bits inside a line — dates and,
   above all, **amounts**. Small combinators (`sign`, `number`, `commodity`) are
   composed into `prefixed_amount` / `suffixed_amount`.

Public helpers `parse`, `parse_date`, and `parse_amount` are unit-tested directly.

Key rules it implements (all from the PRD's file-format section):

- **Account/amount split:** the account name ends at the first run of *2+ spaces*
  or a tab; everything after is the amount. That's why `Equity:Opening Balances`
  (a single space) stays one name.
- **Amount inference:** if exactly one posting in a transaction has no amount,
  it's filled in so the postings sum to zero (single-commodity case).
- **Amounts:** prefixed (`$45`, `-$3`, `$-3`), suffixed (`10 AAPL`), thousands
  separators (`$1,234.56`), and negatives in either position.
- **Tags:** `; key: value` and `; :flag1:flag2:` comment forms become structured
  tags on the transaction or posting; anything else stays a plain comment.
- **Diagnostics:** every error carries precise byte offsets. The parser computes
  them with a pointer trick (`offset_in`): because each token is a *subslice* of
  the original text, subtracting pointers gives its exact position for the caret.

**Behavioral examples:**

- Change `find_amount_sep` to split on a *single* space → `Equity:Opening
  Balances` now parses as account `Equity:Opening` with amount `Balances`, which
  fails amount parsing. The `keeps_single_space_account_names` test catches it.
- Add a variant to the `Directive` enum (e.g. `Year(i16)`) → `journal.rs`'s
  `apply_directive` match stops compiling until you handle the new case. The
  compiler points at the exact spot.
- Feed a file whose first line is indented → `parse` returns a `ParseError`
  ("posting outside of a transaction") instead of panicking.
- Widen `commodity` to also accept digits → `10 AAPL` would tokenize wrong,
  because the number parser and commodity parser would fight over the `10`.

---

### `app/core/src/journal.rs` — the assembler

Takes the flat `Vec<Entry>` from the parser and folds it into a `Journal`: the
transaction list plus first-seen-order indexes of accounts and commodities, the
alias table, and any non-fatal `warnings`. Two entry points:

- `Journal::parse_str(text)` — assemble from in-memory text. `include` directives
  can't be resolved (no base directory) and are recorded as warnings.
- `Journal::from_path(path)` — additionally follows `include` directives,
  reading each file relative to the including file's directory.

Aliases are applied to every account before it's indexed: an exact full-name
match wins, otherwise the longest matching leading path segment is rewritten
(`alias A = Assets` turns `A:Checking` into `Assets:Checking`).

**Behavioral examples:**

- Call `Journal::parse_str` on text containing `include foo.journal` → the
  include is recorded in `journal.warnings` rather than failing; `from_path`
  would actually read the file.
- Add an `alias Cash = Assets:Checking` line → every posting to `Cash` (or
  `Cash:...`) is rewritten before indexing.
- Change `accounts` from `IndexSet` to `HashSet` → still compiles, but accounts
  no longer report in first-seen order, breaking the ordering assertion in
  `indexes_accounts_and_commodities_in_order`.
- Point `from_path` at a missing file → you get a `JournalError::Io`, not a
  parse error — the two failure modes are distinct variants.

---

### `app/core/tests/sample_journal.rs` — integration test

Files under `tests/` are compiled as **separate crates** that can only use the
public API (like an external user would). This one loads
`examples/sample.journal` via `Journal::from_path` and asserts the whole thing
parses, every transaction balances, the inferred opening balance is `-$7,500`,
and the hand-computed `Assets:Checking` total (`$11,185.98`) matches.

**Behavioral examples:**

- Break a line in `examples/sample.journal` (e.g. a bad amount) → this test fails
  with the caret diagnostic, since `from_path` returns a `JournalError`.
- Add a transaction to the sample → the `sample_journal_parses` count assertion
  (`== 9`) fails until you update it.

---

## Commands

Run all of these from inside `app/`:

```bash
cd ~/Rust/tally/app
```

| What | Command |
|------|---------|
| Build everything | `cargo build` |
| Run the binary | `cargo run -p tally` |
| Run all tests | `cargo test` |
| Run tests, show output | `cargo test -- --nocapture` |
| Run a single test by name | `cargo test infers_the_blank_posting` |
| Test just the parser module | `cargo test -p tally-core parser` |
| Format code | `cargo fmt` |
| Check formatting without changing | `cargo fmt --check` |
| Lint (like eslint) | `cargo clippy` |
| Add a dependency | `cargo add <crate-name> -p tally-core` |

> `-p tally-core` or `-p tally` tells Cargo which crate in the workspace you
> mean. Without it, Cargo may not know which one you're targeting.

---

## Naming conventions

| Thing | Convention | Example |
|-------|-----------|---------|
| Crate names | `kebab-case` | `tally-core` |
| Crate names in code | `snake_case` | `use tally_core::...` |
| Files & modules | `snake_case` | `parser.rs`, `source_span` |
| Types (struct/enum) | `PascalCase` | `ParseError`, `Directive` |
| Functions & variables | `snake_case` | `parse_amount()`, `find_amount_sep` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_POSTINGS` |

---

## What comes next (phases)

| Phase | Adds | New modules in `core/` |
|-------|------|------------------------|
| 2 ✅ | Journal parser — turns `.journal` text into the model | `error.rs`, `parser.rs`, `journal.rs` |
| 3 | CLI commands: `bal`, `reg`, `accounts`, `print` | `query.rs`, `report.rs` |
| 4 | TUI: Register, Balances, Accounts views | (in `tally/src/tui/`) |
| 5 | TUI: Add/edit transactions, write back to file | (in `tally/src/tui/`) |
| 6 | Dashboard charts, budgets, `tally.toml` config | (in `tally/src/tui/`) |
