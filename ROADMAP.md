# Tally — Roadmap

Ideas and planned improvements beyond the initial eight phases.
Items are loosely ordered by impact and effort; nothing here is committed.

---

## Desktop app (Tauri)

**Priority: Near-term**

A native desktop app wrapping the same `tally-core` engine via Tauri.
Because the domain logic lives in a pure Rust library with no terminal
dependencies, the web frontend can call it directly via Tauri commands —
no rewrite needed.

Planned scope:
- A web-based frontend (React or Svelte) calling `tally-core` via
  `tauri::command` bindings — same parser, reports, and printer.
- Native file picker for opening journals.
- System tray icon with quick net-worth summary.
- Native notifications for budget overage.
- Packaged as a signed `.dmg` / `.exe` / `.AppImage` distributed alongside
  the CLI binary in each release.

---

## CLI improvements

### Payee and tag filters on `bal` / `reg`
```sh
tally reg --payee "Grocery Store"
tally bal --tag category:food
```

### `--cleared` / `--pending` / `--uncleared` flags
Filter by transaction status on any report command.

### `tally stats` subcommand
Quick summary: total transactions, date range, top payees, commodity list,
journal file size.

### `--csv` / `--tsv` output flag
Machine-readable output for `bal` and `reg` so users can pipe into
spreadsheets or other tools.
```sh
tally reg --csv > transactions.csv
```

### `--depth N` flag for `bal`
Collapse the balance tree to a given depth:
```sh
tally bal --depth 1   # top-level accounts only
```

### `tally check` subcommand
Validate the journal without opening the TUI — useful in CI/pre-commit hooks.
Exit code 0 = clean, non-zero = parse errors.

### `include` directive with glob patterns
```ledger
include accounts/*.journal
```

### `--price-db` / commodity valuation (`-X`)
Convert multi-commodity journals to a target currency using a price
database, enabling unified net-worth reporting.

---

## TUI improvements

### Live file-watching (hot reload)
Use the `notify` crate to watch the journal file for external changes
(e.g. edited in another editor) and reload instantly without restarting.

### Jump-to-date in Register
`gd` to open a date-picker; scroll to the nearest entry.

### `:` command palette
Vim-style command entry for actions not bound to a single key:
`:goto 2026-03`, `:export csv`, `:open ~/other.journal`.

### Multi-file support
Open multiple journals in tabs or merge them into a single view:
```sh
tally -f 2025.journal -f 2026.journal
```

### Transaction templates / recurring entries
Pre-fill common transactions from a template list; mark which ones
recur monthly so the TUI can prompt to enter them.

### Undo / redo for in-TUI edits
Keep an in-memory edit history so `Ctrl+Z` reverses the last save.

### Reconciliation mode
Mark postings as reconciled against a bank statement, highlight
unreconciled entries, and show the reconciled vs. unreconciled balance.

### Search across all fields
Extend `/` filter to match on date, amount, tag values, and comments —
not just payee and account name.

### Colour-coding by account type
Configurable per-account-type colours in `tally.toml`:
```toml
[account_colors]
"Assets"      = "green"
"Liabilities" = "red"
"Expenses"    = "yellow"
```

---

## Core / parser improvements

### Periodic / budget transactions
Parse ledger's `~ Monthly` syntax and auto-generate entries for budget
comparisons (already referenced in `tally.toml` budgets but not driven
from the journal itself yet).

### `balance` assertion support
```ledger
2026-01-31 * Balance assertion
    Assets:Checking    $0 = $8,685.98
```
Validate on parse; surface mismatches as errors.

### `auto` (automated) transactions
Ledger's `= /Expenses/` rules that automatically add postings when a
pattern matches — useful for tax lot tracking, splits, etc.

### Commodity lots and cost basis
`{$45.00}` lot price syntax for tracking cost basis of investments.

### `P` price directives
```ledger
P 2026-07-01 AAPL $195.00
```
Build a price history database and use it for `--price-db` valuation.

### Tolerant amount parsing
Accept amounts with missing cents (`$5` → `$5.00`) and handle locale
decimal separators (`,` in European format).

---

## Config (`tally.toml`) improvements

### Per-account display aliases
```toml
[display]
"Assets:Checking:Primary" = "Checking"
```

### Default report flags
```toml
[defaults]
depth = 2
cleared_only = true
```

### Account colour overrides
See TUI improvements above.

### Multiple journal paths with merge
```toml
file = ["~/finance/2026.journal", "~/finance/investments.journal"]
```

---

## Developer / ecosystem

### WASM build of `tally-core`
Compile the pure core crate to WebAssembly so the same engine can
power a browser-based view (e.g. drag-and-drop a `.journal` file into
the docs site for a live demo).

### `tally-core` published to crates.io
Expose the parser, model, and reports as a stable library so other
tools can build on Tally's engine.

### Bank import plugins
OFX / CSV importer as an optional plugin crate — not in the core
binary, but installable:
```sh
cargo install tally-import-ofx
tally import ofx checking.ofx >> 2026.journal
```

### hledger `balance --tree` compatibility test suite
Run the same journals through both `hledger` and `tally bal` and
diff the output — ensures ledger-compatible correctness.

### VS Code / Neovim extension
Syntax highlighting and snippets for `.journal` / `.ledger` files,
with Tally's error diagnostics surfaced via LSP.

---

## Release pipeline improvements

### Homebrew formula
`brew install tally` via a tap:
```
brew tap murtazapatel89100/tally
brew install tally
```

### AUR (Arch Linux) package
`yay -S tally-bin` from the Arch User Repository.

### Nix flake
`nix run github:murtazapatel89100/Tally` with a reproducible build.

### Shell completion in release archives
Bundle the generated completions (bash/zsh/fish) alongside the binary
so users don't need to run `tally completions` manually.

### Automatic changelog from Conventional Commits
`git-cliff` or `release-please` to generate `CHANGELOG.md` and
pre-fill GitHub release notes on each tag push.
