# Apollo 18 high-density web performance analysis

Date: 2026-09-07

## Executive summary

The 1152×1152 lunar-phase showcase is CPU-bound inside the shared Rust software renderer. It is not limited by Wasm-to-JavaScript transfer, `ImageData`, Canvas 2D presentation, layout, or JavaScript garbage collection.

The representative release test completed **16.46 FPS**, corresponding to roughly **60.8 ms per frame**, against a budget of **33.3 ms per frame** for 30 FPS. Controlled probes attribute approximately:

- **39–42 ms** to the lunar fragment shader excluding output encoding;
- **12–15 ms** to linear-to-sRGB output encoding;
- **about 6.5 ms** to the remaining rasterization, framebuffer/depth initialization, and host work;
- **about 0.2 ms** to `putImageData`, with negligible `ImageData` construction time.

The original analysis proposed caching a fixed screen-space lunar view. That recommendation is superseded by the completed ephemeris orientation work and Ticket 28's realistic timelines: sub-Earth libration and lunar position angle change the globe pose, so screen-space coverage, visible geography, map lookup, and world-space terrain normals are recurring work. Reusing them would freeze or approximate required behavior.

The measurements remain useful, but the next optimization must begin with a repeatable 1152×1152 profile of the ephemeris-span animation. Preparation may retain only measured pose-independent data, such as canonical mesh data or map-derived object-space quantities. The 30 FPS target cannot be predicted from the old fixed-view probe.

No production fixes were made during this analysis.

## Performance contract and environment

The investigation used the Ticket 19 workload:

- release Wasm build;
- Playwright Chromium 151.0.7922.34 in headless mode;
- 1440×900 CSS-pixel viewport;
- device pixel ratio 2;
- 730.625×730.625 CSS-pixel canvas;
- 1152×1152 canvas backing resolution;
- level-5 octasphere;
- canonical lunar color and elevation maps;
- terrain-normal lunar-phase animation;
- Canvas 2D `ImageData` presentation.

Reference machine:

- Apple MacBook Pro, model Mac15,7;
- Apple M3 Pro, arm64;
- macOS 15.7.7;
- Rust/Cargo 1.97.1;
- Trunk 0.21.14;
- installed Playwright package 1.62.1.

The canonical red-capable command was:

```bash
./scripts/dev/web-performance-test.sh
```

It reported:

```text
backingResolution: 1152x1152
measuredFramesPerSecond: 16.46
measurementSeconds: 7.96
Expected: >= 30
```

This closely reproduces Ticket 13's recorded 15.42 FPS baseline.

## Investigation method

Four complementary methods were used:

1. The existing release browser performance test established the end-to-end failure.
2. A Chrome DevTools Protocol CPU profile and browser timing wrappers separated Wasm execution from `ImageData`, `putImageData`, layout, and browser scheduling.
3. Temporary controlled probes bypassed one renderer stage at a time. These probes intentionally produced incorrect images and existed only to bound stage costs; all probe source changes were removed afterward.
4. A temporary native 1152×1152 render harness was sampled with macOS `sample` to recover Rust symbol names hidden by the optimized Wasm artifact.

Late timing experiments were discarded after an unrelated interactive Chrome renderer was observed consuming most of a CPU core. Absolute numbers below come from the initial repeatable measurements and profiles. This is also a reminder that final acceptance measurements must be taken with competing workloads controlled.

Raw temporary profiles and timing output are under ignored `target/performance-analysis/` output and are not project deliverables.

## Results

### End-to-end stage timing

The unprofiled callback timing was stable in its first two runs at 60.36 and 60.44 ms mean; a third run measured 63.63 ms. The first two runs agree with the canonical test's approximately 60.8 ms frame time.

| Controlled browser build | Mean/median frame callback | What remains |
|---|---:|---|
| Canonical renderer | about 60.4 ms | Complete render and presentation |
| Output encoding bypassed | about 45.5 ms | Complete lunar shading and rasterization, fixed pixel writes |
| Terrain-normal path bypassed | about 48.4 ms | Geographic/color lookup, smooth globe lighting, encoding, and rasterization |
| Lunar shader replaced by a constant color | about 18.5 ms | Rasterization, framebuffer/depth handling, output encoding, and host work |
| Constant shader and output encoding bypassed | about 6.5 ms | Rasterization, framebuffer/depth handling, and host work |

These probes are not perfectly additive because compiler optimization changes when work is removed. They nevertheless establish useful bounds:

- output encoding is material at roughly 12–15 ms;
- terrain-normal derivation contributes roughly 12 ms in the canonical pipeline;
- the broader lunar shader is the dominant cost at roughly 39–42 ms excluding encoding;
- rasterization and initialization are secondary and cannot by themselves explain the missed target.

At 1152×1152, the globe's 90% diameter covers approximately 844,000 pixels. Expensive fragment work therefore runs on the order of eight hundred thousand times per frame.

### Browser CPU profile

The instrumented eight-second profile raised the mean callback to 64.1 ms, so its sample percentages are more useful than its absolute frame time.

- Wasm execution accounted for more than 98% of sampled CPU time.
- The large inlined Wasm render function accounted for 76.9% self time.
- three math helpers accounted for another 11.0%, 6.0%, and 4.8%; inspection of their Wasm bodies and call sites is consistent with trigonometric/libm and rounding work used by geographic conversion, terrain slopes, and output encoding;
- `putImageData` accounted for 0.3% of sampled time;
- Wasm-to-JavaScript transitions, `requestAnimationFrame`, and `getBoundingClientRect` were individually negligible.

Browser timing wrappers measured:

| Stage | Mean per frame |
|---|---:|
| Full callback under profiler | 64.12 ms |
| `ImageData` construction | 0.005 ms |
| `putImageData` | 0.209 ms |

The generated JavaScript passes a `Uint8ClampedArray` view of Wasm memory directly to `ImageData`; there is no separate JavaScript pixel-copy loop. CDP reported zero layout time during the measurement, about 0.18 MB net JavaScript heap growth, and no sampled garbage-collector bottleneck.

### Native profile

The temporary native harness rendered 50 frames at 1152×1152 in 2.233 seconds:

```text
mean_ms=44.661
fps=22.39
```

A sampling profile attributed top-of-stack samples as follows:

| Native symbol/category | Samples |
|---|---:|
| Triangle fragment rasterization | 30.8% |
| Lunar shader body | 19.6% |
| `LunarElevationMap::perturbed_radial` | 17.1% |
| `LinearRgb::to_srgb8` | 16.2% |
| `cosf`, `asinf`, `atan2f`, `sinf`, and `hypotf` combined | about 12% |
| `Rasterizer::new` | 1.1% |
| Octasphere generation and other render setup | below 1% individually |

Native and Wasm percentages are not interchangeable, but both identify per-fragment work and output encoding as the important areas. Frame allocation, clearing, mesh generation, and host presentation are not dominant.

### Static Wasm observations

The release module validates with `wasm-tools` and contains no Wasm SIMD instructions. SIMD is therefore available as a later investigation direction, but the current scattered map accesses, scalar trigonometry, and lookup-table gathers are not an obvious first SIMD target.

## Hypothesis outcomes

### 1. Fragment shading dominates — confirmed

For each covered lunar fragment, the current implementation performs most of the following:

1. barycentrically interpolate and normalize the globe location;
2. calculate longitude with `atan2` and latitude with `asin`;
3. map geographic coordinates independently into color-map and elevation-map texels;
4. construct an east/north tangent frame, including `hypot`;
5. load neighboring elevation samples and calculate physical slopes;
6. evaluate `cos(latitude)` for the eastward physical distance;
7. construct, rotate, and normalize the terrain normal;
8. evaluate Lambertian intensity;
9. load linear lunar color and multiply it by intensity;
10. encode three linear channels to sRGB.

Replacing the lunar shader with a constant color reduced callback time from about 60.4 ms to 18.5 ms.

### 2. Rasterizer bookkeeping dominates — rejected as the primary cause

The current rasterizer recomputes three edge functions at every sample, divides all three edge values by triangle area for covered samples, and initializes fresh framebuffer and depth storage every frame. These are real optimization opportunities, but the constant-shader/no-encoding probe completed in about 6.5 ms. Rasterizer work alone is too small to recover the required 27 ms.

### 3. Framebuffer initialization or output encoding dominates — partially confirmed

Output encoding is substantial: bypassing it saved roughly 15 ms in the complete pipeline. Initialization and allocation are not substantial: native sampling put `Rasterizer::new` at 1.1%, and the stripped-down browser probe left only about 6.5 ms for initialization, rasterization, writes, and host work together.

Output encoding alone is not sufficient. Even with encoding bypassed, the complete shader took about 45.5 ms per frame, still over the 33.3 ms budget.

### 4. Wasm transfer or Canvas 2D presentation dominates — rejected

`ImageData` construction was effectively free at this measurement resolution, and `putImageData` averaged about 0.2 ms. The complete presentation path is well below 1% of the frame budget.

### 5. Allocation or JavaScript garbage collection dominates — rejected

No JavaScript garbage-collection hotspot appeared. JavaScript heap growth was small, and native allocation/initialization symbols were minor. Reuse may still improve locality and remove work, but GC is not the reason the target is missed.

## Original proposed improvements

The priorities below record the conclusions drawn from the fixed-view showcase measured at the time. Their timing evidence remains informative, but the screen-space preparation recommendation is superseded for the ephemeris-span animation because globe pose now changes every frame.

### Superseded priority 1: prepare the static lunar surface once per backing resolution

The measured lunar-phase scene kept the camera and globe fixed while only the Sun direction changed. The shared renderer repeated geometry generation, rasterization, geographic conversion, map sampling, and terrain-normal derivation every animation frame.

Introduce a deep shared-renderer module that prepares the visible lunar surface for a specific backing resolution and pair of lunar maps. Its small interface should support:

- preparation or rebuild when dimensions/maps change;
- deterministic rendering from `SceneTime` into the normal RGBA framebuffer.

The implementation can rasterize once and retain, for each visible pixel, the static inputs needed by later phase frames:

- pixel location or an equivalent dense index;
- linear lunar albedo, or a lunar-color-map sample index;
- the terrain normal in the fixed globe/world orientation.

A recurring frame then needs only Sun-direction calculation, a normal/Sun dot product, multiplication in linear RGB, sRGB encoding, and framebuffer writes. Resolution changes invalidate and rebuild the prepared surface. The existing one-shot `render_lunar_globe` function can remain as a deterministic convenience wrapper so golden and external behavior do not change; native sequences and the web animation should both use the shared prepared implementation rather than duplicating caching in the web adapter.

Why this is first:

- it removes the measured 39–42 ms recurring shader/geographic/terrain path structurally rather than tuning each scalar operation;
- it also removes recurring triangle traversal and most depth work;
- the constant-shader result of about 18.5 ms suggests enough margin beneath 33.3 ms for the retained Lambertian dot product and cached-data reads;
- it follows the scene's explicit phase invariant instead of reducing resolution, map detail, octasphere subdivision, or presentation quality.

Tradeoffs and checks:

- a cache containing three `f32` normal channels plus three `f32` albedo channels would cost roughly 20 MB for approximately 844,000 visible pixels before alignment/index overhead;
- storing a color-map sample index with the normal can reduce cache size at the cost of a random map read each frame;
- the first frame and resize frame still pay preparation cost;
- the cache must preserve exact top-left coverage, depth ownership, lunar orientation, map sampling, terrain normals, and deterministic output;
- this optimization depends on the current fixed-globe lunar-phase scene. A future rotating globe must rebuild or use a different strategy rather than silently reusing invalid prepared data.

### Priority 2: make output encoding cheaper

The current 4,097-entry table stores floating-point sRGB samples. Each channel clamps, computes a table position, loads two samples, interpolates them, multiplies by 255, and rounds. This costs roughly 12–15 ms per canonical frame.

Investigate a direct quantized-output table, likely a larger `u8` table indexed from clamped linear intensity. A 65,536-entry table is only 64 KiB and can be generated once. The lookup resolution should be selected by exhaustive comparison against the exact transfer function, with the existing requirement that encoded channels differ by at most one output code. This must be tested against all canonical goldens; a faster approximation is unacceptable if it exceeds the established tolerance or changes alpha.

This work is valuable both with and without prepared lunar data. It is not sufficient by itself to meet 30 FPS.

### Priority 3: precompute elevation-map gradient data

If a prepared per-pixel surface is rejected or proves too memory-heavy, move invariant elevation work from fragments into `LunarElevationMap` construction:

- precompute eastward and northward elevation derivatives or dimensionless slope terms per elevation texel;
- precompute map-dimension scale constants;
- retain longitude wrapping and polar-row behavior exactly.

Additionally, reuse the horizontal radius already needed by the tangent frame as `cos(latitude)` for a normalized globe location rather than evaluating a separate cosine. The controlled no-terrain probe indicates an upper bound of roughly 12 ms for this area.

This is secondary because removing terrain work alone left about 48.4 ms per frame.

### Priority 4: use incremental edge evaluation and reciprocal area

The edge functions are affine in screen coordinates. For each triangle, calculate values at the first sample and update them with constant x/y increments rather than rebuilding `Vec2` values and evaluating three perpendicular dot products for every candidate pixel. Calculate `1 / area` once and multiply edge values instead of performing three per-fragment divisions.

This preserves the edge-function learning model and top-left ownership rule. Focused shared-edge and exact triangle/cube golden tests must prove that changed evaluation order does not alter coverage unexpectedly.

The stripped-down probe bounds all remaining rasterizer/initialization/host work at about 6.5 ms, so this is not the first change to pursue.

### Priority 5: reuse framebuffer storage and precompute the octasphere

A retained renderer can reuse RGBA/depth allocations and keep the canonical level-5 octasphere instead of rebuilding its `HashMap`, vertices, and triangle list every frame. These changes reduce allocation churn and simplify preparation, but profiles show they are small optimizations rather than primary bottlenecks.

### Defer SIMD and threading

The current module has no Wasm SIMD instructions, but SIMD should follow structural improvements and a new profile. The hot path contains scalar trigonometry and scattered map/table reads, which limit straightforward vectorization. Threading remains outside the phase-one constraints and would require browser workers/shared memory and deployment changes; it should not be used to avoid fixing the single-threaded algorithm first.

## Revised implementation sequence

1. Add retained timing spans for render and presentation to the performance harness, without putting diagnostic logging in the hot path.
2. Measure the 1152×1152 ephemeris-span animation across representative pose, phase, and geographic samples.
3. Prepare only invariant mesh or lunar-map data whose cost is dominant in that realistic profile; do not cache a fixed screen-space view.
4. Verify unchanged deterministic pixels across representative ephemeris records and after a web resize.
5. Re-run the release browser performance contract in a controlled reference environment.
6. Optimize the newly measured dominant recurring cost and re-profile; investigate sRGB encoding, elevation gradients, incremental edges, allocation reuse, or SIMD only when evidence supports the choice.

## Required validation for any later fix

A retained optimization should not be accepted until all of the following pass:

```bash
./scripts/dev/quality-gate.sh
./scripts/dev/web-smoke-test.sh
./scripts/dev/web-performance-test.sh
```

It must also demonstrate:

- at least 30 completed, rendered, and presented FPS after warmup at 1152×1152;
- unchanged backing-resolution policy;
- unchanged canonical lunar golden output within the documented tolerance;
- unchanged native dimensions and deterministic sequence behavior;
- correct ephemeris-driven libration, roll, illumination, and geographic sampling;
- no fixed-view cache reused across incompatible poses;
- correct resolution-dependent storage reconstruction after responsive resizing;
- no skipped software-render or Canvas 2D presentation work in counted frames;
- recorded reference environment, baseline, final result, and profile evidence.
