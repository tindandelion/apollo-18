# 03: Render a deterministic orthographic cube

**What to build:** Add Apollo 18's first deterministic 3D still: a colored cube rendered through the shared software renderer with the agreed left-handed coordinate convention, orthographic projection, back-face culling, and depth buffering, and saved by a separate native binary.

**Blocked by:** 02: Interpolate triangle vertex colors

**Status:** ready-for-agent

- [ ] Object and view transformations use left-handed coordinates with `+Y` up and `+Z` forward.
- [ ] Orthographic projection and top-left viewport conversion produce the expected cube orientation and framing.
- [ ] Counter-clockwise front faces remain correct after viewport Y inversion.
- [ ] A depth buffer maps near to `0`, far to `1`, and lets smaller values win.
- [ ] Back-face culling and depth testing produce correct hidden-surface behavior from multiple fixed cube views.
- [ ] A fixed cube pose renders through the existing deterministic frame-rendering seam.
- [ ] A separate native cube binary writes the deterministic cube frame as a valid 800×800 PNG.
- [ ] An exact cube golden and focused transformation, winding, culling, and depth tests pass.
- [ ] Learning documentation explains the transform pipeline, orthographic projection, winding, and depth.
- [ ] The local quality gate passes.
