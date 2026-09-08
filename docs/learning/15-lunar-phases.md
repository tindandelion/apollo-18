# Animating lunar phases and geocentric orientation

A **lunar phase** is the visible pattern of illumination set by the angle between the viewing direction and the **Sun direction**. Apollo 18 gets that direction from NASA's hourly **subsolar point**. NASA's matching **sub-Earth point** determines geocentric libration, while the **lunar position angle** determines the disk's apparent roll relative to celestial north. Together they define the **lunar globe pose**.

## Scene time and astronomical time

The animation uses the fixed **animation epoch** `2026-01-01T00:00:00Z`. Let `t` be explicit scene time and let `T = 10 seconds`. One display cycle advances through the mean **synodic month** `M = 29.530588853 days`:

```text
cycle_fraction(t) = (t mod T) / T
astronomy_time(t) = animation_epoch + cycle_fraction(t) × M
```

The mapping is derived directly from scene time, never from accumulated frame steps. Native and web requests at equal scene times therefore sample the same astronomical instant regardless of frame rate. A real ephemeris does not repeat after exactly one mean synodic month, so the reset at ten seconds can have a small deliberate discontinuity.

## Interpolating hourly samples

For an astronomical time between adjacent hourly samples, let `u` be its fraction through the hour. Subsolar and sub-Earth latitude use ordinary linear interpolation:

```text
latitude(u) = latitude₀ + u(latitude₁ - latitude₀)
```

Both longitudes and lunar position angle are periodic. First choose the signed difference in `[-180°, 180°)` and then interpolate:

```text
delta = wrap(longitude₁ - longitude₀, -180°, 180°)
longitude(u) = wrap(longitude₀ + u × delta, -180°, 180°)
```

This takes the short path across a wrap boundary. For example, halfway from `179°` to `-179°` is `±180°`, not `0°`; halfway from a position angle of `359°` to `1°` is `0°`, not `180°`.

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

The shared lunar-phase animation policy samples both points and the position angle for one astronomical instant, then packages the resulting pose and Sun direction into a **lunar appearance**. The ephemeris sample hides the source values and owns their conversion into the object-to-world rotation and matching world-space Sun direction. Lunar rasterization only consumes the explicit appearance.

## Terrain shading and scope

Each fragment retains terrain-normal Lambertian shading:

```text
diffuse = max(dot(terrain_normal, sun_direction), 0)
linear_output = linear_lunar_color × diffuse
```

There is no ambient or specular term. Terrain normals can still create sparse rim highlights where undisplaced spherical geometry would be dark, as recorded in ADR-0005. The hourly source and linear interpolation are appropriate for this visual animation, not scientific analysis. The sub-Earth point provides an Earth-centered view, not a location-dependent terrestrial view; topocentric parallax remains out of scope.
