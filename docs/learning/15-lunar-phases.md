# Animating lunar phases from the subsolar point

A **lunar phase** is the visible pattern of illumination set by the angle between the viewing direction and the **Sun direction**. Apollo 18 now gets that direction from NASA's hourly **subsolar point** rather than moving the Sun through a synthetic circle. The camera and **lunar globe pose** remain fixed in this stage, isolating date-dependent illumination from the libration and apparent-roll stages that follow.

## Scene time and astronomical time

The animation uses the fixed **animation epoch** `2026-01-01T00:00:00Z`. Let `t` be explicit scene time and let `T = 10 seconds`. One display cycle advances through the mean **synodic month** `M = 29.530588853 days`:

```text
cycle_fraction(t) = (t mod T) / T
astronomy_time(t) = animation_epoch + cycle_fraction(t) × M
```

The mapping is derived directly from scene time, never from accumulated frame steps. Native and web requests at equal scene times therefore sample the same astronomical instant regardless of frame rate. A real ephemeris does not repeat after exactly one mean synodic month, so the reset at ten seconds can have a small deliberate discontinuity.

## Interpolating hourly samples

For an astronomical time between adjacent hourly samples, let `u` be its fraction through the hour. Subsolar latitude is ordinary linear interpolation:

```text
latitude(u) = latitude₀ + u(latitude₁ - latitude₀)
```

Longitude is periodic. First choose the signed difference in `[-180°, 180°)` and then interpolate:

```text
delta = wrap(longitude₁ - longitude₀, -180°, 180°)
longitude(u) = wrap(longitude₀ + u × delta, -180°, 180°)
```

This takes the short path across the antimeridian. For example, halfway from `179°` to `-179°` is `±180°`, not `0°`.

## Converting a subsolar point to Sun direction

Apollo 18's globe coordinates put lunar north on `+Y`, zero-degree longitude on `-Z`, and east on `+X`. For subsolar longitude `λ` and latitude `φ`, the unit direction from the lunar globe toward the Sun is:

```text
sun_direction = (cos φ sin λ, sin φ, -cos φ cos λ)
```

At `(0°, 0°)` this gives `-Z`, placing the Sun on the viewer's side and producing a full Moon for the current identity pose. The shared lunar-phase animation policy asks the ephemeris for that astronomical instant and packages its direction into a **lunar appearance**. The ephemeris knows only how to sample UTC instants; lunar rasterization knows only the explicit appearance.

## Terrain shading and scope

Each fragment retains terrain-normal Lambertian shading:

```text
diffuse = max(dot(terrain_normal, sun_direction), 0)
linear_output = linear_lunar_color × diffuse
```

There is no ambient or specular term. Terrain normals can still create sparse rim highlights where undisplaced spherical geometry would be dark, as recorded in ADR-0005. The hourly source and linear interpolation are appropriate for this visual animation, not scientific analysis. Sub-Earth-point libration and lunar position angle are intentionally deferred, so the familiar near side remains fixed during this stage.
