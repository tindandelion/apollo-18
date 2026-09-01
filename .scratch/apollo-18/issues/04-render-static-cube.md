# 04: Render a deterministic orthographic cube

**What to build:** Add Apollo 18's first deterministic 3D still: a colored cube transformed through the agreed left-handed coordinate convention and orthographic projection into the established NDC rasterization stage, then saved by a separate native binary.

**Blocked by:** 03: Add viewport conversion, back-face culling, and depth buffering

**Status:** done

## Settled design

- The canonical scene uses a unit cube centered at the world origin, with corners at `±0.5`.
- The camera is at `(0, 0, -3)`, looks along `+Z`, and uses `+Y` as up.
- The cube rotates `+30°` around Y and then `-20°` around X.
- The symmetric orthographic volume spans `[-1.25, 1.25]` horizontally and vertically, with near depth `2` and far depth `4`.
- Each face has one solid sRGB color and duplicates vertices where needed: front (`-Z`) red, back (`+Z`) cyan, right (`+X`) green, left (`-X`) magenta, top (`+Y`) blue, and bottom (`-Y`) yellow.
- The shared renderer adds a separate `render_cube(width, height)` function and retains `render_triangles`; a scene-selecting interface is not introduced.
- The exact cube golden uses the canonical 800×800 dimensions.
- Focused hidden-surface tests use mirrored `+30°` and `-30°` yaw views, asserting that the expected front/right/top or front/left/top face colors appear while opposite faces remain hidden. Only the canonical `+30°` view receives a golden.
- The web host remains on the triangle until ticket 05 evolves it to the animated cube.

## Acceptance criteria

- [x] Object and view transformations use left-handed coordinates with `+Y` up and `+Z` forward.
- [x] Orthographic projection maps the visible cube into the validated NDC position range with the expected orientation and framing.
- [x] Cube geometry is clockwise when front-facing before viewport conversion and integrates with the established back-face culling and depth testing behavior.
- [x] Back-face culling and depth testing produce correct hidden-surface behavior from multiple fixed cube views.
- [x] A fixed cube pose renders through the existing deterministic frame-rendering seam.
- [x] A separate native cube binary writes the deterministic cube frame as a valid 800×800 PNG.
- [x] An exact cube golden and focused object, view, orthographic projection, winding, and multi-view hidden-surface tests pass.
- [x] `docs/learning/04-transforming-a-cube.md` explains object and view transformations, the left-handed convention, orthographic projection, and how the cube enters the established NDC pipeline.
- [x] The local quality gate passes.
