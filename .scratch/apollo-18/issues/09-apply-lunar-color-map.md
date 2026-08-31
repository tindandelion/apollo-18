# 09: Apply the NASA lunar color map

**What to build:** Turn the octasphere into a recognizable lunar globe by applying NASA's 2025 2048×1024 lunar color map through per-fragment radial-direction lookup in both native and web output.

**Blocked by:** 08: Render a solid-color octasphere

**Status:** ready-for-agent

- [ ] The selected NASA color data is versioned with source URL, retrieval date, checksum, and usage provenance distinct from the code license.
- [ ] Shared image handling decodes JPEG bytes without coupling file formats to rasterization behavior.
- [ ] Each fragment's normalized radial direction maps to longitude and latitude without mesh UV attributes.
- [ ] Longitude wraps, latitude clamps, and nearest-neighbor access samples valid pixels at seams and poles.
- [ ] Zero-degree longitude faces the initial camera and the familiar lunar near side appears centered.
- [ ] Sampled sRGB color can be decoded to linear RGB and encoded back to sRGB without unintended color shifts.
- [ ] The native lunar binary and webpage display the same color-mapped lunar globe without runtime NASA network access.
- [ ] Canonical realistic lunar goldens compare decoded pixels with a small documented tolerance and emit amplified diff images on failure.
- [ ] Golden replacement requires an explicit update action.
- [ ] Focused tests cover JPEG decoding, radial mapping, seam/pole behavior, nearest-neighbor access, and color conversion.
- [ ] Learning documentation explains spherical lookup and color spaces.
- [ ] The local quality gate passes.
