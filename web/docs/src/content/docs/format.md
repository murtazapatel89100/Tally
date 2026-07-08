---
title: Journal Format
description: The ledger-compatible plain-text format that Tally reads and writes.
---

Tally reads the same format as [ledger](https://ledger-cli.org/) and [hledger](https://hledger.org/), so existing journals open unchanged.

## Basic structure

```text
; A comment line
2026-01-15 * (CODE) Grocery Store        ; payee with cleared status & code
    Expenses:Food:Groceries    $45.00    ; posting with amount
    Assets:Checking                       ; blank amount → inferred as -$45.00
```

## Transactions

A transaction starts with a date, an optional status marker, an optional code in parentheses, and the payee. Indented lines are postings.

```text
DATE [STATUS] [(CODE)] PAYEE [; COMMENT]
    ACCOUNT    [AMOUNT]    [; COMMENT]
    ACCOUNT    [AMOUNT]    [; COMMENT]
```

- **DATE**: `YYYY-MM-DD`, `YYYY/MM/DD`, or `YYYY.MM.DD`
- **STATUS**: `*` = cleared, `!` = pending, none = uncleared
- **CODE**: optional reference code in parentheses, e.g. `(REF-123)`
- **AMOUNT inference**: exactly one posting per transaction may omit its amount; it is computed so the transaction balances to zero

## Amounts

```text
; Prefixed commodity
$45.00        ; US dollars
-$3.50        ; negative
$1,234.56     ; thousands separator

; Suffixed commodity
45.00 EUR
10 AAPL
```

## Comments

```text
; Semicolon comment
# Hash comment
% Percent comment
* Star comment (at start of line)
| Pipe comment (at start of line)

2026-01-15 * Payee    ; trailing transaction comment
    Expenses:Food  $10   ; trailing posting comment
```

## Tags & metadata

```text
2026-01-15 * Payee
    ; category: food
    ; :tag1:tag2:
    Expenses:Food  $10
    Assets:Checking
```

## Directives

```text
; Declare accounts (optional, for validation/autocomplete)
account Assets:Checking
account Expenses:Food:Groceries

; Declare commodities
commodity $
commodity EUR

; Aliases
alias Cash = Assets:Checking

; Include another file
include more.journal
```

## Round-trip guarantee

`tally print` re-serializes the journal to canonical form that re-parses identically. In-TUI edits use this same serializer.

## Example journal

```text
account Assets:Checking
account Assets:Savings
account Expenses:Food:Groceries
account Income:Salary

2026-01-01 * Opening Balances
    Assets:Checking    $5,000.00
    Assets:Savings     $2,500.00
    Equity:Opening Balances

2026-01-05 * Grocery Store
    Expenses:Food:Groceries    $123.45
    Assets:Checking

2026-01-15 * Employer
    Assets:Checking    $3,500.00
    Income:Salary     -$3,500.00
```
