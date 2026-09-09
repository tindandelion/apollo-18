# Ticket 19 synthesized performance analysis

## Summary

The two performance reports agree on the central diagnosis:

- Current performance is about **63 ms/frame, or 15.8 FPS**.
- Canvas presentation and framebuffer handoff cost only about **0.2 ms/frame**.
- Approximately **70% of the frame is lunar fragment shading**.
- Raster traversal and framebuffer encoding are also material.
- Build flags, caching, and other setup optimizations cannot independently reach 30 FPS.
- Reaching 30 FPS requires almost a **2× throughput improvement**.

The primary performance contract was reproduced with:

```text
./scripts/dev/web-performance-test.sh
1152×1152
15.85 FPS
Expected: >= 30 FPS
```

## Resolving the apparent contradictions

### Rasterization versus shading priority

The GPT report recommends raster traversal first because native sampling attributed roughly 32% to `rasterize_fragments`. The Opus staged experiment is more useful for estimating the potential gain:

- Coverage scan without shading: approximately **4.6 ms**
- Full lunar shader contribution: approximately **44–45 ms**
- Constant-color shading, depth, sRGB encoding, and writes: approximately **12 ms**

Therefore:

- Incremental edges and tighter traversal are worthwhile.
- They are unlikely to recover the required 30 ms by themselves.
- Fragment shading and output encoding must remain the primary optimization front.

### Whether SIMD is proven necessary

The Opus report overstates the conclusion slightly. It demonstrates that:

- scalar micro-optimizations tested so far do not reach 30 FPS;
- merely enabling `simd128` does not help;
- deliberately SIMD-friendly code is likely needed.

It does **not** yet measure an explicit four-fragment SIMD implementation. Consequently, SIMD is the leading architectural hypothesis, not yet a proven solution.

### Native timing discrepancy

The reports give approximately **37.5 ms** and **45.7 ms** for native rendering. This difference is much larger than the reported measurement noise and should not be used for budgeting until the native harnesses are reconciled.

It does not affect the immediate decision: both native measurements show that substantial algorithmic work remains, and the browser contract is authoritative.

## Recommended course of action

### Phase 1: Establish the retained measurement loop

Before optimization:

1. Create a dedicated Ticket 19 feature branch.
2. Keep `./scripts/dev/web-performance-test.sh` as the acceptance signal.
3. Add or retain a diagnostic browser measurement that reports medians for:
   - complete render;
   - `ImageData` construction;
   - `putImageData`;
   - completed FPS.
4. Measure each experiment with:
   - a baseline immediately before or after it;
   - three warmed runs;
   - output and golden validation.
5. Treat changes below roughly **3 ms across separate builds as inconclusive**.

The browser measurement is primary; native profiling is for locating code, not deciding whether the ticket passes.

### Phase 2: Take bounded, output-preserving scalar wins

Implement these independently, measure each, and retain only demonstrated improvements.

#### 1. Remove `roundf` from sRGB encoding

Replace non-negative `f32::round()` with the measured truncate-and-compare implementation.

- Expected gain: approximately **5 ms**
- Reported as bit-exact
- Low implementation risk
- Add exhaustive tests around quantization thresholds

This is the strongest immediate change.

#### 2. Move lighting into object space

Transform the Sun direction into lunar-globe object space once per frame instead of transforming every fragment's terrain normal into world space.

Rotation preserves dot products and lengths, so this should preserve lighting semantics while removing a per-fragment matrix transform.

This was recommended but not directly measured in either report and should be tested early.

#### 3. Reuse the tangent frame's horizontal radius

The tangent-frame calculation already obtains:

```text
sqrt(x² + z²) = cos(latitude)
```

Pass or reuse that value when calculating physical terrain slopes rather than calling `latitude.cos()`.

- Expected gain: approximately **2.5 ms in isolation**
- Known not to stack fully with the rounding change
- Must pass terrain-normal and lunar golden tests

Do not replace `hypot` itself: the measured inline alternative regressed performance.

#### 4. Skip color lookup and sRGB encoding for exact black

After calculating the terrain normal and Lambertian intensity, if the resulting intensity is exactly zero, write opaque black directly.

This preserves ADR-0005 because the decision uses the terrain normal, not a geometric day-side approximation. Its benefit will vary with lunar phase, so evaluate it over the complete performance window rather than a hand-picked crescent.

#### 5. Test incremental edge stepping

Replace repeated edge-function reconstruction with row-origin evaluation and X/Y increments. Also test reciprocal-area multiplication.

Do **not** begin with tighter scanline spans: those carry greater top-left coverage risk, while staged evidence limits the likely traversal payoff.

Validate especially:

- shared-edge ownership;
- winding and culling;
- exact triangle and cube goldens;
- lunar goldens;
- deterministic native output.

Floating-point accumulation may alter edge equality or interpolation, so this optimization is not automatically output-preserving.

### Phase 3: Reprofile the optimized scalar renderer

After Phase 2, obtain a new stage profile.

The expected result is probably around **45–52 ms/frame**, still short of 30 FPS. The profile determines what enters the SIMD kernel:

- coverage and barycentric interpolation;
- globe-location interpolation and normalization;
- terrain-normal arithmetic;
- lighting;
- sRGB encoding.

Do not spend time at this stage on:

- color-map compression;
- removing the depth buffer;
- unchecked indexing;
- replacing `hypot`;
- global `simd128`;
- mesh caching;
- plain `wasm-opt`;
- framebuffer or Canvas changes.

All were measured as small, inconclusive, or regressive.

### Phase 4: Build explicit four-fragment SIMD incrementally

If the scalar renderer remains above budget, restructure around four-fragment batches.

Recommended sequence:

1. Introduce a four-fragment shader batch with a scalar fallback.
2. Process 2×2 fragment quads while preserving exact per-lane coverage and depth behavior.
3. Vectorize:
   - barycentric interpolation;
   - depth interpolation and comparison;
   - globe-location interpolation;
   - normalization;
   - tangent and lighting arithmetic;
   - sRGB interpolation and quantization.
4. Initially keep lunar-map gathers and `atan2`/`asin` scalar per lane.
5. Measure before introducing approximate SIMD transcendental functions.
6. Only then investigate branch-free four-wide `atan2`/`asin`.

This avoids combining a large rasterizer rewrite with changed lunar-coordinate mathematics in one experiment.

The existing `FragmentShader` interface shades one fragment at a time, so this will require an intentional interface change. Prefer a batch method with a scalar default rather than a lunar-specific bypass of the software renderer.

### Phase 5: Use fidelity trades only as a final contingency

The current evidence says:

- **Precomputed complete normals:** reject. They visibly quantize shading and conflict with ADR-0003's smooth fragment-local tangent frame.
- **Cos-folded slope field:** promising but currently fails lunar golden tolerance. Revisit only if polar handling can make it pass unchanged goldens.
- **Interpolated lunar coordinates:** defer. This risks antimeridian and polar artifacts and conflicts with the explicit per-fragment globe-location lookup decision.
- **Resolution, subdivision, map detail, skipped frames, GPU:** prohibited by Ticket 19.

A fidelity-changing optimization should require an explicit design decision, not be introduced as performance tuning.

## Ranked hypotheses

1. **Per-fragment numerical and encoding work is the dominant bottleneck.**  
   Prediction: exact scalar reductions provide measurable gains, and explicit four-wide shading produces the largest remaining improvement.

2. **Raster candidate arithmetic is a secondary but necessary contributor.**  
   Prediction: incremental edges improve the complete frame by several milliseconds, but cannot independently reach 30 FPS.

3. **Phase-dependent exact-black work elimination provides a useful supplemental gain.**  
   Prediction: improvement correlates with the unlit fraction while preserving terrain-normal rim highlights.

4. **Setup, allocation, mesh generation, and post-link optimization are tail work.**  
   Prediction: each remains around 0–3 ms and matters only near the final budget.

## Practical target ladder

| Milestone | Target |
| --- | ---: |
| Current baseline | ~63 ms |
| Exact scalar/data-flow changes | ≤50 ms |
| Raster stepping improvements | ≤46 ms |
| Explicit four-wide processing | ≤35 ms |
| Final build/post-link tuning | **≤33.3 ms / ≥30 FPS** |

## Conclusion

Take the small exact wins first, measure incremental rasterization, and then pursue explicit SIMD against the new residual profile. A raster-only plan is not supported by the staged measurements, while jumping immediately to approximate SIMD would combine too much performance and correctness risk.
