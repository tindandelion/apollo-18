# 03: Add viewport conversion, back-face culling, and depth buffering

**What to build:** Evolve the triangle showcase into a deterministic demonstration of the post-projection pipeline: validated normalized device coordinates, top-left viewport conversion, back-face culling, and depth-buffered overlapping triangles.

**Blocked by:** 02: Interpolate triangle vertex colors

**Status:** done

- [x] `NdcVertex` stores its position as a private `Vec3` and has a checked constructor that rejects non-finite values, `x` or `y` outside `[-1, 1]`, and depth outside `[0, 1]` without clamping or panicking.
- [x] Explicit `NdcVertex` and `ScreenVertex` types identify the coordinate space on each side of viewport conversion; no unqualified vertex type crosses that boundary.
- [x] Viewport conversion maps NDC with `+Y` up into continuous top-left framebuffer coordinates using `screen_x = (ndc_x + 1) * width / 2` and `screen_y = (1 - ndc_y) * height / 2`.
- [x] Clockwise NDC triangles are front-facing; after viewport Y inversion, positive screen-space area identifies those front faces while non-positive area culls counter-clockwise, degenerate, and precision-collapsed triangles without reordering accepted vertices.
- [x] An internal rasterizer owns the output framebuffer and a same-sized depth buffer, while the returned framebuffer remains tightly packed RGBA with no depth storage in its presentation seam.
- [x] The depth buffer starts at positive infinity, accepts a fragment only when its normalized depth is strictly smaller than the stored depth, and therefore permits the complete near-`0` through far-`1` range with deterministic first-fragment ownership on exact ties.
- [x] Normalized depth is interpolated affinely from screen-space barycentric weights and rejected fragments change neither color nor stored depth.
- [x] The shared `render_triangles` scene, retained native `triangle` binary, and webpage display partially overlapping near and far triangles submitted in depth-challenging order plus an obviously placed back-facing triangle; interpolated vertex colors remain visible.
- [x] The existing triangle golden is explicitly updated, and focused tests cover NDC validation, viewport corners and Y inversion, front-face and degenerate culling, depth interpolation, near/far limits, exact ties, occlusion independent of submission order, and rejected-fragment writes.
- [x] `docs/learning/03-viewport-culling-and-depth.md` explains NDC, viewport conversion, winding reversal, back-face culling, affine orthographic depth, and strict-less depth testing; the earlier triangle guides are updated where they describe the evolved scene or its coordinate inputs.
- [x] The local quality gate passes.
