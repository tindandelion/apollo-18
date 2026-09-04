# Derive terrain normals on the lunar reference sphere

Shade the lunar globe with per-fragment terrain normals derived from the elevation map as a bump on the 1,737.4 km reference sphere, without displacing octasphere vertices. Slopes use that radius only, not the local radius `R + h`, because the rasterized surface is still the sphere; NASA's file is a displacement map in source terminology, but Apollo 18 treats it as a lunar elevation map for lighting. Gradients are 4-connected central differences of nearest-neighbor texels (longitude wrap, one-sided latitude and zero eastward slope on polar rows). The tangent frame comes from the fragment's object-space radial direction; only the elevation stencil is quantized. The object-space terrain normal is rotated with the globe before Lambertian lighting so geography stays attached to the surface while the Sun stays fixed in world space.

## Considered options

- **Local-radius bump (`R + h`)** and **virtual displaced-neighbor cross products** were rejected because they describe a surface that is never drawn.
- **Polar east–west from the adjacent latitude ring** was rejected as extra machinery for about two pixels at the top and bottom of the canonical disk.
- **Rebuilding `û` from the quantized texel** was rejected because it would facet lighting to the elevation grid twice; color lookup already uses interpolated radial direction and then nearest-neighbor sampling.
