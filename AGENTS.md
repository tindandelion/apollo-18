# Apollo 18 agent guidance

## Project context

Apollo 18 is a learning-first Rust software renderer for presenting the Moon with NASA lunar surface data. Use the canonical project language in `CONTEXT.md`; read relevant decisions under `docs/adr/` before changing renderer conventions.

## Planned implementation

When implementing the renderer, native binaries, web showcase, lunar assets, or rendering tests, read `.scratch/apollo-18/spec.md` and the ticket being worked under `.scratch/apollo-18/issues/`. Work one frontier ticket at a time; completion requires every acceptance criterion and the ticket's quality gate to pass.

## Quality gate

Run these checks before completing every implementation ticket:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
trunk build --release crates/web/index.html
```

Also run every ticket-specific golden, native-output, browser, asset-provenance, and deterministic-animation check required by its acceptance criteria.

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

This repo uses the default five canonical triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

This repo uses a single-context domain docs layout. See `docs/agents/domain.md`.
