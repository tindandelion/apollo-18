# 35: Reprofile the optimized scalar renderer

**What to build:** Establish the cumulative high-density browser result after the retained scalar optimizations and identify the measured residual work that must be addressed to reach 30 FPS.

**Blocked by:** 30: Remove WebAssembly rounding calls from sRGB encoding; 31: Calculate lunar illumination in object space; 32: Reuse the terrain tangent frame's horizontal radius; 33: Bypass downstream work for exactly unlit fragments; 34: Advance raster edge equations incrementally

**Status:** ready-for-agent

- [ ] Three warmed release-browser runs record sustained FPS and median complete-frame and stage timings at 1152×1152.
- [ ] A runtime profile attributes the remaining cost among raster traversal, depth and framebuffer output, lunar coordinate derivation, lunar map sampling, terrain-normal derivation, and illumination.
- [ ] The cumulative contribution of each retained scalar optimization is recorded without claiming effects below measurement noise.
- [ ] The remaining frame-time reduction required to reach 33.3 ms is calculated.
- [ ] The proposed four-fragment SIMD scope is limited to measured residual bottlenecks and identifies which operations remain scalar initially.
- [ ] All renderer tests, realistic lunar goldens, browser smoke coverage, and the local quality gate pass.
