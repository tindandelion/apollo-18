# High-density web rendering performance analysis

## Scope

This document analyzes Ticket 19's existing 1152×1152 release-browser workload. It records measurements, identifies recurring bottlenecks, and proposes experiments; it does not implement a performance fix.

The required output remains a single-threaded software-rendered level-5 octasphere presented through Canvas 2D `ImageData`, with the existing lunar color map, lunar elevation map, terrain-normal shading, ephemeris-driven appearance, and canvas backing resolution.

## Executive summary

The renderer is CPU-bound inside Wasm, not presentation-bound.

- Three release-browser runs sustained **15.55–15.97 FPS** (mean **15.81 FPS**) against the **30 FPS** target.
- The existing timeline probe measured a **63.42 ms** complete frame: **63.20 ms** in recurring render/host work and only **0.22 ms** in `putImageData`.
- A more focused temporary probe measured `ImageData` construction at about **0.007 ms/frame** and `putImageData` at about **0.21 ms/frame**. The framebuffer crossing is not the missing 30 FPS.
- A five-second Chrome CPU profile attributed about **98.9%** of sampled time to Wasm functions. About **27.9%** of total samples were in three out-of-line numerical helpers, while the large inlined render callback accounted for **70.7%**.
- A symbolized native sample corroborated two broad recurring costs: fragment shading/terrain/color encoding (about **62%**) and triangle traversal/rasterization (about **32%**).
- A temporary constant-black shader, which retained the current rasterizer, depth test, sRGB framebuffer encoding, Wasm handoff, and Canvas presentation, sustained **54.43–54.87 FPS**. Full lunar shading therefore accounts for roughly 45 ms, or 71%, of browser frame time in this comparison.
- A second temporary shader kept color mapping and smooth-globe Lambertian lighting but removed elevation sampling and terrain-normal derivation. It sustained **20.82–21.65 FPS** (mean **21.33 FPS**), so terrain mapping is significant but removing it alone does not reach 30 FPS.
- Per-frame allocation, clearing, and mesh generation are visible but secondary. `Rasterizer::new` accounted for about **1.1%** of native samples.
- Plain Binaryen `wasm-opt -O4` improved the artifact-only experiment only to **16.27–16.37 FPS**. Enabling Wasm SIMD globally regressed it to **14.86–15.05 FPS**.

At 63.42 ms per completed frame, reaching 30 FPS's 33.33 ms budget requires approximately a **47% frame-time reduction** or **1.9× throughput**. No single build flag or presentation optimization is likely to supply that. The best path is a combined optimization of triangle traversal and per-fragment lunar shading, followed by a post-link Wasm optimization pass.

## Reference environment

Measurements were taken on 2026-09-09 with:

| Component | Value |
| --- | --- |
| Machine | MacBook Pro `Mac15,7` |
| CPU | Apple M3 Pro, 12 cores (6 performance, 6 efficiency) |
| Memory | 36 GB |
| Architecture | arm64 |
| Operating system | macOS 15.7.9 (24G830) |
| Rust/Cargo | 1.97.1 |
| Trunk | 0.21.14 |
| Node.js | 22.19.0 |
| Project-local Playwright | 1.62.1 |
| Browser | Headless Chrome for Testing 151.0.7922.34 |
| Browser viewport | 1440×900 CSS pixels |
| Device pixel ratio | 2 |
| Canvas CSS dimensions | approximately 730.625×730.625 CSS pixels |
| Canvas backing resolution | 1152×1152 |
| Warmup | 2 seconds |
| Measurement window | 8 seconds |

The repository's `package.json` requests `@playwright/test` with `^1.58.2`; the installed lockfile-resolved project-local version is 1.62.1. A separately resolvable `npx` version is not used by the project test scripts.

## Feedback loop and baseline

The red-capable command is:

```bash
cd crates/web/smoke-tests
npm run test:performance
```

It drives the release Trunk build, verifies the 1152×1152 canvas backing resolution, and counts a frame only after the animation callback has returned. The callback renders and calls `putImageData` before requesting its successor, so the test cannot meet its target by counting callbacks that skip rendering or presentation.

Three runs produced:

| Run | FPS | Measured interval |
| ---: | ---: | ---: |
| 1 | 15.97 | 7.95 s |
| 2 | 15.92 | 7.97 s |
| 3 | 15.55 | 7.97 s |
| **Mean** | **15.81** | |

The narrow 0.42 FPS range reproduced the reported failure reliably.

The existing diagnostic command:

```bash
cd crates/web/smoke-tests
npm run test:timeline-performance
```

reported:

| Measurement | Result |
| --- | ---: |
| Startup preparation | 368.50 ms |
| Mean recurring render, host work, and framebuffer handoff excluding `putImageData` | 63.20 ms |
| Mean `putImageData` presentation | 0.22 ms |
| Mean complete frame | 63.42 ms |
| Derived throughput | 15.77 FPS |

Startup asset decoding is outside recurring frame time and therefore is not responsible for the sustained-FPS failure.

## Profiling method

Temporary probes were used and removed after measurement:

1. A Chrome DevTools Protocol sampling profile captured five seconds after a two-second warmup at the required resolution.
2. A browser wall-clock probe wrapped the `ImageData` constructor and `CanvasRenderingContext2D.putImageData` for 30 completed frames.
3. A temporary native release harness decoded the same checked-in assets once and repeatedly called `render_lunar_globe(1152, 1152, ...)` for 15 seconds. macOS `sample` captured five seconds with release debug symbols.
4. Artifact-only builds tested Binaryen `wasm-opt -O4` and Rust's `+simd128` target feature. They were served from `target/` and did not replace source or retained build output.
5. A geometry calculation recreated the level-5 octasphere at the canonical square projection to estimate candidate-fragment pressure from per-triangle bounding boxes.

Raw temporary artifacts remain ignored under `target/performance-analysis/`; no temporary source instrumentation remains.

## Findings

### 1. Canvas and framebuffer handoff are not bottlenecks

The wall-clock probe measured, over 30 frames:

| Stage | Mean time |
| --- | ---: |
| Complete callback | 66.74 ms |
| `ImageData` construction | 0.007 ms |
| `putImageData` | 0.21 ms |
| Remaining renderer and host work | 66.52 ms |

The complete callback differs slightly from the separate baseline because it was a shorter instrumented sample, but the stage proportions are decisive. Chrome's CPU profile also sampled only 14.6 ms total in `putImageData` over five seconds and 0.5 ms total in `ImageData` construction.

The 1152×1152 RGBA framebuffer is 5,308,416 bytes. Despite its size, the current `Uint8ClampedArray`/`ImageData` path does not account for meaningful recurring wall time on this browser. Changing framebuffer format would add conversion work and is unsupported by this evidence.

### 2. Almost all browser CPU time is inside Wasm

The five-second Chrome profile aggregated samples as follows:

| Profile entry | Share of sampled time |
| --- | ---: |
| Large inlined Wasm render callback (`wasm-function[79]`) | 70.7% |
| Out-of-line numerical helper (`wasm-function[111]`) | 16.5% |
| Out-of-line numerical helper (`wasm-function[226]`) | 6.4% |
| Out-of-line rounding helper (`wasm-function[475]`) | 5.0% |
| All remaining browser/JS/native entries | 1.4% |

Release symbol names are stripped, so the Chrome profile cannot directly name the first two numerical helpers. Inspection of the Wasm and the symbolized native profile ties the hot out-of-line work to per-fragment trigonometric/numerical operations; function 475 implements float rounding and is called at the three-channel sRGB write boundary.

No Wasm SIMD instructions were present in the normal release artifact.

### 3. Native profiling separates rasterization from shading

The native harness sustained **21.85–21.94 FPS**, or about **45.7 ms/frame**. Native execution is faster than Wasm but still misses the 33.33 ms target, demonstrating that algorithmic work must be reduced even before accounting for the Wasm/native performance gap.

The five-second symbolized native sample's self-time distribution was approximately:

| Hot work | Share |
| --- | ---: |
| Rasterizer triangle traversal and fragment loop | 32.1% |
| `LunarShader::shade` own work | 18.5% |
| Terrain-normal derivation | 17.4% |
| Linear-to-sRGB table encoding | 14.6% |
| `cosf`, `atan2f`, `asinf`, `sinf`, and `hypotf` combined | 11.7% |
| `Rasterizer::new` allocation/clear | 1.1% |
| Other setup and teardown | approximately 4.6% |

The shading-related rows together account for roughly **62%** of native samples. Raster traversal accounts for another **32%**. These are the two optimization fronts that can plausibly recover the required frame budget.

### 4. Per-fragment lunar shading repeats expensive work

For every covered fragment, the current path performs most or all of the following:

- interpolate and normalize the globe location;
- calculate longitude with `atan2` and latitude with `asin`;
- convert coordinates to a nearest texel for terrain lookup;
- calculate a tangent frame using `hypot`;
- fetch up to four neighboring elevation samples and calculate physical slopes;
- calculate `cos(latitude)` even though a related horizontal radius was already calculated for the tangent frame;
- rotate and normalize the perturbed radial;
- calculate Lambertian intensity;
- convert the same geographic coordinates to a nearest texel again for color lookup;
- perform three interpolated table lookups and rounding operations for sRGB output.

At the canonical projection, the visible disk covers approximately **844,000 pixels**. Small costs in this path are therefore multiplied hundreds of thousands of times per frame.

A temporary constant-black `LunarShader` experiment removed globe-location interpolation, geographic conversion, terrain sampling and normals, color-map sampling, and lighting. It deliberately retained triangle traversal, depth testing, framebuffer writes, the existing three-channel sRGB encoding boundary, Wasm handoff, and Canvas presentation. Three release-browser runs measured **54.68, 54.43, and 54.87 FPS** (mean **54.66 FPS**). The timeline probe measured **18.14 ms** recurring render/host work, **0.22 ms** presentation, and **18.36 ms** per complete frame. Restoring the full shader returned the performance test to **16.00 FPS**.

Compared with the full-shader 63.42 ms frame, the experiment removed approximately **45.06 ms**, or **71%**, and demonstrated that the current rasterizer can exceed the 30 FPS target when paired with trivial fragment work. This is an upper bound rather than the gain available from any one optimization: the experiment removed all lunar shading semantics and cannot be retained.

A second temporary experiment isolated terrain mapping more narrowly. It replaced the elevation-derived perturbed radial with the smooth normalized globe location, while retaining globe-location interpolation, `atan2`/`asin` geographic conversion, lunar color-map sampling, globe lighting, sRGB encoding, and the complete raster/presentation path. Three warmed release-browser runs measured **20.82, 21.65, and 21.52 FPS** (mean **21.33 FPS**). The timeline probe measured **49.10 ms** recurring render/host work, **0.21 ms** presentation, and **49.31 ms** per complete frame. On the same timeline basis, removing terrain work saved **14.11 ms/frame**, or about **22%** of the full 63.42 ms frame. The full terrain shader was then restored and measured **15.63 FPS**.

This confirms terrain derivation as a material bottleneck, but also shows that color mapping, geographic conversion, smooth lighting, sRGB encoding, and rasterization still consume roughly 47–49 ms/frame. Terrain optimization alone cannot meet the 33.33 ms target.

A symbolized five-second native sample of this smooth color-mapped variant breaks down that remainder further:

| Native top-of-stack function or operation | Share of samples |
| --- | ---: |
| `LunarShader::shade` own/inlined work | 38.7% |
| `rasterize_fragments` own work | 32.3% |
| `LinearRgb::to_srgb8` | 8.7% |
| `asinf` | 7.2% |
| `atan2f` | 5.5% |
| Dynamic stubs for `asin`/`atan2` | 2.4% |
| `Rasterizer::new` | 1.7% |
| Other work | 3.5% |

The compiler inlines globe-location interpolation and normalization, nearest-texel arithmetic, color-map access, normal transformation, normalization, and the diffuse dot product into `LunarShader::shade`, so the 38.7% row is a bundle rather than color-map memory access alone. Geographic trigonometry (`asin` and `atan2`, including their stubs) accounts for approximately **15%** separately. Raster traversal is the largest individually named leaf at **32.3%**, and sRGB encoding is another **8.7%**.

A matching Chrome profile confirms that Wasm magnifies numerical costs: **72.7%** was in the large inlined render function, **14.9%** in the out-of-line three-channel float-rounding helper used by sRGB encoding, **10.5%** in another out-of-line numerical helper associated with geographic conversion, and about **1.9%** everywhere else. These profiles make the remaining bottleneck concrete: it is not one slow color-map array read, but the combined per-fragment geographic/shading/encoding pipeline plus raster candidate traversal.

### 5. Bounding-box traversal examines substantially more samples than it shades

A level-5 octasphere contains 8,192 triangles, of which approximately 4,096 face the camera. At the canonical square identity projection, summing their integer bounding-box areas gives about **2.15 million candidate samples** for a disk containing about **844,000 covered samples**—roughly **2.54 candidate tests per covered fragment**.

The precise ratio varies slightly with pose and pixel alignment, but the source of overhead is stable: the rasterizer evaluates all three edge functions from scratch at every pixel in each small triangle's bounding box. Most candidate samples are rejected, while accepted samples additionally divide all three edge values by triangle area.

This explains why raster traversal consumes about one third of native profile samples even though the lunar globe has little meaningful overdraw.

### 6. Per-frame setup is real but too small to lead the work

Every frame currently:

- allocates and clears a 5.3 MB framebuffer;
- allocates and initializes a 5.3 MB `f32` depth buffer;
- regenerates the static level-5 octasphere through repeated `HashMap`-backed subdivision;
- transforms the same shared mesh vertex once per triangle reference rather than once per unique vertex.

These are avoidable costs, but profiling does not support treating them as the first intervention. `Rasterizer::new` was about 1.1% of native samples, and mesh generation did not appear as a separately significant top-of-stack entry. Caching and reuse should follow improvements to recurring per-candidate and per-fragment work unless a later browser-specific allocation trace contradicts this result.

### 7. Generic post-link optimization helps only marginally

An artifact-only `wasm-opt -O4` experiment produced **16.27–16.37 FPS**, compared with the unmodified mean of **15.81 FPS**. That is a small improvement of roughly 3–4%, well short of 30 FPS. It may be worth retaining after correctness checks, but it cannot substitute for source-level work.

A release build with Rust's global `-C target-feature=+simd128` produced SIMD instructions but regressed to **14.86–15.05 FPS**. Automatic vectorization does not fit the current branch-heavy, triangle-at-a-time inner loop well enough to help. SIMD should not be enabled globally on the evidence available. It may become useful only after data layout and raster traversal are deliberately restructured and separately benchmarked.

## Ranked improvement plan

Each item below is a proposal for a measured implementation experiment, not an already validated fix. The follow-up experiments show that accepted-fragment shading is the dominant aggregate cost (about 71% in the constant-shader differential), while raster traversal remains the largest individually named leaf (about 32% in the smooth color-mapped native profile). The priorities therefore identify two required optimization fronts; they should not be read as claiming rasterization costs more than the complete shader pipeline.

### Priority 1: Reduce triangle candidate traversal and edge arithmetic

The strongest rasterization experiment should combine:

1. **Incremental edge equations.** Evaluate each edge once at a bounding-box row origin, then advance with precomputed X/Y increments instead of rebuilding a `Vec2` and recomputing three perpendicular dot products per candidate pixel.
2. **Precompute reciprocal area.** Multiply edge values by one reciprocal rather than divide each of three values by area for every covered fragment.
3. **Tighter scanline spans.** Derive conservative X bounds per row, while retaining the exact pixel-center and top-left acceptance test at span boundaries. This can avoid much of the estimated 2.54× bounding-box candidate pressure.

Incremental equations alone reduce arithmetic but continue visiting rejected candidates. Scanline bounds alone reduce visits but leave expensive repeated edge evaluation. Their combination has the best chance of materially reducing the 32% raster share while preserving the educational edge-function model and exact shared-edge behavior.

Required validation: all triangle winding, shared-edge, clipping, depth, exact small golden, and realistic lunar golden tests. Floating-point stepping can change edge equality if implemented carelessly, so top-left ownership must remain explicit and deterministic.

### Priority 2: Fuse geographic and terrain calculations per fragment

Several semantics-preserving opportunities should be benchmarked independently:

- Compute nearest lunar texel coordinates once and share them between elevation and color sampling instead of calling `nearest_texel` twice.
- Reuse the horizontal radius calculated for the tangent frame as `cos(latitude)` for a normalized globe location, avoiding duplicate transcendental work. Compare outputs carefully because equivalent equations can round differently.
- Precompute longitude and latitude elevation central differences per source texel when the elevation map is loaded. Per-fragment work would still apply the lunar-reference-radius and latitude-dependent scale, preserving nearest-neighbor terrain semantics while replacing four scattered source reads and repeated difference calculations with compact derivative reads.
- Transform the Sun direction into object space once per frame, then dot it with the normalized object-space terrain normal. A rotation preserves vector lengths and dot products, so this can remove the per-fragment matrix transform while retaining geographically consistent illumination.

These changes target the terrain and shader work that occupies at least 36% of native samples, in addition to some out-of-line math cost.

Required validation: terrain-normal unit tests, lunar appearance tests, all realistic goldens, and explicit comparison at terminators and accepted terrain-normal rim highlights.

### Priority 3: Avoid unnecessary night-side color and sRGB work

Lambertian intensity is clamped to zero on the unlit side. Once the terrain normal has established an exact zero intensity, the renderer can potentially write opaque black without:

- sampling the lunar color map;
- multiplying three linear channels;
- interpolating the sRGB encode table three times;
- rounding three channels.

This preserves terrain-normal rim highlights because the early decision occurs after terrain-normal lighting, not from a geometric globe-location day-side gate. Its benefit changes with lunar phase, so it must be measured across representative ephemeris states rather than only the first test interval.

The sRGB path itself accounts for about 14.6% of native samples. A second experiment could replace interpolation-plus-rounding with a quantization-oriented lookup or threshold representation designed around the final 256 output codes. Any replacement must first prove the existing exact small-output behavior and realistic-golden tolerance; a faster approximation that shifts terminator or surface output is not acceptable.

### Priority 4: Cache immutable mesh work and transform unique vertices

Cache the canonical level-5 octasphere rather than regenerating it per frame, and transform its 4,098 unique vertices once before indexing 8,192 triangles. This should simplify profiles and reduce low-level setup, but the measured ceiling is small compared with traversal and shading.

Avoid introducing a broad renderer-state API solely to reuse framebuffer allocations until a focused browser allocation profile shows a larger benefit than the native sample. The existing high-leverage deterministic render seam is more valuable than an unmeasured allocation optimization.

### Priority 5: Apply post-link optimization after source-level gains

Re-test `wasm-opt -O4` after each major source-level improvement. Its current 3–4% gain could become useful near the target, but generated-code changes must be checked against:

- Wasm validation;
- output/golden correctness;
- browser smoke tests;
- module size and startup behavior;
- current Chrome, Firefox, and Safari compatibility.

Do not enable global Wasm SIMD based on the current experiment. If later raster or shading loops use a structure-of-arrays or multi-pixel organization suitable for explicit SIMD, benchmark that specific kernel against scalar code and retain runtime/browser compatibility evidence.

## Lower-priority or unsupported directions

- **Optimize Canvas 2D presentation:** unsupported; measured at about 0.2 ms/frame.
- **Use an RGB framebuffer:** unsupported; Canvas requires RGBA and current transfer cost is negligible.
- **Lower canvas backing resolution, map detail, or octasphere subdivision:** prohibited by the ticket.
- **Skip rendering or presentation on counted frames:** prohibited by the performance contract.
- **Move to WebGL/WebGPU:** intentionally deferred until reasonable software-renderer optimization is exhausted.
- **Enable threading first:** the renderer is intentionally single-threaded, and a GitHub Pages deployment would need additional worker/shared-memory and cross-origin-isolation design. The current profile offers substantial scalar work to remove first.
- **Special-case away the depth buffer for a convex sphere:** potentially measurable, but it introduces a lunar-specific rendering path and should be considered only after deeper general rasterizer and shader improvements. The native profile suggests a lower payoff than the first three priorities.

## Suggested implementation sequence and gates

1. Add retained stage timing around software rendering, `ImageData` construction, and presentation, plus a repeatable symbolized sampling/differential harness for shader attribution.
2. Implement and benchmark incremental edge stepping plus reciprocal area as one bounded raster experiment.
3. Reprofile the complete terrain render and smooth color-mapped render. Add tighter scanline traversal only if candidate traversal remains prominent enough to justify its greater coverage-rule risk.
4. Fuse shared geographic texel work, object-space lighting, and terrain-gradient preparation one change at a time, profiling after each change.
5. Add the exact-zero night-side fast path.
6. Investigate a quantization-oriented sRGB encoder if Wasm rounding/encoding remains prominent.
7. Reprofile and choose the next step from measured residuals rather than assuming either rasterization or shading remains dominant.
8. Cache immutable mesh/unique transformed vertices if setup remains visible.
9. Re-evaluate `wasm-opt -O4`; consider explicit SIMD only for a newly suitable kernel.

For every retained change:

- run the 1152×1152 release-browser performance test multiple times after warmup;
- compare stage timings and a CPU profile rather than only aggregate FPS;
- run browser smoke coverage at device pixel ratio 2 and after resize;
- run deterministic native output and all golden tests;
- run `./scripts/dev/quality-gate.sh`;
- record the exact environment, baseline, final FPS, and output-correctness evidence.

## Conclusion

The 30 FPS failure is not caused by JavaScript callbacks, `ImageData`, Canvas presentation, startup decoding, or one conspicuous allocation. It is the cumulative cost of a clear but deliberately unoptimized inner pipeline: roughly 2.15 million candidate tests feed about 844,000 expensive lunar fragments each frame, and each accepted fragment performs geographic trigonometry, terrain-gradient work, normal transforms, map access, lighting, and sRGB encoding.

The most credible route to 30 FPS is therefore a combined reduction in **candidate-pixel raster work** and **accepted-fragment shading work**. Generic Wasm optimization can provide a small final gain, but the measured 1.9× requirement demands algorithmic and data-flow improvements first.
