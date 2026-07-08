# web/

Web-facing projects for Tally. This folder holds two independent apps:

| Path | Description | Status |
|------|-------------|--------|
| [`docs/`](docs/) | Documentation & landing site (Astro + Starlight), deployed to [tally.rs](https://tally.rs). | ✅ Live |
| [`app/`](app/) | Web GUI version of the Tally CLI, powered by a future WASM build of `tally-core`. | 🚧 Planned |

Each subfolder is self-contained with its own `package.json` and toolchain — there is no
root-level workspace. Work on them independently.

See [`docs/`](docs/) to build the documentation site and [`app/`](app/) for the plan for
the web GUI.
