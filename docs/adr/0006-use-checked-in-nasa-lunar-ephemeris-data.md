# Use checked-in NASA lunar ephemeris data

Apollo 18 derives date-dependent lunar illumination and apparent Earth-centered orientation from NASA Scientific Visualization Studio's original annual Moon Phase and Libration JSON, committed unchanged with separate provenance, rather than calculating ephemerides in project code or depending on a live service. Native and web hosts provide the same bytes to a focused shared-renderer ephemeris module, which strictly validates required values and contiguous hourly timestamps and samples them only by astronomical instant. A separate lunar-phase animation module owns the animation epoch, scene-time mapping, and complete-cycle coverage requirement.

For the first ephemeris-driven stage, the animation epoch is fixed at `2026-01-01T00:00:00Z`. Scene time `t` maps each ten-second cycle onto a mean synodic month:

```text
astronomy_time(t) = epoch + ((t mod 10 seconds) / 10 seconds) × 29.530588853 days
```

Each astronomical instant uses the nearest hourly NASA record without interpolating its sub-Earth point, subsolar point, or lunar position angle. An instant exactly halfway between records selects the later record. An ephemeris sample converts its sub-Earth point into the object-to-world rotation and its matching subsolar point into the world-space Sun direction through the renderer's established lunar longitude and latitude convention; this is coordinate conversion, not an astronomical position model.

## Considered options

Runtime NASA requests were rejected because they would make rendering depend on network availability, do not support both hosts consistently, and were not CORS-compatible with the static showcase. JPL Horizons, Skyfield with JPL kernels, SPICE, and Astronomy Engine could provide more precision or arbitrary date coverage, but would add generation machinery or astronomical calculations that Apollo 18 does not need for this visual presentation.

## Consequences

The animation is reproducible and inspectable, but inherits NASA's hourly precision, changes pose and illumination in discrete hourly steps, has bounded annual coverage, and adds approximately 2 MB to project assets and compiled hosts. Nearest-record sampling avoids angular wrap handling and keeps Apollo 18 from fabricating values between NASA records; it is intended for visual presentation rather than scientific analysis. A mean synodic month is not exactly periodic in the source data, so the ten-second reset deliberately permits a discontinuity rather than blending or inventing ephemeris values. Later stages may select a current animation epoch only when committed data covers the complete cycle.
