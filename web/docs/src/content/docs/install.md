---
title: Installation
description: How to install Tally on Linux, macOS, and Windows.
---

## From cargo

```sh
cargo install tally
```

Requires a stable Rust toolchain (1.80+). This downloads and compiles `tally` into `~/.cargo/bin/`.

## From pre-built binaries

Download the latest release from the [GitHub releases page](https://github.com/murtazapatel89100/Tally/releases).

| Platform | File |
|----------|------|
| Linux x86-64 | `tally-x86_64-unknown-linux-musl.tar.gz` |
| macOS (Apple Silicon) | `tally-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `tally-x86_64-apple-darwin.tar.gz` |
| Windows x86-64 | `tally-x86_64-pc-windows-msvc.zip` |

Extract the archive and place the `tally` binary somewhere on your `PATH`.

## Shell completions

After installing, add completions for your shell:

```sh
# bash
tally completions bash >> ~/.bash_completion

# zsh
tally completions zsh > ~/.zfunc/_tally
# then add `fpath=(~/.zfunc $fpath)` + `autoload -U compinit && compinit` to ~/.zshrc

# fish
tally completions fish > ~/.config/fish/completions/tally.fish
```

## Quick test

```sh
tally bal -f /path/to/your.journal
```

If you see your account balances, you're ready to go.
