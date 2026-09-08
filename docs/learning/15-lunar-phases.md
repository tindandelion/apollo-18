# Animating lunar phases and geocentric orientation

A **lunar phase** is the visible pattern of illumination set by the angle between the viewing direction and the **Sun direction**. Apollo 18 gets that direction from NASA's hourly **subsolar point**. NASA's matching **sub-Earth point** determines geocentric libration, while the **lunar position angle** determines the disk's apparent roll relative to celestial north. Together they define the **lunar globe pose**.

## Scene time and astronomical time

The **animation epoch** is the first validated ephemeris timestamp: the astronomical instant represented at scene time zero. For the checked-in source it is `2026-01-01T00:00:00Z`. It comes from data rather than a host's UTC wall clock.

Apollo 18 uses two deterministic timeline policies. Native rendering uses a **synodic-month animation**. Let `t` be explicit scene time, `Tₙ = 10 seconds`, and `M = 29.530588853 days` be the mean **synodic month**:

```text
native_fraction(t) = (t mod Tₙ) / Tₙ
native_astronomy_time(t) = animation_epoch + native_fraction(t) × M
```

The web showcase uses an **ephemeris-span animation**. Let `T_web = 120 seconds`, `first` be the animation epoch, and `last` be the final validated timestamp:

```text
web_fraction(t) = (t mod T_web) / T_web
web_astronomy_time(t) = first + web_fraction(t) × (last - first)
```

The first browser animation callback establishes monotonic scene time zero and presents `first`. Later callbacks derive scene time from that monotonic origin. If rendering stalls, elapsed time continues and astronomical samples are skipped rather than slowing playback. A replacement ephemeris still occupies one 120-second web cycle regardless of its calendar span.

Both mappings derive directly from scene time, never from accumulated frame steps. A real ephemeris does not repeat after exactly one mean synodic month, and its final state need not match its first, so both resets can have deliberate discontinuities.

## Selecting an hourly sample

For an astronomical time between adjacent hourly NASA records, Apollo 18 uses the nearer complete record. If `h` is the number of hours since the first record, the selected record index is:

```text
sample_index = round(h)
```

An exact half-hour tie selects the later record. The subsolar point, sub-Earth point, and lunar position angle always come from that same record, so illumination and pose remain geographically consistent. This deliberately produces small hourly steps in the compressed animation, but avoids inventing intermediate ephemeris values and removes special interpolation rules for longitude and angle wrap boundaries.

Because ephemeris-span time runs from the first timestamp through the last, nearest selection gives the first and last records half the display interval of interior records. The modulo timeline never needs to sample beyond the source. At typical display rates many compressed hourly records are skipped, but playback still traverses the complete astronomical span.

## Converting a subsolar point to Sun direction

Apollo 18's globe coordinates put lunar north on `+Y`, zero-degree longitude on `-Z`, and east on `+X`. For subsolar longitude `λ` and latitude `φ`, the unit direction from the lunar globe toward the Sun is:

```text
sun_direction = (cos φ sin λ, sin φ, -cos φ cos λ)
```

At `(0°, 0°)` this gives `-Z`, placing the direction on the viewer-facing meridian before posing the globe.

## Centering the sub-Earth point

Let the sampled sub-Earth longitude and latitude be `λₑ` and `φₑ`. The same globe-location equation gives the object-space direction toward Earth. The object-to-world rotation is:

```text
object_to_world = rotate_x(-φₑ) × rotate_y(λₑ)
```

The rightmost longitude rotation acts first. It moves the sub-Earth point onto the central meridian; the latitude rotation then moves it to world `-Z`, toward the Earth-centered camera. Before roll, lunar north `(0, 1, 0)` becomes `(0, cos φₑ, -sin φₑ)`. Its framebuffer projection has no sideways component and points upward.

## Aligning lunar north with celestial north

The sub-Earth point fixes the disk center but leaves rotation around the viewing axis unspecified. NASA's lunar position angle `P` supplies that final rotational degree of freedom: it is the apparent counterclockwise angle from celestial north to the Moon's north-pole axis. Apollo 18 makes framebuffer up represent celestial north.

The camera looks along world `+Z`, so the viewing axis is `Z`. The complete object-to-world rotation is:

```text
object_to_world = rotate_z(P) × rotate_x(-φₑ) × rotate_y(λₑ)
```

The rightmost transform still acts first. Positive `rotate_z(P)` moves projected lunar north from up toward framebuffer left, which is counterclockwise despite framebuffer pixel rows increasing downward. Rolling after centering leaves the sub-Earth direction on world `-Z`, so the globe remains centered and keeps the same apparent size.

The subsolar direction begins in the same object-space lunar coordinates and is rotated by the complete `object_to_world` rotation before becoming the world-space **Sun direction**. Geometry and terrain normals use that same rotation. Lunar color-map and elevation-map lookup continue to use the unrotated **globe location**, so geography moves with the posed globe rather than sliding across it.

The shared lunar-phase animation policies select one astronomical instant, then sample both points and the position angle and package the resulting pose and Sun direction into a **lunar appearance**. The ephemeris sample hides the source values and owns their conversion into the object-to-world rotation and matching world-space Sun direction. Lunar rasterization only consumes the explicit appearance.

## Terrain shading and scope

Each fragment retains terrain-normal Lambertian shading:

```text
diffuse = max(dot(terrain_normal, sun_direction), 0)
linear_output = linear_lunar_color × diffuse
```

There is no ambient or specular term. Terrain normals can still create sparse rim highlights where undisplaced spherical geometry would be dark, as recorded in ADR-0005. Nearest-record sampling of the hourly source is appropriate for this visual animation, not scientific analysis. The sub-Earth point provides an Earth-centered view, not a location-dependent terrestrial view; topocentric parallax remains out of scope.
