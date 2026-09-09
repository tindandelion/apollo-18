# Ticket 19 performance analysis: high-density web rendering

Profiling of the release web showcase at the Ticket 13 high-density cap, with
measured what-if experiments for each candidate optimization. No renderer
changes are proposed as committed code here; every experiment described below
was reverted after measurement.

## Verdict

The frame is entirely CPU-bound inside the Wasm software renderer. The
presentation path costs 0.2 ms per frame and is irrelevant.

Reaching 30 FPS needs the frame to drop from about 63 ms to 33.3 ms, a 1.9×
speedup. Every output-preserving micro-optimization identified here, applied
together, lands at about 52 ms (19 FPS). An artificial build with *every*
transcendental function removed — including the `atan2`/`asin` that the
equirectangular lookup genuinely requires — still measures 42.8 ms scalar and
38.6 ms with `simd128` enabled.

**No amount of scalar micro-optimization reaches the target.** The fragment
loop has to be restructured to shade several fragments per iteration using
`simd128` lanes. The micro-optimizations are still worth taking, because they
reduce the work the vectorized loop has to carry.

The single largest change measured is precomputing complete terrain normals
per elevation texel, worth 15.2 ms — but it quantizes shading onto the
elevation grid and fails the lunar goldens by a wide margin. Even taking it,
the frame only reaches 48.5 ms (20.6 FPS). See
[Precomputing terrain normals](#precomputing-terrain-normals).

## Reference environment

- Apple Mac15,7, Apple M3 Pro (`arm64`, 12 cores), macOS 15.7.9 (24G830)
- Rust 1.97.1, Trunk 0.21.14, release profile with workspace defaults
- Playwright with bundled headless Chrome for Testing 151.0.7922.34
- 1440×900 CSS-pixel viewport at device pixel ratio 2
- 730.625×730.625 CSS-pixel canvas selecting the 1152×1152 backing resolution
- Measured 2026-09-09

Two independent noise scales matter when reading the numbers below. Within a
single build the frame time is extremely stable: the p25–p75 spread is
typically 0.3 ms. Across rebuilds the same source measures within about
±2 ms, apparently from code layout. Effects smaller than roughly 3 ms are
therefore not distinguishable from build-to-build variance, and are reported
as inconclusive rather than as wins.

## Baseline

The two committed browser tests agree with each other:

| Measurement | Result |
| --- | --- |
| `web-performance.spec.js` (8 s after 2 s warmup) | 15.99 FPS over 7.94 s |
| `web-timeline-performance.spec.js` | 63.05 ms render, 0.22 ms presentation, 15.81 FPS |

A staged profile that additionally times the `ImageData` construction
separately from `putImageData` attributes the frame as 63.68 ms of software
render, 0.01 ms of `ImageData` construction, and 0.20 ms of `putImageData`.
The `ImageData` cost is negligible because `wasm-bindgen` builds a
`Uint8ClampedArray` view over Wasm linear memory rather than copying the
5.3 MB framebuffer.

The canonical frame covers 843,700 fragments, 63.6% of the 1,327,104-pixel
framebuffer, matching the analytic π/4 × (0.9 × 1152)² disc area. That fixes
the per-fragment budget:

```text
current   62.5 ms / 843,700 fragments = 74 ns per fragment
30 FPS    33.3 ms / 843,700 fragments = 39 ns per fragment
```

For calibration, the identical 1152×1152 frame rendered by a native `arm64`
release binary takes 37.5 ms, or 26.7 FPS. Wasm is only about 1.65× slower
than native here. The gap to 30 FPS is therefore dominated by the cost of the
per-fragment algorithm, not by WebAssembly code generation.

## Where the time goes

Measured by progressively enabling stages of the real render path in the
release browser build. Each row is a separate build and measurement.

| Stage enabled | Frame (ms) | Δ from previous |
| --- | --- | --- |
| Framebuffer and depth-buffer allocation and clear only | 1.1 | 1.1 |
| plus octasphere generation | 1.8 | 0.7 |
| plus per-vertex projection and NDC validation | 1.6 | ~0 (within noise) |
| plus coverage scan and depth write, no shading | 6.4 | 4.6 |
| plus constant-color shading, depth test, sRGB encode, pixel write | 18.6 | 12.2 |
| plus the full lunar fragment shader | 62.5 | 43.9 |

Two conclusions follow immediately. The fragment shader body is 70% of the
frame. But the remaining 18.6 ms of scaffolding — scan, depth test, sRGB
encode, framebuffer write — is already 56% of the entire 33.3 ms budget, so a
successful optimization cannot leave that scaffolding untouched.

Substituting a truncating quantizer for the interpolated sRGB table in the
constant-color configuration drops 18.6 ms to 9.1 ms, attributing 9.4 ms of
the frame to sRGB output encoding alone.

## What the generated Wasm shows

The release Wasm has its name section stripped, so a CPU profile only yields
`wasm-function[N]`. Building the same crate with plain
`cargo build --target wasm32-unknown-unknown --release` retains names, and
disassembling the rasterization loop with `wasm-tools print` shows exactly
which calls survive inlining in the inner loop:

- `call $atan2f` and `call $asinf`, from the geographic conversion
- `call $LunarElevationMap::perturbed_radial`, which itself contains
  `call $hypotf`, `call $cosf`, 13 `f32.div`, and 8 bounds-check branches
- `call $LinearRgb::to_srgb8`, which contains three `call $roundf` and a
  `OnceLock` state check

That is **seven libm calls per fragment**, roughly 5.9 million per frame.
Each is a real call into compiled `libm` code, because WebAssembly has no
instructions for these functions.

The three `roundf` calls are the most surprising. WebAssembly has `f32.floor`
and `f32.nearest`, but Rust's `f32::round` uses round-half-away-from-zero,
which matches neither, so LLVM emits a libm call:

```120:121:crates/renderer/src/rasterizer/color.rs
    fn encode(&self, linear: f32) -> u8 {
        (self.interpolate(linear.clamp(0.0, 1.0)) * 255.0).round() as u8
```

The `cosf` is redundant rather than expensive-but-necessary. It recomputes a
quantity the caller already holds:

```58:58:crates/renderer/src/lunar_globe/lunar_elevation_map.rs
                longitude_derivative / (LUNAR_REFERENCE_RADIUS_KM * latitude.cos()),
```

Because the globe location is a unit vector whose `y` component is
`sin(latitude)`, `cos(latitude)` is exactly `sqrt(1 - y²)`, available with one
multiply and one `f32.sqrt` instruction.

## Measured what-if experiments

Each experiment is an independent build measured against the same-session
baseline of 64.2 ms. Negative deltas are improvements.

| Experiment | Frame (ms) | Δ | Preserves output? |
| --- | --- | --- | --- |
| Baseline | 64.2 | — | — |
| Replace `round` with truncate-and-compare | 58.1 | −6.1 | Yes, bit-exact |
| Replace `latitude.cos()` with `sqrt(1 − y²)` | 58.2 | −6.0 | Yes, within 1 ulp |
| Both of the above plus precomputed slope field | 53.0 | −11.2 | Yes |
| `lto = "fat"`, `codegen-units = 1` | 60.9 | −3.3 | Yes |
| Precompute per-texel elevation derivatives | 63.5 | −0.7 | Yes, inconclusive |
| Remove the depth buffer entirely | 63.1 | −1.1 | Yes, inconclusive |
| Unchecked elevation-map indexing | 62.5 | −1.7 | Yes, inconclusive |
| `#[inline(always)]` on the two non-inlined helpers | 64.3 | +0.1 | Yes, no effect |
| `-C target-feature=+simd128`, no code change | 65.9 | +1.7 | Yes, no gain |
| Replace `hypot` with `sqrt(x² + z²)` | 67.1 | +2.9 | Slower |
| Pack the linear color map to `[u16; 3]` | 68.1 | +3.9 | Slower |
| Scalar polynomial `atan2`/`asin` | 65.8 | +1.6 | Slower |
| Fixed color-map texel (removes lookup entirely) | 65.0 | +0.8 | No — ablation |
| Fixed elevation texel (removes 4 gathers) | 56.4 | −8.0 | No — ablation |
| Cheap linear stand-ins for `atan2`/`asin` | 53.8 | −10.4 | No — ablation |
| Every transcendental removed, scalar | 42.8 | −21.4 | No — ablation |
| Every transcendental removed, plus `simd128` | 38.6 | −25.6 | No — ablation |

### An independent replicate, and what it unsettles

An earlier sweep of the same masks ran to completion unnoticed while other
builds were competing for the machine, giving an unplanned replicate under CPU
contention. It is recorded in `.scratch/apollo-18/perf/results-replicate.tsv`.
Its baseline is 64.7–66.8 ms and its deltas are systematically smaller:

| Experiment | Clean Δ | Contended Δ |
| --- | --- | --- |
| Replace `round` with truncate-and-compare | −6.1 | −5.2 |
| Replace `latitude.cos()` with `sqrt(1 − y²)` | −6.0 | −2.9 |
| Precompute per-texel elevation derivatives | −0.7 | −1.6 |
| Replace `hypot` with `sqrt(x² + z²)` | +2.9 | −2.3 |
| Round plus cos plus slope field | −11.2 | −8.6 |

A third clean session (baseline 63.7 ms) settled the disagreement: the `round`
fix measured −4.6 ms and the `cos` fix −2.4 ms, and together only −5.1 ms
rather than the −7.0 that simple addition predicts. **Take −5 ms for the
`round` fix and −2.5 ms for the `cos` fix as the working figures; the −6.0 ms
originally reported for `cos` was the outlier of three measurements.** The
`hypot` result also flips sign between runs, which is further reason to leave
`hypot` alone rather than to reconsider it.

None of this changes the verdict. The contended replicate makes the scalar
micro-optimizations look *weaker*, not stronger, so the conclusion that they
cannot reach 33.3 ms is if anything better supported.

Three results deserve comment because they contradict reasonable expectations.

**The 25 MB linear color map is not a bottleneck.** `LunarColorMap` stores
2048×1024 texels as `[f32; 3]`, 25.2 MB, far beyond last-level cache, and it
is sampled with a scattered nearest-texel lookup. Pinning the lookup to a
single cached texel changed nothing, and halving the footprint to `[u16; 3]`
made the frame 3.9 ms *slower* because the unpack arithmetic costs more than
the cache traffic it saves. Screen-adjacent fragments map to
texel-adjacent addresses, so the access pattern is far more coherent than the
map size suggests. Do not spend effort compressing this map.

**Replacing `hypot` with inline arithmetic is slower**, reproducibly, by 2–5 ms
across two separate experiments. Removing the call appears to let LLVM inline
`perturbed_radial` into the already-large loop body, and the resulting register
pressure costs more than the call. This is a caution against assuming that
removing a libm call is always a win — `roundf` and `cosf` were, `hypotf` was
not.

**The elevation map's four scattered gathers cost about 8 ms**, but
precomputing them into a per-texel derivative field recovered only 0.7 ms. The
precomputed field is 8.3 MB and doubles the elevation working set, which
appears to give back most of what the removed arithmetic saves.

## Precomputing terrain normals

The elevation work can be moved out of the fragment loop in three progressively
more aggressive forms, which differ sharply in both payoff and fidelity. All
three were measured in a third session against a 63.7 ms baseline, with the
lunar goldens run for each.

| Variant | Frame (ms) | Δ | Golden result |
| --- | --- | --- | --- |
| Precompute raw per-texel derivatives | 63.5 | −0.7 | Passes, bit-exact |
| Precompute slopes with `1/(R·cos φ)` folded in | 58.3 | −5.4 | Fails: 18–44 outliers, max 3–10 codes |
| Precompute complete object-space normals | 48.5 | −15.2 | Fails: 745–1305 outliers, max 6–10 codes |

**The full normal field is the largest single win found anywhere in this
investigation** — 15.2 ms, more than the sRGB and `cos` fixes combined. The
fragment reduces to one gather, one matrix transform, and a dot product. It
removes the four scattered elevation gathers, `hypotf`, the tangent-frame
construction, roughly 13 divisions, the two multiply-subtracts that rebuild the
perturbed radial, and the final `normalize` — the precomputed normals are
already unit length and the object rotation is orthonormal, so renormalizing is
redundant. It also fully absorbs the `round` fix: normal field alone is 48.5 ms
and normal field plus fast rounding is 48.7 ms, so the two do not stack.

It is not free, and the cost is fidelity rather than speed. Today the normal is
assembled from the *fragment's* interpolated globe location and tangent frame,
with only the slopes taken from the nearest texel, so the dominant radial
component varies smoothly across each texel. A precomputed normal field freezes
the entire normal at texel centres, quantizing that smooth component onto the
0.25° elevation grid — about 1.4 px near disc centre. The goldens put the
resulting error at up to 10 sRGB codes across 745–1305 pixels against a budget
of 16. The amplified diff shows it is not a scatter of boundary hits but a
fine texel-scale striping spread across the whole lit crescent and
strengthening toward the terminator, where the Lambert gradient is steepest.
This is a visible appearance change and would have to be reviewed as one.

The middle variant is the interesting compromise. Folding the
`1/(R·cos(latitude))` factor into the precomputed field kills `cosf` *and* the
gathers together while leaving the smooth radial component intact, for 5.4 ms
— far better than the 0.7 ms of the bit-exact derivative field, and it stacks
cleanly with the `round` fix (53.4 ms combined, versus 5.4 + 4.6 predicted).
Its error is much smaller and much more localized: 18–44 outlier pixels against
the budget of 16, so it is only marginally outside the tolerance the project
already accepts. Recovering the last few pixels — most likely by keeping the
fragment's own `cos` near the poles, where the `1/cos` amplification makes the
texel-centre approximation worst — would plausibly bring it inside budget.

Enabling `simd128` on top of the normal field makes it *worse*, 52.5 ms against
48.5 ms, which is consistent with the other `simd128` results in this report.

Even so, the verdict does not move. The best measured configuration of any
kind is the normal field at 48.5 ms, or 20.6 FPS, against a 33.3 ms target.
Precomputing normals is a large and worthwhile win, but it buys about half the
distance and spends real image quality doing it.

## Options that were considered and rejected

**Night-side early-out.** A fragment whose geometric normal points far enough
away from the Sun is provably black regardless of terrain, so the geographic
conversion, both map lookups, and the lighting math can be skipped. The safe
threshold is set by the largest terrain tilt, which the elevation data puts at
0.66 (33.5°) below 80° latitude — but at 9.2 (83.8°) in the row adjacent to
the pole, because the eastward slope is divided by `cos(latitude)`. Using the
safe −0.55 threshold measured a median of 53.3 ms, but with a p25–p75 range of
42.5–65.6 ms: the saving scales with the dark fraction of the disc and is
exactly zero at full Moon. A sustained target cannot be met with an
optimization that disappears at the worst phase.

**WebAssembly threads.** Splitting the framebuffer across Web Workers is the
natural way to spend the machine's 12 cores, but `SharedArrayBuffer` requires
cross-origin isolation, and GitHub Pages cannot serve the necessary
`Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers. This
is closed off as long as the showcase is hosted on Pages.

**Reducing work per frame.** Lowering the backing resolution, reducing
octasphere subdivision, or skipping presentation are all excluded by the
ticket's performance contract.

## Recommended plan

### Step 1 — take the free wins

These are output-preserving and measured. Together they reach about 52 ms
(19 FPS), which does not meet the target but reduces the work the vectorized
loop must carry.

1. Replace `f32::round` in `SrgbEncodeTable::encode` with truncate-and-compare.
   For a non-negative input this is bit-identical to round-half-away-from-zero
   and removes three libm calls per fragment. Worth about 5 ms.
2. Derive `cos(latitude)` from the unit globe location's `y` component instead
   of calling `cosf`. Worth about 2.5 ms. Note that items 1 and 2 do not stack
   fully: together they measure −5.1 ms, not −7.5 ms.
3. Set `lto = "fat"` and `codegen-units = 1` in `[profile.release]`. Worth
   3.3 ms on the unmodified baseline, though it did not stack measurably with
   items 1 and 2.
4. Hoist octasphere generation out of the per-frame path. Worth only 0.7 ms,
   but rebuilding 8,192 triangles through a `HashMap` on every frame is
   indefensible on its own terms.

```76:76:crates/renderer/src/lunar_globe/octasphere.rs
    let mesh = generate(CANONICAL_SUBDIVISION_LEVEL);
```

### Step 2 — restructure the fragment loop for `simd128`

This is the only measured path that can close the remaining gap. The evidence
is that the all-transcendentals-removed floor is 42.8 ms scalar but 38.6 ms
once `simd128` is enabled: even code with nothing left to remove responds to
vectorization, and the scalar floor alone is above the 33.3 ms budget.

The approach is standard for software rasterizers: emit fragments in 2×2 quads
and hold four fragments in each `f32x4` lane through the whole shader —
barycentric interpolation, normalization, the geographic conversion, the
lighting dot product, and the sRGB encode. Only the two texture gathers stay
scalar. Points specific to this renderer:

- Vectorizing `atan2`/`asin` as branch-free four-wide polynomials should
  succeed where the scalar polynomial failed. The scalar attempt lost to libm
  because of its five branches; in lane form those become selects, and the
  cost is amortized across four fragments. The ablation shows 10.4 ms
  available here.
- The sRGB encode must be vectorized too, not just the shading. It is 9.4 ms
  today and roughly 3 ms after step 1, and it sits outside the shader in
  `write_if_nearer`.
- Quad rasterization also replaces the per-sample edge-function evaluation
  with incremental stepping, which addresses part of the 4.6 ms coverage scan.

Expect this to be a substantial change to `rasterizer::triangle` and the
`FragmentShader` interface, since the shader trait currently shades exactly
one fragment per call.

### Step 3 — decide how much image fidelity the target is worth

Two levers remain, and both trade accuracy for speed. They should be decided
deliberately rather than reached for once the budget gets tight.

The cos-folded elevation slope field is the better bargain: 5.4 ms, stacking
cleanly with step 1, for an error currently just outside the existing golden
tolerance at 18–44 outlier pixels. Tightening its polar handling would
plausibly bring it inside budget, at which point it belongs in step 1 instead.

The full precomputed normal field is worth 15.2 ms, the largest single win
measured, but it quantizes shading onto the elevation grid and produces visible
texel-scale striping across the lit crescent. Take it only as a deliberate
appearance change with re-reviewed goldens.

A third possibility, untested, is interpolating equirectangular texture
coordinates per triangle instead of computing `atan2`/`asin` per fragment. With
8,192 triangles each spanning about 1.4° under an orthographic projection the
error should be small, but it needs care at the antimeridian seam and the
poles. Note that the normal field does *not* remove `atan2`/`asin` — the texel
lookup still needs longitude and latitude — so this lever is independent of it
and the two could combine.

## Reproducing these measurements

The staged and what-if builds were produced by temporarily gating each variant
behind a `option_env!`-derived constant in the renderer, rebuilding with
`trunk build index.html --release`, and running a Playwright spec that reports
the median of 80 animation-frame callbacks after 20 warmup frames, with
`ImageData` construction and `putImageData` timed separately. Medians rather
than means are essential: means over 30 frames varied by several ms, while
medians over 80 frames reproduce to within 0.3 ms inside a build.

The raw results are in `.scratch/apollo-18/perf/results.tsv` alongside the
`measure.sh` driver. The probe scaffolding itself was reverted; the working
tree contains no measurement code.
