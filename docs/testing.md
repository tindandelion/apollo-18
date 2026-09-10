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
claim effects smaller than the observed run-to-run variation. To hold one
lunar appearance fixed while measuring phase-dependent work, set
`APOLLO18_SCENE_TIME_OFFSET_SECONDS` to a non-negative ephemeris-span scene
time. The first callback still establishes scene time zero; subsequent measured
callbacks receive the fixed offset. Without the variable, the normal two-minute
animation advances during the diagnostic.

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

Ticket 31 compared three warmed baseline and three changed runs on the same
reference environment. The experiment transformed the world-space Sun
direction into lunar-globe object space once per frame and removed the
object-to-world normal transformation from each lunar fragment:

| Variant | Run | Frames | Completed FPS | Software rendering | Complete frame |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 1 | 141 | 17.50 | 56.2 ms | 56.4 ms |
| Baseline | 2 | 144 | 17.92 | 55.4 ms | 55.6 ms |
| Baseline | 3 | 143 | 17.83 | 55.7 ms | 55.9 ms |
| Object-space illumination | 1 | 148 | 18.37 | 53.95 ms | 54.1 ms |
| Object-space illumination | 2 | 146 | 18.17 | 54.7 ms | 54.9 ms |
| Object-space illumination | 3 | 141 | 17.61 | 56.5 ms | 56.7 ms |

The median complete-frame time changed from 55.9 ms to 54.9 ms. That 1.0 ms
change is smaller than the changed variant's 2.6 ms run-to-run span, the
baseline and changed ranges overlap, and both are inside Ticket 29's 4.2 ms
noise bound. The experiment was therefore inconclusive and reverted. Its
sustained-FPS contract run measured 18.52 FPS over 7.99 seconds; as expected
for this intermediate performance ticket, the later Ticket 19 threshold of
30 FPS remains open.

Ticket 32 compared three warmed baseline and three changed runs on the same
reference environment. It reused the tangent frame's horizontal radius when
calculating physical eastward terrain slope instead of independently
calculating the equivalent latitude cosine for every lunar fragment:

| Variant | Run | Frames | Completed FPS | Software rendering | Complete frame |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 1 | 143 | 17.79 | 55.8 ms | 56.0 ms |
| Baseline | 2 | 144 | 17.94 | 55.4 ms | 55.6 ms |
| Baseline | 3 | 140 | 17.38 | 57.2 ms | 57.4 ms |
| Reused horizontal radius | 1 | 148 | 18.42 | 53.85 ms | 54.1 ms |
| Reused horizontal radius | 2 | 148 | 18.42 | 54.0 ms | 54.2 ms |
| Reused horizontal radius | 3 | 148 | 18.42 | 53.9 ms | 54.1 ms |

The median complete-frame time improved from 56.0 ms to 54.1 ms, a 1.9 ms
(3.4%) reduction. The ranges do not overlap, and the reduction is larger than
the baseline's 1.8 ms run-to-run span, so the change was retained. The
separate sustained-FPS contract rerun measured 18.22 FPS over 7.96 seconds;
the later Ticket 19 threshold of 30 FPS remains open.

Ticket 33 measured both the normal advancing animation and two fixed lunar
appearances on the same reference environment. The fixed full-Moon-like sample
uses scene time `20.22148647105834` (2026-03-03 12:00 UTC), and the fixed
new-Moon-like sample uses `73.5700422422651` (2026-08-12 18:00 UTC). The latter
preserves ADR-0005 terrain-normal rim highlights rather than forcing the disk
to geometric black.

| Appearance | Variant | Run | Frames | Completed FPS | Software rendering | Complete frame |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Full-Moon-like | Baseline | 1 | 147 | 18.36 | 54.0 ms | 54.2 ms |
| Full-Moon-like | Baseline | 2 | 148 | 18.37 | 54.0 ms | 54.2 ms |
| Full-Moon-like | Baseline | 3 | 138 | 17.18 | 57.7 ms | 57.9 ms |
| Full-Moon-like | Unlit bypass | 1 | 140 | 17.44 | 56.9 ms | 57.1 ms |
| Full-Moon-like | Unlit bypass | 2 | 139 | 17.35 | 57.2 ms | 57.4 ms |
| Full-Moon-like | Unlit bypass | 3 | 137 | 17.08 | 58.1 ms | 58.3 ms |
| New-Moon-like | Baseline | 1 | 138 | 17.17 | 57.8 ms | 58.0 ms |
| New-Moon-like | Baseline | 2 | 144 | 17.92 | 55.0 ms | 55.1 ms |
| New-Moon-like | Baseline | 3 | 147 | 18.29 | 54.2 ms | 54.4 ms |
| New-Moon-like | Unlit bypass | 1 | 210 | 26.17 | 37.1 ms | 37.3 ms |
| New-Moon-like | Unlit bypass | 2 | 212 | 26.38 | 37.5 ms | 37.7 ms |
| New-Moon-like | Unlit bypass | 3 | 209 | 26.10 | 37.9 ms | 38.1 ms |

At the illuminated sample, the median complete-frame time changed from 54.2 ms
to 57.4 ms. The 3.2 ms difference is inside Ticket 29's 4.2 ms noise bound,
and the ranges overlap, so no illuminated-phase effect is claimed. At the
unlit sample, the median improved from 55.1 ms to 37.7 ms, a 17.4 ms (31.6%)
reduction with non-overlapping ranges. The benefit is phase-dependent because
only exactly zero terrain-normal Lambertian intensity bypasses lunar color-map
sampling and general sRGB encoding.

Three normal advancing-animation runs measured 22.31, 22.80, and 21.85
completed FPS, with median complete-frame times of 43.6, 42.6, and 43.8 ms.
Compared with Ticket 32's 54.1 ms cumulative median, the representative median
improved by 10.5 ms (19.4%). The bypass was therefore retained. The separate
sustained-FPS contract rerun measured 22.17 FPS over 7.98 seconds; the later
Ticket 19 threshold of 30 FPS remains open.

Ticket 34 compared direct edge evaluation with incremental X/Y edge stepping
and reciprocal-area barycentric normalization on the same reference
environment. All variants traversed the same framebuffer-bounded candidate
rectangles; tighter scanline spans were not introduced.

| Variant | Run | Frames | Completed FPS | Software rendering | Complete frame |
| --- | ---: | ---: | ---: | ---: | ---: |
| Direct edge evaluation | 1 | 182 | 22.76 | 42.25 ms | 42.45 ms |
| Direct edge evaluation | 2 | 182 | 22.61 | 42.65 ms | 42.80 ms |
| Direct edge evaluation | 3 | 183 | 22.73 | 42.30 ms | 42.50 ms |
| Incremental edge stepping | 1 | 183 | 22.74 | 42.50 ms | 42.70 ms |
| Incremental edge stepping | 2 | 182 | 22.72 | 42.85 ms | 42.95 ms |
| Incremental edge stepping | 3 | 176 | 22.01 | 43.75 ms | 43.95 ms |
| Direct evaluation and reciprocal area | 1 | 172 | 21.37 | 45.40 ms | 45.55 ms |
| Direct evaluation and reciprocal area | 2 | 172 | 21.41 | 45.40 ms | 45.60 ms |
| Direct evaluation and reciprocal area | 3 | 172 | 21.41 | 45.40 ms | 45.55 ms |
| Incremental stepping and reciprocal area | 1 | 187 | 23.29 | 41.40 ms | 41.60 ms |
| Incremental stepping and reciprocal area | 2 | 187 | 23.30 | 41.20 ms | 41.40 ms |
| Incremental stepping and reciprocal area | 3 | 184 | 22.94 | 41.40 ms | 41.60 ms |

Neither incremental stepping nor reciprocal-area multiplication was beneficial
in isolation: incremental stepping overlapped the baseline, while reciprocal
normalization with direct edge evaluation regressed to a 45.55 ms median. The
combined experiment improved the baseline median complete-frame time from
42.50 ms to 41.60 ms, a 0.9 ms (2.1%) reduction. Its three measurements did not
overlap the three baseline measurements, and its 0.2 ms span was smaller than
the separation. Exact triangle and cube goldens and tolerance-checked lunar
goldens passed unchanged, but the small whole-frame gain did not justify the
added floating-point accumulation risk and loop complexity. All arithmetic
changes were therefore reverted. The experimental variant's separate
sustained-FPS contract run measured 23.13 FPS over 7.95 seconds; the later
Ticket 19 threshold of 30 FPS remains open.

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
