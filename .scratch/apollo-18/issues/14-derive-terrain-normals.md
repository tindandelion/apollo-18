# 14: Derive terrain normals from lunar elevation

**What to build:** Reveal crater and terrain relief by deriving per-fragment terrain normals from NASA's physical lunar elevation data while leaving the octasphere geometry and silhouette spherical.

**Blocked by:** 12: Light and rotate the lunar globe

**Status:** done

**Scheduling note:** This follows the lit rotating globe. High-DPI canvas resolution waits until after terrain-normal shading exists as the representative lunar workload.

## Settled design

- The canonical asset is stored at `assets/nasa/ldem_4.tif` with a Markdown sidecar. NASA SVS's `ldem_4.tif` returned HTTP 403 at retrieval time, so the file was reconstructed from PDS LOLA `LDEM_4` using the Moon Kit conversion (int16 half-meters / 2000 → km) and a 180° longitude roll so 0° is centered like the color map.
- Shared image handling decodes 32-bit float TIFF via the `tiff` crate into `ElevationImage`. `LunarElevationMap` wraps that image without knowing about TIFF. The `image` crate remains JPEG-only.
- `render_lunar_globe` receives decoded `&LunarColorMap` and `&LunarElevationMap`. Hosts embed both assets with `include_bytes!`, decode once at startup, and pass them by reference.
- Terrain normals are a reference-sphere bump with `R = 1737.4 km` only. The fragment's interpolated object-space globe location supplies `û`, east, north, and `cos φ`; only elevation samples are nearest-neighbor quantized. Interior gradients are 4-connected central differences; longitude wraps; polar rows use one-sided latitude and zero eastward slope. The object-space terrain normal is rotated with the globe before Lambert.
- Canonical goldens are `terrain_shaded_lunar_globe_at_zero_seconds.png` and `terrain_shaded_lunar_globe_at_two_point_five_seconds.png`, replacing the gibbous pair. Learning guide: `docs/learning/14-terrain-normals.md`. Decision record: ADR-0003.

- [x] The NASA 1440×720, 4-pixels-per-degree floating-point lunar elevation map is versioned with source URL, retrieval date, checksum, units, reference radius, and usage provenance.
- [x] Shared image handling decodes the floating-point TIFF without coupling file formats to rasterization behavior.
- [x] Nearest neighboring elevation texels estimate local longitude and latitude gradients robustly, including at longitude wrap and latitude limits.
- [x] Elevation in kilometers and the 1,737.4 km lunar reference radius determine terrain-normal slopes without artistic exaggeration.
- [x] Terrain normals remain distinct from globe location: terrain normals affect lighting while globe location continues to select map locations.
- [x] The octasphere vertices remain on a perfect sphere and its silhouette is unchanged.
- [x] Native and web renders visibly reveal lunar terrain under the directional Sun.
- [x] Canonical terrain-shaded lunar goldens compare with documented tolerance and emit useful diff artifacts.
- [x] Focused tests cover TIFF decoding, units, flat elevation, known gradients, seam behavior, pole behavior, and normalized terrain normals.
- [x] Learning documentation derives terrain normals from spherical elevation gradients.
- [x] The local quality gate passes.
