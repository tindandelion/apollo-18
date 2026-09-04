# Testing Apollo 18

The workspace quality gate covers Rust formatting, linting, tests, and the release web build:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd crates/web && trunk build index.html --release
```

## Native smoke tests

The native smoke tests execute the retained milestone binaries and validate the structure of their PNG artifacts. Temporary directories keep generated images out of the working tree.

From the repository root, run:

```bash
cargo test -p apollo18-native --test native_smoke
```

## Browser smoke test

The browser smoke test uses Playwright and headless Chromium to build, serve, and load the release web host. It verifies that the Wasm application initializes without runtime or resource errors, requests Canvas 2D rather than a GPU context, retains the canonical 800×800 canvas resolution, and presents non-background framebuffer pixels.

Install its Node dependencies and Chromium once:

```bash
cd crates/web/smoke-tests
npm ci
npx playwright install chromium
```

Then run the smoke test from the repository root:

```bash
scripts/web-smoke-test.sh
```

The script forwards additional Playwright arguments, such as `--headed`, after `npm test`. Playwright starts and stops a release-mode Trunk server automatically. Trunk output, Playwright results, and installed Node packages are written only to ignored directories.

## Browser performance test

The browser performance test measures completed `requestAnimationFrame`
callbacks while the release web host renders the canonical 800×800 lunar
globe. After a two-second warmup, it measures eight seconds of animation and
requires at least 30 FPS, matching the project's sustained desktop Wasm
performance target.

Run it from the repository root:

```bash
scripts/web-performance-test.sh
```

The test prints its measured FPS. Results are specific to the executing machine
and bundled Chromium version; use the same environment when comparing changes.
Additional Playwright arguments such as `--headed` are forwarded by the script.

## Golden images

Triangle and cube goldens require exact decoded RGBA pixels. Realistic lunar
goldens allow a maximum absolute difference of one per RGB channel, and up to
sixteen pixels may exceed that RGB tolerance to absorb rare platform
floating-point texel-boundary hits. Alpha must match exactly. A failure writes
an amplified PNG and numerical summary to `target/apollo18/golden-diffs/`.

Golden replacement is intentionally separate from normal test runs:

```bash
APOLLO18_UPDATE_GOLDENS=1 cargo test -p apollo18-renderer --lib \
  tests::terrain_shaded_lunar_globe_matches_golden_pixels
```

The replacement should be reviewed as a visible behavior change before it is
committed.
