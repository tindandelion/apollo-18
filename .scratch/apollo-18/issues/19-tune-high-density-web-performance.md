# 19: Tune high-density web rendering performance

**What to build:** Precompute one normalized object-space terrain normal per lunar-elevation-map texel and use that cache to make the representative web lunar globe sustain at least 30 FPS at the color-map-bounded high-density resolution introduced by Ticket 13. Preserve output resolution, source-map resolution, lunar appearance, and the Canvas 2D presentation path; the reviewed texel-center normal quantization is the accepted quality tradeoff.

**Blocked by:** 13: Use color-map-bounded high-density web rendering

**Status:** ready-for-agent

## Settled direction

The user-reviewed baseline, precomputed-normal, and amplified-difference comparisons established that the quality reduction from texel-center terrain normals is acceptable. Implement the measured 30 FPS prototype as the production path:

- Build one normalized object-space terrain normal for every 1440×720 lunar elevation texel when `LunarElevationMap` is constructed.
- Derive each cached normal at the texel center using the existing lunar reference radius, elevation stencil, longitude wrapping, latitude edge behavior, and tangent-frame equations.
- Retain only the dimensions and cached normals after construction; per-fragment shading no longer needs the source elevation samples.
- Select the cached normal with the existing nearest-texel lunar-coordinate policy.
- Transform the world-space Sun direction into lunar-globe object space once per frame, then evaluate Lambertian illumination directly against the cached object-space normal.
- Remove per-fragment elevation gathers, physical-slope derivation, tangent-frame construction, perturbed-radial normalization, and object-to-world normal transformation.
- Treat the resulting appearance as an intentional rendering change. Regenerate and review the realistic lunar goldens only after the implementation and performance result are otherwise complete; those fixtures then become the new standard.

Update ADR-0003 and the terrain-normal learning guide to describe this convention rather than the former exact per-fragment tangent frame.

## Implementation breakdown

Ticket 19 retains the results of the completed investigation and scalar-optimization sequence:

- 29: Retain high-density browser performance diagnostics
- 30: Remove WebAssembly rounding calls from sRGB encoding
- 31: Calculate lunar illumination in object space
- 32: Reuse the terrain tangent frame's horizontal radius
- 33: Bypass downstream work for exactly unlit fragments
- 34: Advance raster edge equations incrementally
- 35: Reprofile the optimized scalar renderer

Continue performance investigation and implementation directly under this ticket. The previously planned four-fragment vectorization sequence was removed from the backlog because it was not considered the right direction.

## Performance contract

- The workload is the representative lit lunar scene rendered at the 1152-pixel high-density cap selected by Ticket 13.
- Success is sustained throughput of at least 30 completed, rendered, and presented frames per second after warmup in the repeatable release browser performance test.
- The reference machine, operating system, browser and version, viewport, device pixel ratio, selected backing resolution, warmup duration, measurement duration, baseline, and final result are recorded together.
- The target must be reached through the settled terrain-normal precomputation and any additional measured CPU/Wasm or presentation-path optimization. Lowering the selected backing resolution, reducing octasphere subdivision or source lunar-map resolution, skipping render or presentation work for counted frames, or introducing WebGL/WebGPU does not satisfy the ticket.

## Available investigation tools

The reference development machine currently has the following local tools. These are investigation aids, not project build dependencies:

- The project-local Playwright 1.62.1 installation and its bundled Chrome for Testing 151.0.7922.34 can run the release performance contract and collect Chrome DevTools Protocol CPU profiles, timeline traces, long tasks, garbage-collection events, and runtime metrics.
- Browser User Timing and `performance.now()` can measure the software render, `ImageData` construction, `putImageData`, and complete animation-frame boundaries without changing the counted work.
- Rust 1.97.1 and Cargo can run a focused native harness for the same 1152×1152 `render_lunar_globe` workload, separating shared-renderer cost from Wasm transfer and Canvas 2D presentation.
- `cargo-flamegraph` 0.6.14 can produce native sampling flame graphs; `cargo-instruments` 0.4.10 and macOS `sample` provide alternative native profiling paths.
- `wasm-tools` 1.258.0 can validate and inspect the release module, including function indexes, imports and exports, generated instructions, metadata, and SIMD usage after runtime profiling identifies where to look.
- Binaryen `wasm-opt` 132 can provide controlled original-versus-post-optimized Wasm experiments, static metrics, feature inspection, and exploratory SIMD/autovectorization probes. Every variant must be checked for both FPS and rendering correctness.
- The renderer tests, golden fixtures, browser smoke tests, ticket-specific performance test, and `./scripts/dev/quality-gate.sh` can verify that retained optimizations preserve required behavior.

Optional tools such as Samply or Hyperfine may be useful if the available profilers or measurement loops prove insufficient. The agent must explain the need and receive the user's explicit approval before installing any additional tool. Standalone `wasm2wat` is unnecessary while `wasm-tools print` supplies the required text-format inspection.

- [x] The capped high-density baseline is reproduced and profiles identify terrain-normal derivation as a material residual cost.
- [x] Temporary prototypes bound both choices: cached gradients preserve existing goldens but are insufficient, while cached texel-center normals plus object-space illumination pass the 30 FPS contract with an accepted visual difference.
- [ ] `LunarElevationMap` precomputes one finite, normalized object-space terrain normal per source texel using the settled reference-sphere gradient and edge rules, and does not retain source elevation samples after construction.
- [ ] Focused Arrange-Act-Assert tests cover flat and sloped terrain, antimeridian wrapping, polar rows, cache dimensions, unit normals, deterministic construction, nearest-texel selection, and object-space illumination under lunar-globe rotation.
- [ ] Fragment shading samples the terrain-normal cache and performs no elevation stencil sampling, physical-slope derivation, tangent-frame construction, perturbed-radial normalization, or object-to-world normal transformation.
- [ ] The world-space Sun direction is transformed into lunar-globe object space once per frame without changing phase, libration, position angle, or exactly-unlit behavior.
- [ ] The release browser performance test verifies at least 30 completed, rendered, and presented frames per second at the 1152-pixel cap after warmup; instrumentation still requires exactly one software render and Canvas 2D presentation per counted frame.
- [ ] The output preserves the resolution-selection policy, lunar orientation and phase, source lunar color-map and elevation-map resolution, octasphere subdivision, terrain-lighting behavior apart from the accepted texel-center quantization, and Canvas 2D presentation path.
- [ ] After implementation and performance review, the six realistic lunar golden fixtures are explicitly regenerated, visually reviewed as the new standard, and pass normally without the update environment variable; exact triangle and cube goldens remain byte-identical.
- [ ] Native output dimensions and deterministic native output remain unchanged, and browser smoke coverage passes at device pixel ratio 2 and after responsive resizing.
- [ ] ADR-0003, the terrain-normal learning guide, and performance documentation record the new sampling convention, equations, memory/startup cost, reference environment, baseline, final result, and accepted quality tradeoff.
- [ ] The local quality gate, browser smoke tests, realistic lunar goldens, and ticket-specific release browser performance test pass.

## Comments

- 2026-09-11: Terrain precomputation prototypes found two viable bounds. Caching elevation gradients preserved all goldens and reduced paired median complete-frame time by 5.17 ms (10.7%) in the stabilized alternating run set, although an earlier cooler set showed only 1.55 ms with overlapping ranges. Caching texel-center terrain normals and moving illumination to object space reduced a representative cooler median from 43.45 to 31.7 ms and passed the sustained browser contract at 30.13 FPS, but all six realistic lunar goldens exceeded their budgets (499–1,641 outlier pixels; maximum RGB difference 6–10). The prototypes were removed after measurement; full methodology, memory and startup costs are recorded in `docs/testing.md`.
- 2026-09-11: The visual comparisons were reviewed and the texel-center normal quality reduction was accepted. Precomputed terrain normals plus once-per-frame object-space Sun transformation are now the settled Ticket 19 implementation direction. Golden regeneration is deliberately deferred until that implementation is complete and reviewed.
