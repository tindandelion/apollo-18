# 17: Investigate new-Moon rim highlights

**What to investigate:** Determine whether terrain normals produce objectionable rim highlights at exact new Moon, distinguish expected bump-normal behavior from rendering artifacts, and recommend whether Apollo 18 should preserve or refine the effect.

**Blocked by:** 15: Animate the lunar phases

**Status:** ready-for-agent

- [ ] Inspect the canonical new-Moon render and quantify illuminated pixels on the visible hemisphere.
- [ ] Explain which highlights follow from terrain-normal Lambertian shading without displaced geometry, self-shadowing, or cast shadows.
- [ ] Compare preserving the current behavior with practical alternatives such as a globe-location illumination mask.
- [ ] Evaluate alternatives against ADR-0003, the learning goals, and the intended lunar-phase appearance.
- [ ] Record a recommendation and create a follow-up implementation ticket if behavior should change.
- [ ] The local quality gate passes for any code or fixture changes made during the investigation.
