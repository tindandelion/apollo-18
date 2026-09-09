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

### Browser performance diagnostic

The retained diagnostic instruments the same release browser workload and
separates each completed animation callback into software rendering,
`ImageData` construction, and Canvas 2D presentation. A frame is included only
when it constructs and presents exactly one 1152×1152 `ImageData`. After a
two-second warmup, the diagnostic samples for eight seconds and requires at
least 60 completed frames. It reports sustained completed-frame throughput and
the median duration of each stage so isolated optimization results can be
compared without occasional scheduling pauses dominating the result.

Run it from the repository root:

```bash
scripts/dev/web-timeline-performance-test.sh
```

The diagnostic prints the host, operating system, architecture, processor,
browser and version, viewport, device pixel ratio, canvas CSS dimensions,
backing resolution, warmup, sample size, throughput, and median stage timings.
Compare changes on the same host and browser. Use three warmed runs, and do not
claim effects smaller than the observed run-to-run variation.

The Ticket 29 baseline was measured on 2026-09-09 in four separate release
browser runs on the reference environment below. `ImageData` construction was
below the browser timer's precision at the median:

| Run | Frames | Completed FPS | Software rendering | `ImageData` | Canvas presentation | Complete frame |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 129 | 16.09 | 61.8 ms | 0.0 ms | 0.2 ms | 62.0 ms |
| 2 | 129 | 16.09 | 61.8 ms | 0.0 ms | 0.2 ms | 61.9 ms |
| 3 | 129 | 16.06 | 61.9 ms | 0.0 ms | 0.2 ms | 62.1 ms |
| 4 | 118 | 14.69 | 66.0 ms | 0.0 ms | 0.2 ms | 66.1 ms |

The complete-frame medians span 4.2 ms across these runs, so an isolated change
of that size or less is inconclusive without interleaved baseline and changed
build measurements.

The separate sustained-FPS contract measured 15.20 FPS over 7.96 seconds in a
separate warmed run. Its expected 30 FPS assertion remains open for Ticket 19.

Ticket 30 compared three baseline and three changed runs on the same reference
environment on 2026-09-09. It replaced per-channel floating-point rounding
with equivalent non-negative integer-and-fraction quantization:

| Variant | Run | Frames | Completed FPS | Software rendering | Complete frame |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 1 | 130 | 16.13 | 61.6 ms | 61.8 ms |
| Baseline | 2 | 127 | 15.77 | 64.1 ms | 64.3 ms |
| Baseline | 3 | 121 | 15.07 | 66.7 ms | 66.8 ms |
| Integer-and-fraction quantization | 1 | 145 | 18.00 | 55.2 ms | 55.4 ms |
| Integer-and-fraction quantization | 2 | 145 | 18.00 | 55.2 ms | 55.4 ms |
| Integer-and-fraction quantization | 3 | 144 | 17.97 | 55.3 ms | 55.5 ms |

The median complete-frame time improved from 64.3 ms to 55.4 ms, an 8.9 ms
(13.8%) reduction. Even the fastest baseline and slowest changed medians are
separated by 6.3 ms, larger than both this baseline's 5.0 ms run-to-run span
and Ticket 29's prior 4.2 ms noise bound. The separate sustained-FPS contract
rerun measured 17.95 FPS over 7.97 seconds. This is a cumulative improvement,
but the expected 30 FPS assertion remains open for the later Ticket 19 work.

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
