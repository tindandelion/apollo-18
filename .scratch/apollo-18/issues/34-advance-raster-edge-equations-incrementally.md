# 34: Advance raster edge equations incrementally

**What to build:** Reduce candidate-fragment raster work by advancing edge equations across rows and samples while preserving the software renderer's explicit edge-function coverage, barycentric interpolation, top-left ownership, and depth behavior.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** done

- [x] Edge values are initialized at a bounded traversal origin and advanced incrementally in both framebuffer axes.
- [x] Reciprocal-area multiplication is retained only if it preserves required interpolation and output behavior while measurably improving the representative workload.
- [x] Both input windings, degenerate and off-screen triangles, exact on-edge samples, adjacent shared edges in both draw orders, and strict depth ties remain covered by focused tests.
- [x] Exact triangle and cube goldens, realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [x] The retained diagnostic records candidate traversal and complete-frame effects; changes smaller than measurement noise or harmful to output are reverted and documented.
- [x] Tighter scanline spans are not introduced unless a new residual profile demonstrates that their additional coverage-rule risk is necessary.
- [x] The release browser performance contract, browser smoke coverage, and local quality gate pass apart from the still-open 30 FPS threshold if the cumulative work has not reached it yet.

## Comments

- 2026-09-10: Three warmed direct-evaluation runs measured 42.45–42.80 ms median complete-frame time. Incremental stepping alone measured 42.70–43.95 ms, so no isolated gain is claimed. Reciprocal normalization with direct edge evaluation regressed to 45.55–45.60 ms. Combining both changes measured 41.40–41.60 ms, only a 0.9 ms (2.1%) median improvement; the experimental sustained performance contract reached 23.13 FPS and remained below Ticket 19's still-open 30 FPS target. The arithmetic changes were reverted because that gain did not justify their floating-point accumulation risk and loop complexity. Full measurements are recorded in `docs/testing.md`.
- 2026-09-10: Standards and specification review found no blocking issues. The horizontal shared-edge regression test was retained. Focused rasterizer tests, all golden renders, native smoke coverage, browser smoke coverage, and `./scripts/dev/quality-gate.sh` passed.
