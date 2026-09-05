# 07: Investigate new-Moon rim highlights

**What to investigate:** Determine whether terrain normals produce objectionable rim highlights at exact new Moon, distinguish expected bump-normal behavior from rendering artifacts, and recommend whether Apollo 18 should preserve or refine the effect.

**Blocked by:** 15: Animate the lunar phases

**Status:** done

- [x] Inspect the canonical new-Moon render and quantify illuminated pixels on the visible hemisphere.
- [x] Explain which highlights follow from terrain-normal Lambertian shading without displaced geometry, self-shadowing, or cast shadows.
- [x] Compare preserving the current behavior with practical alternatives such as a globe-location illumination mask.
- [x] Evaluate alternatives against ADR-0003, the learning goals, and the intended lunar-phase appearance.
- [x] Record a recommendation and create a follow-up implementation ticket if behavior should change.
- [x] The local quality gate passes for any code or fixture changes made during the investigation.

## Investigation

The canonical 800×800 phase goldens identify 407,044 lunar-disk pixels: a pixel belongs to the disk when at least one phase differs from the `#181818` background. In the new-Moon golden, 1,019 of those pixels (0.250%) have at least one nonzero sRGB channel. The brightest channel reaches byte 135; 733 pixels (0.180% of the disk) exceed the background's byte value. The illuminated pixels are sparse points around the lunar rim rather than illumination across the disk interior.

These highlights follow from the renderer's two surface approximations. Rasterization determines visibility from the undisplaced spherical octasphere, while Lambertian shading uses a terrain normal tilted away from the globe location by elevation gradients. Near the limb, a fragment can therefore be visible according to the sphere while its terrain normal points toward the far-side Sun. Real displaced terrain would require consistent geometric visibility, occlusion, and shadowing to decide whether such a point can receive and return sunlight. Apollo 18 implements none of those effects, so the highlights are an expected limitation of bump-style terrain-normal shading rather than a rasterization or phase-timing defect.

The practical options are:

1. **Preserve terrain-normal Lambertian shading unchanged.** This is the smallest and most direct implementation of ADR-0003, but exact new Moon retains visible rim points and the apparent terminator can extend onto the sphere's geometric night side.
2. **Gate terrain shading with the globe location.** A fragment is eligible for illumination only when its rotated globe location has a positive dot product with the Sun direction. Eligible fragments retain the existing terrain-normal Lambertian intensity. This makes exact new Moon black and gives globe location and terrain normal separate, teachable responsibilities without changing geometry.
3. **Multiply by smooth-sphere Lambertian intensity.** This removes night-side leakage but applies cosine attenuation in addition to terrain Lambertian shading, unnecessarily darkening ordinary phases and obscuring the physical terrain-normal baseline.
4. **Fade a mask near the terminator.** A one-sided smooth transition can soften the gate, but introduces an arbitrary angular width before evidence shows that a hard boundary is objectionable.
5. **Displace geometry or add visibility and shadowing.** This is the most coherent physical model, but conflicts with ADR-0003's spherical silhouette and exceeds the first renderer's learning scope.

## Recommendation

Adopt option 2 with a hard geometric day-side gate. In world space, illumination becomes:

```text
day_side = dot(n_globe, sun_direction) > 0
diffuse = if day_side {
    max(dot(n_terrain, sun_direction), 0)
} else {
    0
}
```

This preserves ADR-0003: globe location still identifies the undisplaced reference sphere, and terrain normal still controls local Lambertian relief. It also sharpens the learning model by making large-scale day-side eligibility explicit and guarantees no directly illuminated visible pixels at exact new Moon. Start with a hard gate rather than an arbitrary fade, and reassess only if canonical renders reveal an objectionable terminator discontinuity.

ADR-0004 records the rendering convention. Ticket 18 implements it before view-volume clipping or Retina-display work.

## Comments

Canonical renders of Ticket 18's hard gate and a two-degree one-sided `smoothstep` alternative both produced an objectionable geometric terminator. The accepted recommendation is therefore to preserve the existing terrain-normal Lambertian behavior and its sparse exact-new-Moon rim highlights. ADR-0005 supersedes ADR-0004; Ticket 18 is closed as `wontfix`.
