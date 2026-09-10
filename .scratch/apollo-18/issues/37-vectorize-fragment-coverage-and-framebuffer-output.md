# 37: Vectorize fragment coverage and framebuffer output

**What to build:** Use explicit Wasm SIMD for measured four-fragment raster and framebuffer operations while preserving the scalar implementation for native rendering and operations that do not benefit from vectorization.

**Blocked by:** 36: Add four-fragment rendering with a scalar fallback

**Status:** ready-for-agent

- [ ] SIMD is applied only to operations identified by the residual profile, such as direct edge evaluation and coverage, barycentric or depth interpolation, depth comparison, and sRGB output processing.
- [ ] Per-lane top-left coverage and strict depth semantics match the scalar implementation, including partially covered batches and shared edges.
- [ ] Native rendering and supported non-SIMD paths retain correct scalar behavior.
- [ ] The release Wasm contains the intended SIMD instructions without relying on ineffective global autovectorization.
- [ ] Exact triangle and cube goldens, realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [ ] Current desktop Chrome, Firefox, and Safari compatibility remains supported.
- [ ] The retained diagnostic demonstrates a repeatable complete-frame improvement larger than measurement noise.
- [ ] The release browser performance contract, browser smoke coverage, and local quality gate pass apart from the still-open 30 FPS threshold if lunar shading still prevents it.

## Comments

- 2026-09-10: Ticket 35's residual profile confirmed raster traversal and depth/framebuffer output as appropriate SIMD targets. “Edge advancement” was narrowed to direct edge evaluation and coverage so this ticket does not reopen Ticket 34's rejected incremental edge-stepping experiment or its floating-point accumulation risk.
