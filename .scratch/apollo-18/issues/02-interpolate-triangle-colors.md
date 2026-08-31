# 02: Interpolate triangle vertex colors

**What to build:** Extend the first triangle into a visible demonstration of edge-function rasterization and barycentric interpolation. Assign sRGB red, green, and blue to the existing vertices, interpolate them in linear RGB, and produce matching sRGB native and web images.

**Blocked by:** 01: Render the first triangle natively and on the web

**Status:** done

- [x] Direct pixel-center edge-function evaluation determines triangle coverage within a bounded region.
- [x] The same three edge values used for coverage are normalized by triangle area into barycentric weights that affinely interpolate all three vertex colors.
- [x] External sRGB vertex colors are decoded to linear RGB before interpolation, and final linear colors are clamped, sRGB-encoded, and rounded to the nearest 8-bit framebuffer values with alpha `255`.
- [x] The rasterizer accepts both input windings, preserves each color's vertex association, and skips exactly zero-area triangles without writes.
- [x] Fully and partially off-screen triangles, including extreme finite coordinates, are handled without crashes, unbounded traversal, or invalid framebuffer writes.
- [x] Shared triangle edges follow the top-left ownership rule without cracks or unstable overlap; differently colored adjacent triangles produce identical output in either draw order.
- [x] The native triangle binary and webpage display the interpolated triangle through the unchanged shared frame-rendering seam.
- [x] The existing `first_triangle.png` fixture is explicitly updated, and focused tests cover edge values, barycentric weights, interpolation, the piecewise sRGB transfer function, both windings, degeneracy, off-screen coverage, and shared edges.
- [x] Tests distinguish linear-light interpolation from encoded-sRGB interpolation, including a half-intensity linear channel that encodes near sRGB byte `188` rather than `128`.
- [x] `docs/learning/02-barycentric-interpolation.md` explains edge functions, coverage, barycentric interpolation, linear RGB interpolation, sRGB output encoding, and the top-left ownership rule with concrete shared-edge examples; the first guide links to it without duplicating the full treatment.
- [x] ADR 0002 records sRGB as the external representation, linear RGB as the renderer's working representation, and the renderer's input/output conversion boundaries.
- [x] The local quality gate passes.
