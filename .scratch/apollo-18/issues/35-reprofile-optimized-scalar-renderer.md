# 35: Reprofile the optimized scalar renderer

**What to build:** Establish the cumulative high-density browser result after the retained scalar optimizations and identify the measured residual work that must be addressed to reach 30 FPS.

**Blocked by:** 30: Remove WebAssembly rounding calls from sRGB encoding; 31: Calculate lunar illumination in object space; 32: Reuse the terrain tangent frame's horizontal radius; 33: Bypass downstream work for exactly unlit fragments; 34: Advance raster edge equations incrementally

**Status:** done

- [x] Three warmed release-browser runs record sustained FPS and median complete-frame and stage timings at 1152×1152.
- [x] A runtime profile attributes the remaining cost among raster traversal, depth and framebuffer output, lunar coordinate derivation, lunar map sampling, terrain-normal derivation, and illumination.
- [x] The cumulative contribution of each retained scalar optimization is recorded without claiming effects below measurement noise.
- [x] The remaining frame-time reduction required to reach 33.3 ms is calculated.
- [x] The next optimization investigation is bounded by measured residual bottlenecks and identifies which operations remain scalar.
- [x] All renderer tests, realistic lunar goldens, browser smoke coverage, and the local quality gate pass.

## Comments

- 2026-09-10: Three warmed release-browser runs measured 22.12–22.80 completed FPS and 42.4–43.9 ms median complete-frame time at 1152×1152. The median result was 22.73 FPS and 42.8 ms, leaving 9.47 ms (22.1%) to reach the 33.33 ms budget. A separate performance-contract run measured 22.42 FPS and failed only its intentionally still-open 30 FPS assertion.
- 2026-09-10: A release-browser CPU profile and source-attributed companion native profile identified raster traversal (28.6%), lunar-coordinate derivation (22.7%), terrain-normal derivation (22.5%), lunar-map sampling (10.6%), illumination (8.3%), and depth/framebuffer output (7.3%) as the directional residual split. `docs/testing.md` records the methodology, cumulative scalar evidence, and measurement limits.
- 2026-09-10: Standards and specification review found no blocking issues. Focused realistic goldens, browser smoke coverage, and `./scripts/dev/quality-gate.sh` passed.
