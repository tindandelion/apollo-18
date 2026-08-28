# 10: Animate the lunar phases

**What to build:** Add a deterministic lunar-phase showcase in which the camera and terrain-shaded lunar globe remain fixed while the directional Sun completes a full illumination cycle every twenty seconds.

**Blocked by:** 09: Derive terrain normals from lunar elevation

**Status:** ready-for-agent

- [ ] Sun direction follows a complete repeatable cycle derived from explicit scene time.
- [ ] One full cycle lasts twenty seconds and passes through full, quarter, new, and returning phases.
- [ ] The camera and lunar globe remain fixed during the phase animation.
- [ ] Terrain-normal shading responds continuously to the changing Sun direction without changing map lookup or globe geometry.
- [ ] The native lunar binary can render any selected phase frame and produce a numbered PNG sequence.
- [ ] The webpage displays the same phase behavior using monotonic elapsed time.
- [ ] Canonical goldens protect representative full, gibbous, quarter, crescent, and new-phase timestamps.
- [ ] Focused tests cover cycle periodicity, key phase directions, and frame-rate independence.
- [ ] Learning documentation explains the Sun–Moon–viewer relationship that produces lunar phases.
- [ ] The local quality gate passes.
