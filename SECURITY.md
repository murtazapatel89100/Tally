# Security Policy

## Supported versions

Tally is pre-1.0 and under active development. Security fixes are applied to the
latest release only.

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |
| < 0.1   | ❌        |

## Threat model

Tally is a **local, offline** tool. It reads and writes plain-text journal files
on your machine and makes **no network connections** of its own. The main surfaces
worth scrutiny are:

- The **journal parser** (`tally-core`) — handling of untrusted/malformed input.
- **File writes** from in-TUI transaction entry/edit — correctness of what gets
  persisted to your journal.

There is no server, telemetry, or account system.

## Reporting a vulnerability

**Please do not open a public issue for security vulnerabilities.**

Report privately using either of:

1. **GitHub private vulnerability reporting** — go to the
   [Security tab](https://github.com/murtazapatel89100/Tally/security/advisories/new)
   and open a draft advisory.
2. **Email** — <murtazapatel89100@gmail.com> with the subject line `SECURITY: Tally`.

Please include:

- A description of the issue and its impact.
- Steps to reproduce (a minimal `.journal` snippet is ideal).
- The Tally version (`tally --version`) and your OS.

You can expect an initial acknowledgment within **7 days**. We'll keep you updated
on progress and coordinate a disclosure timeline once a fix is ready. We're happy
to credit you in the release notes unless you prefer to remain anonymous.
