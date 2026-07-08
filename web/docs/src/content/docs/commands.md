---
title: Commands
description: CLI command reference for Tally.
---

## Global flags

| Flag | Description |
|------|-------------|
| `-f, --file <PATH>` | Journal file to read. Falls back to `$TALLY_FILE`, then `$LEDGER_FILE`, then the `file` key in `tally.toml`. |
| `-h, --help` | Print help. |
| `-V, --version` | Print version. |

## `tally` / `tally tui`

Launch the interactive TUI.

```sh
tally -f ~/finance/2026.journal
TALLY_FILE=~/finance/2026.journal tally
```

## `tally bal`

Show account balances in a tree. Aggregates sub-account totals.

```sh
tally bal -f journal.ledger
tally bal Expenses               # filter to Expenses subtree
tally bal --from 2026-01-01 --to 2026-03-31
```

| Flag | Description |
|------|-------------|
| `[ACCOUNT]` | Optional account prefix filter. |
| `--from DATE` | Start date (inclusive), `YYYY-MM-DD`. |
| `--to DATE` | End date (exclusive), `YYYY-MM-DD`. |

## `tally reg`

Show a chronological posting register with running totals.

```sh
tally reg -f journal.ledger
tally reg Assets:Checking
tally reg --from 2026-01-01
```

Accepts the same `[ACCOUNT]`, `--from`, `--to` flags as `bal`.

## `tally accounts`

List all known accounts (declared or seen in transactions), one per line.

```sh
tally accounts -f journal.ledger
```

## `tally print`

Re-serialize the journal in canonical ledger format. Use this for round-trip verification or normalizing formatting.

```sh
tally print -f journal.ledger
tally print -f journal.ledger > journal.normalized.ledger
```

## `tally completions`

Print shell completion script.

```sh
tally completions bash   >> ~/.bash_completion
tally completions zsh    > ~/.zfunc/_tally
tally completions fish   > ~/.config/fish/completions/tally.fish
tally completions elvish
tally completions powershell
```
