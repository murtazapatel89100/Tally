# Tally

A modern, plain-text accounting tool — like [`ledger`](https://www.ledger-cli.org/)
and [`hledger`](https://hledger.org/), but written in Rust with a fast, polished TUI.

Tally reads standard ledger/hledger journal files (so you can switch instantly)
and layers modern conveniences on top: first-class tags, friendly parse errors,
budgets, and an interactive terminal UI for exploring, entering, and visualizing
your finances.

## Repository layout

| Path   | Description |
|--------|-------------|
| `app/` | The Rust application — `tally-core` library + `tally` CLI/TUI binary. See [`app/prd.md`](app/prd.md). |
| `web/` | Documentation + landing site. |

## Status

Early development. Current milestone: **M0 — scaffold & PRD**.
See [`app/prd.md`](app/prd.md) for the full roadmap.

## Quick goals

- Read existing ledger/hledger journals unchanged.
- A TUI that's genuinely pleasant: register, balances, accounts, search, entry,
  and charts.
- Exact decimal money math (no floats, ever).
- Single static binary, zero runtime dependencies.
