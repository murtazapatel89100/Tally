# Tally Web App (planned)

> 🚧 **Future plan — not yet implemented.**

A browser-based GUI version of the Tally CLI. The goal is to bring the same views the
terminal UI offers — dashboard, balances tree, register, and in-app transaction
entry — to the web, running entirely client-side.

## Approach

Because all of Tally's domain logic lives in [`tally-core`](../../app/core) — a pure Rust
library with **no terminal or UI dependencies** — the same parser, reports, and printer
can be compiled to **WebAssembly** and called directly from the browser. No server, no
backend: you drag a `.journal` file in, and everything runs locally.

This mirrors the desktop (Tauri) plan in the [ROADMAP](../../ROADMAP.md): one engine
(`tally-core`), multiple frontends (CLI/TUI, desktop, web).

## Planned scope

- WASM build of `tally-core` exposing parse → reports → print.
- A single-page app (framework TBD) rendering the Dashboard, Balances, Register, and
  Accounts views.
- Client-side only — journals never leave the browser.
- Shared visual language with the TUI themes (Tokyo Night / Nord / light).

## Status

Placeholder. See the [WASM build](../../ROADMAP.md) item in the roadmap. Contributions and
design proposals are welcome — open an issue to discuss before starting.
