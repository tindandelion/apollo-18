# 15: Animate the lunar phases

**What to build:** Add a deterministic lunar-phase showcase in which the camera and terrain-shaded lunar globe remain fixed while the directional Sun completes a full illumination cycle every ten seconds.

**Blocked by:** 14: Derive terrain normals from lunar elevation

**Status:** done

## Settled design

- The Sun direction rotates at constant angular speed around lunar north (`+Y`) in the world `XZ` plane.
- Scene time `0` is full Moon. The conventional north-up progression reaches left-lit quarter at `2.5s`, new Moon at `5s`, right-lit quarter at `7.5s`, and returns to full at `10s`.
- Zero-degree longitude remains fixed facing the camera.
- The native host keeps its existing frame-rate and frame-count sequence interface; no arbitrary start-frame option is added.
- Existing terrain-normal Lambertian behavior is preserved without a globe-location illumination mask. Ticket 17 separately investigates possible new-Moon rim highlights.
- The two rotating-globe lunar goldens are replaced by phase-specific goldens at `0s`, `1.25s`, `2.5s`, `3.75s`, and `5s`.

- [x] Sun direction follows a complete repeatable cycle derived from explicit scene time.
- [x] One full cycle lasts ten seconds and passes through full, quarter, new, and returning phases.
- [x] The camera and lunar globe remain fixed during the phase animation.
- [x] Terrain-normal shading responds continuously to the changing Sun direction without changing map lookup, globe geometry, or the existing terrain-normal Lambertian behavior; possible new-Moon rim highlights are deferred to Ticket 17.
- [x] The native lunar binary produces a deterministic numbered PNG sequence of the phase cycle using its existing frame-rate and frame-count interface.
- [x] The webpage displays the same phase behavior using monotonic elapsed time.
- [x] Canonical goldens protect representative full, gibbous, quarter, crescent, and new-phase timestamps.
- [x] Focused tests cover cycle periodicity, key phase directions, and frame-rate independence.
- [x] Learning documentation explains the Sun–Moon–viewer relationship that produces lunar phases.
- [x] The local quality gate passes.
