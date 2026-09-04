# 09: Apply the NASA lunar color map

**What to build:** Turn the octasphere into a recognizable lunar globe by applying NASA's 2025 2048×1024 lunar color map through per-fragment globe-location lookup in both native and web output.

**Blocked by:** 08: Animate a vertex-colored octasphere

**Status:** done

## Settled design

- The canonical asset is NASA CGI Moon Kit's 2025 `lroc_color_2k.jpg`, stored at `assets/nasa/lroc_color_2k.jpg`. A Markdown sidecar under `assets/nasa/` records the source page, direct download URL, retrieval date, SHA-256 checksum, and usage terms separately from Apollo 18's code license.
- The shared image module uses the pure-Rust `image` crate with only JPEG support enabled for this ticket. JPEG decoding produces an Apollo 18-owned `SrgbImage`; `LunarColorMap` wraps that decoded image without knowing about JPEG.
- `LunarColorMap` accepts any non-empty image dimensions so focused tests can use small synthetic maps. A canonical-asset test separately verifies the committed map's 2048×1024 dimensions and recorded SHA-256 checksum.
- `render_lunar_globe` receives a decoded `&LunarColorMap`. Both native and Wasm hosts embed the JPEG with `include_bytes!`, decode it once during startup, and pass the map by reference; the renderer performs no filesystem or network access.
- Lunar vertices carry object-space globe location before globe rotation. The rasterizer interpolates and normalizes that location per fragment before lunar map lookup, so the map rotates with the globe.
- Vertex-color and lunar-map triangles use one generic rasterization path parameterized by a small, statically dispatched fragment shader. Each shader converts per-vertex attributes and barycentric weights to linear RGB; this internal seam is not a general-purpose shader language or material system.
- The color-mapped globe retains ticket 08's deterministic ten-second rotation. Its canonical golden is an 800×800 frame at scene time zero named `color_mapped_lunar_globe_at_zero_seconds.png`, replacing `vertex_colored_octasphere.png`; triangle and cube goldens remain byte-exact.
- Realistic lunar golden comparison permits a maximum absolute difference of one per RGB channel and requires alpha to match exactly. A failure writes an amplified visual diff and summary statistics under `target/apollo18/golden-diffs/`; golden replacement remains gated by the existing explicit update action.

- [x] The selected NASA color data is versioned with source URL, retrieval date, checksum, and usage provenance distinct from the code license.
- [x] Shared image handling decodes JPEG bytes without coupling file formats to rasterization behavior.
- [x] Each fragment's normalized globe location maps to longitude and latitude without mesh UV attributes.
- [x] Longitude wraps, latitude clamps, and nearest-neighbor access samples valid pixels at seams and poles.
- [x] Zero-degree longitude faces the initial camera and the familiar lunar near side appears centered.
- [x] Sampled sRGB color can be decoded to linear RGB and encoded back to sRGB without unintended color shifts.
- [x] The native lunar binary and webpage display the same color-mapped lunar globe without runtime NASA network access.
- [x] Canonical realistic lunar goldens compare decoded pixels with a small documented tolerance and emit amplified diff images on failure.
- [x] Golden replacement requires an explicit update action.
- [x] Focused tests cover JPEG decoding, globe-location mapping, seam/pole behavior, nearest-neighbor access, and color conversion.
- [x] Learning documentation explains spherical lookup and color spaces.
- [x] The local quality gate passes.
