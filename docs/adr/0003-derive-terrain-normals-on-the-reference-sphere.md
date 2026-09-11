# Precompute terrain normals on the lunar reference sphere

Shade the lunar globe with terrain normals derived from the elevation map as a bump on the 1,737.4 km reference sphere, without displacing octasphere vertices. When a `LunarElevationMap` is constructed, derive and normalize one object-space terrain normal at every elevation-texel center, then discard the source elevation samples. Slopes use the reference radius only, not the local radius `R + h`, because the rasterized surface is still the sphere. Gradients are 4-connected central differences of nearest-neighbor texels, with longitude wrapping, one-sided latitude differences on polar rows, and zero eastward slope on those rows.

Fragment shading uses lunar coordinates to select one cached normal with the existing nearest-texel policy. It does not rebuild the tangent frame at the fragment's interpolated globe location. Transform the world-space Sun direction into lunar-globe object space once per frame and evaluate Lambertian illumination against the cached object-space normal. This is mathematically equivalent to rotating each normal into world space while avoiding a transformation for every fragment.

## Considered options

- **Exact per-fragment tangent frames** preserved smoother normals within each elevation texel but repeated elevation gathers, physical-slope derivation, tangent-frame construction, normalization, and object-to-world transformation for every fragment. It did not meet the high-density browser performance target.
- **Cached elevation gradients** preserved the exact per-fragment convention but did not remove enough work to meet the target.
- **Local-radius bumps (`R + h`)** and **virtual displaced-neighbor cross products** were rejected because they describe a surface that is never drawn.
- **Polar east–west slopes from the adjacent latitude ring** were rejected as extra machinery for the top and bottom rows. Polar rows instead use zero eastward slope.

The cached convention intentionally quantizes terrain-normal orientation to elevation texel centers. Review found the small resulting lighting differences acceptable in exchange for meeting the 1152×1152 Wasm rendering target. The lunar color map, elevation-map resolution, octasphere geometry, and spherical silhouette remain unchanged.
