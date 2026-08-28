# 03: Render a rotating orthographic cube

**What to build:** Add the first complete 3D scene: a rotating colored cube rendered with the agreed left-handed coordinate convention, orthographic projection, back-face culling, and depth buffering in both native and web hosts.

**Blocked by:** 02: Interpolate triangle vertex colors

**Status:** ready-for-agent

- [ ] Object and view transformations use left-handed coordinates with `+Y` up and `+Z` forward.
- [ ] Orthographic projection and top-left viewport conversion produce the expected cube orientation and framing.
- [ ] Counter-clockwise front faces remain correct after viewport Y inversion.
- [ ] A depth buffer maps near to `0`, far to `1`, and lets smaller values win.
- [ ] Back-face culling and depth testing produce correct hidden-surface behavior from multiple cube views.
- [ ] Cube rotation is derived from explicit scene time rather than accumulated frame steps.
- [ ] A separate native cube binary renders deterministic frames, while the webpage animates the same scene.
- [ ] Exact cube goldens and focused transformation, winding, culling, and depth tests pass.
- [ ] Learning documentation explains the transform pipeline, orthographic projection, winding, and depth.
- [ ] The local quality gate passes.
