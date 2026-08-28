# 02: Interpolate triangle vertex colors

**What to build:** Extend the first triangle into a visible demonstration of edge-function rasterization and barycentric interpolation, producing matching native and web images with smoothly interpolated vertex colors.

**Blocked by:** 01: Render the first triangle natively and on the web

**Status:** ready-for-agent

- [ ] Pixel-center edge functions determine triangle coverage within a bounded region.
- [ ] Normalized edge values produce barycentric weights that interpolate all three vertex colors affinely.
- [ ] Degenerate and off-screen triangles are handled without crashes or invalid framebuffer writes.
- [ ] Shared triangle edges follow one documented deterministic ownership rule without visible cracks or unstable overlap.
- [ ] The native triangle binary and webpage display the interpolated triangle through the shared frame-rendering seam.
- [ ] Exact goldens and focused tests cover coverage, barycentric weights, interpolation, degeneracy, and shared edges.
- [ ] Learning documentation explains edge functions, coverage, barycentric interpolation, and the shared-edge rule.
- [ ] The local quality gate passes.
