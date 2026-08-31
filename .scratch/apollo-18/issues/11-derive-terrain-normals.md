# 11: Derive terrain normals from lunar elevation

**What to build:** Reveal crater and terrain relief by deriving per-fragment terrain normals from NASA's physical lunar elevation data while leaving the octasphere geometry and silhouette spherical.

**Blocked by:** 10: Light and rotate the lunar globe

**Status:** ready-for-agent

- [ ] The NASA 1440×720, 4-pixels-per-degree floating-point lunar elevation map is versioned with source URL, retrieval date, checksum, units, reference radius, and usage provenance.
- [ ] Shared image handling decodes the floating-point TIFF without coupling file formats to rasterization behavior.
- [ ] Nearest neighboring elevation texels estimate local longitude and latitude gradients robustly, including at longitude wrap and latitude limits.
- [ ] Elevation in kilometers and the 1,737.4 km lunar reference radius determine terrain-normal slopes without artistic exaggeration.
- [ ] Terrain normals remain distinct from radial direction: terrain normals affect lighting while radial direction continues to select map locations.
- [ ] The octasphere vertices remain on a perfect sphere and its silhouette is unchanged.
- [ ] Native and web renders visibly reveal lunar terrain under the directional Sun.
- [ ] Canonical terrain-shaded lunar goldens compare with documented tolerance and emit useful diff artifacts.
- [ ] Focused tests cover TIFF decoding, units, flat elevation, known gradients, seam behavior, pole behavior, and normalized terrain normals.
- [ ] Learning documentation derives terrain normals from spherical elevation gradients.
- [ ] The local quality gate passes.
