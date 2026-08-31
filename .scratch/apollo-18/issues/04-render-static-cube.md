# 04: Render a deterministic orthographic cube

**What to build:** Add Apollo 18's first deterministic 3D still: a colored cube transformed through the agreed left-handed coordinate convention and orthographic projection into the established NDC rasterization stage, then saved by a separate native binary.

**Blocked by:** 03: Add viewport conversion, back-face culling, and depth buffering

**Status:** ready-for-agent

- [ ] Object and view transformations use left-handed coordinates with `+Y` up and `+Z` forward.
- [ ] Orthographic projection maps the visible cube into the validated NDC position range with the expected orientation and framing.
- [ ] Cube geometry is clockwise when front-facing before viewport conversion and integrates with the established back-face culling and depth testing behavior.
- [ ] Back-face culling and depth testing produce correct hidden-surface behavior from multiple fixed cube views.
- [ ] A fixed cube pose renders through the existing deterministic frame-rendering seam.
- [ ] A separate native cube binary writes the deterministic cube frame as a valid 800×800 PNG.
- [ ] An exact cube golden and focused object, view, orthographic projection, winding, and multi-view hidden-surface tests pass.
- [ ] `docs/learning/04-transforming-a-cube.md` explains object and view transformations, the left-handed convention, orthographic projection, and how the cube enters the established NDC pipeline.
- [ ] The local quality gate passes.
