# Tally — Product Requirements Document

**Status:** Draft · **Owner:** service@bandofcapes.com · **Last updated:** 2026-06-30

---

## 1. Overview & vision

Tally is a plain-text, double-entry accounting tool for the terminal. It follows
the philosophy of `ledger` and `hledger` — your data lives in a human-readable,
version-controllable text file that you own — but delivers a **modern, fast,
genuinely pleasant TUI** and a clean Rust implementation.

The bet: plain-text accounting is powerful but has a steep on-ramp. The existing
CLIs are report generators; you still write transactions by hand in an editor and
re-run commands to see results. Tally closes that loop with an interactive UI for
exploring, entering, and visualizing finances — without giving up the plain-text
file as the source of truth.

**One-line pitch:** *ledger's data model, a modern terminal experience.*

## 2. Problem statement

Plain-text accounting users today juggle a text editor (to write entries) and a
CLI (to query). The feedback loop is slow, entry is error-prone (typos in account
names, unbalanced transactions discovered only on the next command run), and there
is no built-in way to *browse* or *visualize* the data. Newcomers bounce off the
syntax and the lack of guidance.

## 3. Goals & non-goals

### Goals
- **G1.** Parse standard ledger/hledger journals so existing users switch with zero migration.
- **G2.** A TUI that makes exploring data (register, balances, accounts, search) instant.
- **G3.** In-TUI transaction entry/editing with account autocomplete and live balance validation, written back to the journal file.
- **G4.** Visual reporting: net-worth trend, expense breakdown, budget progress.
- **G5.** Exact money arithmetic (decimal, never float).
- **G6.** Single self-contained binary; fast startup even on large journals.

### Non-goals (initial)
- Not a bank-sync / OFX / Plaid importer (may come later as a plugin).
- Not a GUI/desktop app (the `web/` site is docs + landing only for now).
- Not a full reimplementation of every ledger feature on day one (see milestones).
- No multi-user / server / cloud component.

## 4. Target users

- **Existing ledger/hledger users** who want a better day-to-day experience.
- **Plain-text-accounting-curious** developers who found the existing tools too bare.
- **Privacy/ownership-minded** people who want local, file-based finances under version control.

## 5. Competitive landscape

| Tool | Lang | Strengths | Gaps Tally addresses |
|------|------|-----------|----------------------|
| ledger | C++ | Fast, mature, powerful query language | No interactive UI, no guided entry |
| hledger | Haskell | Great docs, web UI add-on, robust | Terminal UI is basic; entry still editor-based |
| beancount | Python | Strong correctness, Fava web UI | Different (non-ledger) syntax; web-centric |
| **Tally** | **Rust** | **First-class TUI, in-app entry, charts, ledger-compatible** | — |

## 6. File format

### 6.1 Compatible subset (must parse)

```ledger
; A comment line
2026-01-15 * (CODE) Grocery Store        ; payee with cleared status & code
    Expenses:Food:Groceries    $45.00    ; a posting with amount
    Assets:Checking                       ; blank amount -> inferred (= -$45.00)
```

- **Transactions:** `DATE [*|!] [(CODE)] PAYEE` header, then indented postings.
  - `*` = cleared, `!` = pending, none = uncleared.
- **Postings:** `ACCOUNT<2+ spaces>AMOUNT`. Account is colon-delimited hierarchy.
- **Amount inference:** exactly one posting per transaction may omit its amount;
  it is computed so the transaction balances to zero.
- **Amounts/commodities:** prefixed (`$45.00`, `-$3`) and suffixed (`45.00 USD`,
  `10 AAPL`); thousands separators (`$1,234.56`); negative amounts.
- **Comments:** line comments (`;`, `#`, `%`, and `*`/`|` at start of line) and
  trailing `; ...` comments on transactions/postings.
- **Tags & metadata:** `; key: value` and `; :tag1:tag2:` forms.
- **Directives:** `include FILE`, `alias OLD = NEW`, `account NAME`,
  `commodity SYM`, plus tolerant skipping of unsupported directives with a warning.

### 6.2 Extensions ("better than ledger")

- Tags/metadata are **first-class**: indexed and usable as TUI filters and in reports.
- Optional **`tally.toml`** config: default journal path, color theme, budget
  definitions, account display options.
- **Friendly parse errors**: file, line, column, and a caret pointing at the
  offending token — never a bare "parse error".
- (Later) Periodic / budget transactions (`~ Monthly` style) for budget reports.

### 6.3 Round-trip guarantee

`tally print` re-serializes parsed transactions to canonical ledger syntax that
re-parses to an identical model. In-TUI edits write back using this serializer,
preserving surrounding hand-written content where feasible.

## 7. Functional requirements

### 7.1 CLI (non-interactive)

| Command | Aliases | Description |
|---------|---------|-------------|
| `tally` / `tally tui` | — | Launch the interactive TUI |
| `tally balance` | `bal` | Account balance report (tree, totals) |
| `tally register` | `reg` | Chronological postings with running total |
| `tally accounts` | `acc` | List all accounts |
| `tally print` | — | Canonically re-serialize the journal |

Common flags: `-f/--file <PATH>` (else `$TALLY_FILE`, else `$LEDGER_FILE`),
date range (`--begin`, `--end`), account/payee/tag filters, `--cleared`/`--pending`.

### 7.2 TUI views

- **FR-Dashboard:** net-worth sparkline over time; expense-by-category bar chart;
  budget gauges (when budgets are configured).
- **FR-Register:** scrollable, searchable list of postings; live filtering by
  account / payee / tag / date range; running balance column; jump-to-date.
- **FR-Balances:** collapsible account tree with aggregated subtotals and totals.
- **FR-Accounts:** flat, searchable account list with quick-jump into register.
- **FR-Entry/Edit:** modal form (date, payee, status, postings). Account fields
  offer **fuzzy autocomplete** from existing accounts; live "balances to zero"
  indicator; on save, the transaction is appended/updated in the journal file and
  the model re-parsed. Validation prevents saving an unbalanced/invalid entry.

### 7.3 Cross-cutting

- **FR-Search:** global incremental search/filter usable from any list view.
- **FR-Errors:** parse errors shown in a panel with location; the app stays usable
  for the valid portion where possible.
- **FR-Config:** read `tally.toml` if present for theme/budgets/default file.

## 8. UX & keybindings (initial)

- Vim-ish navigation: `j/k` move, `g/G` top/bottom, `/` search, `:` command palette.
- `Tab`/number keys switch top-level views (Dashboard, Register, Balances, Accounts).
- `a` add transaction, `e` edit selected, `Enter` drill in, `?` help overlay, `q` quit.
- A theme system (default dark + light); colors configurable via `tally.toml`.

## 9. Technical design

### 9.1 Stack

| Concern | Choice |
|---------|--------|
| TUI | ratatui + crossterm |
| CLI args | clap (derive) |
| Money | rust_decimal (exact decimal) |
| Dates | time |
| Parsing | hand-written / nom |
| Text input | tui-textarea / tui-input |
| Config | serde + toml |
| Errors | thiserror (lib) + anyhow (bin) |
| Fuzzy match | nucleo / fuzzy-matcher |

### 9.2 Architecture (Cargo workspace)

```
app/
├── Cargo.toml            # workspace: members = ["core", "tally"]
├── core/  (tally-core)   # pure domain logic, no TUI deps — WASM-reusable
│   └── src/{model,parser,journal,query,report}.rs
└── tally/ (binary)
    └── src/{main.rs, cli/, tui/{app,theme,views/,widgets/}}
```

Keeping all domain logic in `tally-core` (no terminal deps) leaves a clean path to
a future WASM-powered web app reusing the same engine.

### 9.3 Data model (core)

- `Commodity { symbol, position }`
- `Amount { quantity: Decimal, commodity }`
- `Posting { account: Account, amount: Option<Amount>, status, tags, comment }`
- `Transaction { date, status, code, payee, postings, tags, source_span }`
- `Account` = colon-delimited path; `Journal` holds transactions + an account index
  + resolved aliases/includes.

## 10. Milestones

| # | Name | Deliverable |
|---|------|-------------|
| M0 | Scaffold | Folders, README, this PRD ✅ |
| M1 | Core + CLI | `tally-core` model/parser/reports; `bal`/`reg`/`accounts`/`print` against a sample journal; parser unit tests |
| M2 | TUI explore | Register, Balances, Accounts views + search/filter |
| M3 | TUI entry | Add/edit form, fuzzy autocomplete, write-back to journal |
| M4 | Charts/budgets | Dashboard sparkline/bar charts/gauges; `tally.toml` budgets |
| M5 | Docs/landing | Astro + Starlight site in `web/`: landing + format spec + command reference |
| M6 | Release | Versioned binary, install instructions, CI |

## 11. Success metrics

- An existing ledger/hledger journal opens unchanged with correct balances.
- Time-to-first-insight (open file → see balances) under a few seconds for large files.
- A transaction can be entered end-to-end in the TUI without touching an editor.
- `tally print` round-trips fixtures byte-equivalently (modulo canonical formatting).

## 12. Open questions

- Commodity **price/valuation** support (market prices, `-X`) — which milestone?
- How aggressively to **preserve original formatting/comments** on in-TUI edits.
- **Budget syntax**: reuse ledger periodic transactions vs. define in `tally.toml`.
- Multi-currency reporting and conversion — in scope for v1 or later?
