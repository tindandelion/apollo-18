# 34: Advance raster edge equations incrementally

**What to build:** Reduce candidate-fragment raster work by advancing edge equations across rows and samples while preserving the software renderer's explicit edge-function coverage, barycentric interpolation, top-left ownership, and depth behavior.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** ready-for-agent

- [ ] Edge values are initialized at a bounded traversal origin and advanced incrementally in both framebuffer axes.
- [ ] Reciprocal-area multiplication is retained only if it preserves required interpolation and output behavior while measurably improving the representative workload.
- [ ] Both input windings, degenerate and off-screen triangles, exact on-edge samples, adjacent shared edges in both draw orders, and strict depth ties remain covered by focused tests.
- [ ] Exact triangle and cube goldens, realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [ ] The retained diagnostic records candidate traversal and complete-frame effects; changes smaller than measurement noise or harmful to output are reverted and documented.
- [ ] Tighter scanline spans are not introduced unless a new residual profile demonstrates that their additional coverage-rule risk is necessary.
- [ ] The release browser performance contract, browser smoke coverage, and local quality gate pass apart from the still-open 30 FPS threshold if the cumulative work has not reached it yet.
