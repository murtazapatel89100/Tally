<!--
Thanks for contributing to Tally! Please fill in the sections below.
See CONTRIBUTING.md for the checks your change needs to pass.
-->

## Summary

<!-- What does this PR do, and why? -->

## Related issue

<!-- e.g. "Closes #123". Delete if not applicable. -->

## Type of change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that changes existing behavior)
- [ ] Documentation only
- [ ] Refactor / chore (no user-facing change)

## Checklist

- [ ] `cargo fmt --manifest-path app/Cargo.toml --all --check` passes
- [ ] `cargo clippy --manifest-path app/Cargo.toml --all-targets --all-features` is clean
- [ ] `cargo test --manifest-path app/Cargo.toml --all` passes
- [ ] Snapshots reviewed (`cargo insta review`) if report output changed
- [ ] Docs updated (`web/` and/or README) if behavior changed
- [ ] `CHANGELOG.md` updated under **[Unreleased]** for user-facing changes
- [ ] Commits follow [Conventional Commits](https://www.conventionalcommits.org/)

## Notes for reviewers

<!-- Anything that needs extra context: design trade-offs, follow-ups, screenshots for TUI changes. -->
