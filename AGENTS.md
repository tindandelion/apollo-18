# Apollo 18 agent guidance

## Project context

Apollo 18 is a learning-first Rust software renderer for presenting the Moon with NASA lunar surface data. Use the canonical project language in `CONTEXT.md`; read relevant decisions under `docs/adr/` before changing renderer conventions.

## Planned implementation

When implementing the renderer, native binaries, web showcase, lunar assets, or rendering tests, read `.scratch/apollo-18/spec.md` and the ticket being worked under `.scratch/apollo-18/issues/`. Work one frontier ticket at a time; completion requires every acceptance criterion and the ticket's quality gate to pass.

## Learning documentation

For every graphics stage, add or update a concise guide under `docs/learning/` that explains the stage's reasoning and equations. Keep tutorial material in these guides and reserve code comments for local implementation reasoning.

## Lunar Globe: Ground Truth

Use NASA SVS's north-up 2026 lunar frames as the visual ground truth for ephemeris-driven phase, libration, and position angle:

- Source page: <https://svs.gsfc.nasa.gov/5587/>
- Hourly 730×730 frame set: <https://svs.gsfc.nasa.gov/vis/a000000/a005500/a005587/frames/730x730_1x1_30p/>
- Direct frame pattern: `https://svs.gsfc.nasa.gov/vis/a000000/a005500/a005587/frames/730x730_1x1_30p/moon.NNNN.jpg`

Frames are chronological hourly samples. `moon.0001.jpg` is `2026-01-01T00:00Z`; calculate later frame numbers as hours since that instant plus one, padded to four digits. Compare phase, terminator, central lunar location, and landmark roll visually. Apollo 18 uses different surface assets and rendering, so these frames are orientation references rather than pixel-comparison fixtures.

## NASA assets

Keep runtime lunar data checked in under `assets/nasa/` rather than fetching it over the network. Whenever an asset is added or replaced, add or update its adjacent provenance document with the source URL, retrieval date, checksum, and usage guidance. Keep NASA data provenance and usage terms separate from Apollo 18's code license.

## Validation

Run the canonical quality gate from the repository root before completing every implementation ticket:

```bash
./scripts/dev/quality-gate.sh
```

Also run every ticket-specific native-output, asset-provenance, and deterministic-animation check required by its acceptance criteria.

For web initialization, Canvas presentation, backing-resolution, or responsive behavior changes, also run:

```bash
scripts/dev/web-smoke-test.sh
```

For performance-sensitive renderer or web changes, run:

```bash
scripts/dev/web-performance-test.sh
scripts/dev/web-timeline-performance-test.sh
```

Compare performance on the same machine and browser using three warmed runs. Do not claim changes smaller than observed run-to-run variation. Count only frames that perform one software render and one Canvas 2D presentation at the 1152×1152 cap. Set `APOLLO18_SCENE_TIME_OFFSET_SECONDS` to compare a fixed lunar appearance.

Install browser-test dependencies when needed:

```bash
cd crates/web/smoke-tests
npm ci
npx playwright install chromium
```

### Golden renders

Triangle and cube goldens require exact decoded pixels; lunar goldens use the tolerance encoded by `crates/renderer/tests/golden_renders.rs`. Failed lunar comparisons write amplified differences under `target/apollo18/golden-diffs/`.

Golden replacement is an explicit visible behavior change:

```bash
APOLLO18_UPDATE_GOLDENS=1 cargo test -p apollo18-renderer \
  --test golden_renders golden_pixels
```

Visually review every replaced golden before committing it.

### Unit tests

Write each unit test in the Arrange-Act-Assert pattern: set up inputs, exercise one behavior, then assert the observable result, with a blank line between those phases. A `///` doc comment on the test describes the scenario it covers.

## Deployment

The [public web showcase](https://www.tindandelion.com/apollo-18/) is hosted on GitHub Pages because the canonical repository remote is GitHub. `.github/workflows/deploy-website.yml` builds and deploys the site on pushes to `main` and supports manual workflow dispatch. Preserve the Pages base path supplied to Trunk so JavaScript and Wasm assets resolve under both the repository path and the configured custom domain.

## Review conventions

Native artifact encoding and golden-fixture encoding are separate responsibilities that may diverge. Similar PNG-writing code across those boundaries is intentional, not duplicated code to extract.

## Branches

Start development of each new feature by creating and switching to a dedicated feature branch.

## Commits

Before creating a Git commit, propose the exact commit message and wait for the user's confirmation.

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

This repo uses the default five canonical triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

This repo uses a single-context domain docs layout. See `docs/agents/domain.md`.
