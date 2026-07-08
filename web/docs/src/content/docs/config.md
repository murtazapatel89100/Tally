---
title: Configuration
description: Configuring Tally via tally.toml.
---

Tally reads `tally.toml` from the current directory first, then from the OS config directory:

- **Linux**: `~/.config/tally/tally.toml`
- **macOS**: `~/Library/Application Support/com.tally.tally/tally.toml`
- **Windows**: `%APPDATA%\tally\tally\config\tally.toml`

## Options

```toml
# Default journal file (used when -f / $TALLY_FILE are not set)
file = "~/finance/2026.journal"

# Color theme: "dark" (default) or "light"
theme = "dark"

# Budget definitions — shown as progress gauges on the Dashboard
[[budgets]]
account = "Expenses:Food"
monthly = 600.00
label = "Food"

[[budgets]]
account = "Expenses:Utilities"
monthly = 150.00
label = "Utilities"

[[budgets]]
account = "Expenses:Entertainment"
monthly = 200.00
```

## Budget gauges

Budget gauges appear on the Dashboard. Each entry defines:

| Key | Type | Description |
|-----|------|-------------|
| `account` | string | Account prefix to match (e.g. `Expenses:Food` matches `Expenses:Food:Groceries` too). |
| `monthly` | number | Monthly spending limit in your primary currency. |
| `label` | string (optional) | Display label. Defaults to the `account` value. |

The gauge colour indicates progress:
- **Green** — under 80% spent
- **Yellow** — 80–99% spent  
- **Red** — at or over budget
