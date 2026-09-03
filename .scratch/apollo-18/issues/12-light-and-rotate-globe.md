# 12: Light and rotate the lunar globe

**What to build:** Turn the color-mapped lunar globe into an animated showcase with smooth linear-light Lambertian shading, a gibbous directional Sun, and a ten-second rotation in native frame output and the webpage.

**Blocked by:** 09: Apply the NASA lunar color map

**Status:** ready-for-agent

- [ ] The smooth globe uses normalized interpolated radial direction as its per-fragment lighting normal.
- [ ] One neutral directional Sun produces Lambertian diffuse shading in linear RGB.
- [ ] Ambient and specular contributions are zero.
- [ ] The initial Sun direction is approximately 30 degrees from the viewing direction, producing a gibbous appearance.
- [ ] The lunar globe rotates around lunar north once every ten seconds against the opaque sRGB `#181818` background.
- [ ] Animation is a pure function of explicit scene time and is independent of frame rate.
- [ ] The native lunar binary renders a selected deterministic frame or a numbered PNG sequence at a requested frame rate.
- [ ] The webpage animates from monotonic elapsed time through `requestAnimationFrame`.
- [ ] Canonical gibbous and rotated lunar goldens protect mapping, linear-light shading, orientation, and timing.
- [ ] Focused tests cover radial normals, Lambertian limits, sRGB round trips, and time-to-rotation mapping.
- [ ] Learning documentation explains linear-light Lambertian shading and time-driven animation.
- [ ] The local quality gate passes.
