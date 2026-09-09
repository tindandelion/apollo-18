# Testing Apollo 18

The workspace quality gate covers Rust formatting, linting, tests, and the release web build. Run its canonical script from the repository root:

```bash
./scripts/dev/quality-gate.sh
```

## Native smoke tests

The native smoke tests execute the retained milestone binaries and validate the structure of their PNG artifacts. Temporary directories keep generated images out of the working tree.

From the repository root, run:

```bash
cargo test -p apollo18-native --test native_smoke
```

## Browser smoke test

The browser smoke test uses Playwright and headless Chromium to build, serve, and load the release web host. It verifies that the Wasm application initializes without runtime or resource errors, requests Canvas 2D rather than a GPU context, selects a backing resolution from the canvas CSS dimensions and device pixel ratio, and presents non-background framebuffer pixels. Its high-density scenario uses a 1440×900 CSS-pixel viewport at device pixel ratio 2 and verifies the 1152×1152 cap, framebuffer presentation at that same resolution, responsive resizing, and a device-pixel-ratio change.

Install its Node dependencies and Chromium once:

```bash
cd crates/web/smoke-tests
npm ci
npx playwright install chromium
```

Then run the smoke test from the repository root:

```bash
scripts/dev/web-smoke-test.sh
```

The script forwards additional Playwright arguments, such as `--headed`, after `npm test`. Playwright starts and stops a release-mode Trunk server automatically. Trunk output, Playwright results, and installed Node packages are written only to ignored directories.

## Browser performance test

The browser performance test measures completed `requestAnimationFrame`
callbacks while the release web host renders the representative lunar globe at
the capped high-density backing resolution. It uses a 1440×900 CSS-pixel
viewport at device pixel ratio 2, verifies the 1152×1152 backing resolution,
warms up for two seconds, and measures eight seconds of animation. Ticket 13
records the baseline without a minimum frame-rate gate; Ticket 19 will restore
the sustained 30 FPS target at this resolution.

Run it from the repository root:

```bash
scripts/dev/web-performance-test.sh
```

The test prints its measured FPS and browser environment. Results are specific
to the executing machine and bundled Chromium version; use the same environment
when comparing changes. Additional Playwright arguments such as `--headed` are
forwarded by the script.

The Ticket 13 baseline was measured on 2026-09-06 with:

- Apple Mac15,7 with an Apple M3 Pro (`arm64`)
- macOS 15.7.7
- Playwright Chromium 151.0.7922.34 in headless mode
- 1440×900 CSS-pixel viewport at device pixel ratio 2
- 730.625×730.625 CSS-pixel canvas and 1152×1152 backing resolution
- 15.42 FPS over 7.98 measured seconds after the two-second warmup

## Golden images

Golden render tests exercise the renderer's public scene interfaces from
`crates/renderer/tests/golden_renders.rs`. Triangle and cube goldens require exact decoded RGBA pixels. Realistic lunar
goldens allow a maximum absolute difference of one per RGB channel, and up to
sixteen pixels may exceed that RGB tolerance to absorb rare platform
floating-point texel-boundary hits. Alpha must match exactly. A failure writes
an amplified PNG and numerical summary to `target/apollo18/golden-diffs/`.

Golden replacement is intentionally separate from normal test runs:

```bash
APOLLO18_UPDATE_GOLDENS=1 cargo test -p apollo18-renderer \
  --test golden_renders golden_pixels
```

The replacement should be reviewed as a visible behavior change before it is
committed.
