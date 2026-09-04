# 12: Light and rotate the lunar globe

**What to build:** Turn the color-mapped lunar globe into an animated showcase with smooth linear-light Lambertian shading, a gibbous directional Sun, and a ten-second rotation in native frame output and the webpage.

**Blocked by:** 09: Apply the NASA lunar color map

**Status:** done

## Settled design

- **Sun direction** is the unit direction from the lunar globe toward the Sun. Lambertian intensity is `max(dot(lighting_normal, sun_direction), 0)` in linear RGB, with no ambient or specular term.
- At scene time zero the Sun is 30 degrees toward camera-right from the globe-to-camera direction, so the right side is brighter and the terminator sits on the left.
- Object-space radial direction still drives lunar color-map lookup. A copy rotated into world space is interpolated and normalized per fragment to become the smooth lighting normal.
- Native output keeps the existing sequence-only `--fps` and `--num-frames` contract. A single frame is `--fps 1 --num-frames 1`; there is no separate still-image mode.
- Canonical goldens are `gibbous_lunar_globe_at_zero_seconds.png` and `gibbous_lunar_globe_at_two_point_five_seconds.png`, replacing `color_mapped_lunar_globe_at_zero_seconds.png`.

- [x] The smooth globe uses normalized interpolated radial direction as its per-fragment lighting normal.
- [x] One neutral directional Sun produces Lambertian diffuse shading in linear RGB.
- [x] Ambient and specular contributions are zero.
- [x] The initial Sun direction is approximately 30 degrees from the viewing direction, producing a gibbous appearance.
- [x] The lunar globe rotates around lunar north once every ten seconds against the opaque sRGB `#181818` background.
- [x] Animation is a pure function of explicit scene time and is independent of frame rate.
- [x] The native lunar binary renders a selected deterministic frame or a numbered PNG sequence at a requested frame rate.
- [x] The webpage animates from monotonic elapsed time through `requestAnimationFrame`.
- [x] Canonical gibbous and rotated lunar goldens protect mapping, linear-light shading, orientation, and timing.
- [x] Focused tests cover radial normals, Lambertian limits, sRGB round trips, and time-to-rotation mapping.
- [x] Learning documentation explains linear-light Lambertian shading and time-driven animation.
- [x] The local quality gate passes.
