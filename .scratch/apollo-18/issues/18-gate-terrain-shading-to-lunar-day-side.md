# 18: Gate terrain shading to the lunar day side

**What to build:** Prevent terrain normals from illuminating fragments on the lunar globe's geometric night side, so exact new Moon is black while terrain relief continues to control brightness on the day side.

**Blocked by:** 07: Investigate new-Moon rim highlights

**Status:** wontfix

**Scheduling note:** Complete this correction before view-volume clipping or Retina-display work so later goldens and performance measurements use the intended lunar-phase behavior.

## Settled design

- Rotate the interpolated globe location into world space and use a positive dot product with the Sun direction as a hard day-side eligibility gate.
- Preserve the existing terrain-normal Lambertian intensity for eligible fragments; do not multiply it by smooth-sphere Lambertian intensity.
- Do not introduce a smoothing width, geometry displacement, self-shadowing, or cast shadows.
- ADR-0004 records the relationship between globe location, terrain normal, and Sun direction.

- [ ] Fragments whose world-space globe location does not face the Sun receive zero lunar illumination even when their terrain normal faces the Sun.
- [ ] Day-side fragments retain the existing terrain-normal Lambertian response without additional cosine attenuation.
- [ ] Exact new Moon contains no directly illuminated visible lunar fragments, while full, quarter, crescent, and returning phases preserve the intended north-up progression.
- [ ] Globe rotation, map lookup, octasphere geometry, spherical silhouette, native dimensions, and golden dimensions remain unchanged.
- [ ] Focused tests distinguish day-side eligibility from terrain-normal intensity and cover the exact terminator boundary.
- [ ] Canonical lunar-phase goldens are reviewed and updated where the geometric gate changes expected pixels.
- [ ] Learning documentation explains geometric day-side eligibility separately from terrain-normal Lambertian shading.
- [ ] The local quality gate passes.

## Comments

The geometric day-side gate was rejected after reviewing both a hard boundary and a two-degree one-sided `smoothstep` fade in the canonical quarter and crescent renders. Both made the large-scale terminator look less natural than the existing terrain-normal Lambertian result. ADR-0005 supersedes ADR-0004 and records the decision to accept sparse new-Moon rim highlights rather than implement this ticket.
