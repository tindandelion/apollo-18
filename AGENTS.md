# Apollo 18 agent guidance

## Project context

Apollo 18 is a learning-first Rust software renderer for presenting the Moon with NASA lunar surface data. Use the canonical project language in `CONTEXT.md`; read relevant decisions under `docs/adr/` before changing renderer conventions.

## Planned implementation

When implementing the renderer, native binaries, web showcase, lunar assets, or rendering tests, read `.scratch/apollo-18/spec.md` and the ticket being worked under `.scratch/apollo-18/issues/`. Work one frontier ticket at a time; completion requires every acceptance criterion and the ticket's quality gate to pass.

## Learning documentation

For every graphics stage, add or update a concise guide under `docs/learning/` that explains the stage's reasoning and equations. Keep tutorial material in these guides and reserve code comments for local implementation reasoning.

## Quality gate

Run these checks before completing every implementation ticket:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd crates/web && NO_COLOR=true trunk build index.html --release
```

Also run every ticket-specific golden, native-output, browser, asset-provenance, and deterministic-animation check required by its acceptance criteria.

## Deployment

The [public web showcase](https://www.tindandelion.com/apollo-18/) is hosted on GitHub Pages because the canonical repository remote is GitHub. `.github/workflows/deploy-website.yml` builds and deploys the site on pushes to `main` and supports manual workflow dispatch. Preserve the Pages base path supplied to Trunk so JavaScript and Wasm assets resolve under both the repository path and the configured custom domain.

## Review conventions

Native artifact encoding and golden-fixture encoding are separate responsibilities that may diverge. Similar PNG-writing code across those boundaries is intentional, not duplicated code to extract.

## Commits

Before creating a Git commit, propose the exact commit message and wait for the user's confirmation.

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

This repo uses the default five canonical triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

This repo uses a single-context domain docs layout. See `docs/agents/domain.md`.
