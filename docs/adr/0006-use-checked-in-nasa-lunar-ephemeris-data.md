# Use checked-in NASA lunar ephemeris data

Apollo 18 derives date-dependent lunar illumination and apparent Earth-centered orientation from NASA Scientific Visualization Studio's original Moon Phase and Libration JSON, committed unchanged with separate provenance, rather than calculating ephemerides in project code or depending on a live service. Native and web hosts provide the same bytes to a focused shared-renderer ephemeris module, which strictly validates required values and contiguous hourly timestamps and samples them only by astronomical instant.

The first validated ephemeris timestamp is the **animation epoch**: the astronomical instant represented at scene time zero. Neither host reads the UTC wall clock. A separate lunar-phase animation module owns two timeline policies:

- The native **synodic-month animation** maps each ten-second cycle across the first mean synodic month of `29.530588853 days`:

  ```text
  astronomy_time(t) = epoch + ((t mod 10 seconds) / 10 seconds) × 29.530588853 days
  ```

- The web **ephemeris-span animation** maps each 120-second cycle linearly from the first validated timestamp through the last:

  ```text
  progress(t) = (t mod 120 seconds) / 120 seconds
  astronomy_time(t) = first_timestamp + progress(t) × (last_timestamp - first_timestamp)
  ```

  The first web frame explicitly presents the first record before monotonic playback advances. A delayed animation callback skips astronomical instants instead of slowing the timeline. Replacing the checked-in source changes the represented span but not the 120-second playback duration.

Each astronomical instant uses the nearest hourly NASA record without interpolating its sub-Earth point, subsolar point, or lunar position angle. An instant exactly halfway between records selects the later record. In ephemeris-span playback this gives the first and last records half as much timeline as interior records. An ephemeris sample converts its sub-Earth point into the object-to-world rotation and its matching subsolar point into the world-space Sun direction through the renderer's established lunar longitude and latitude convention; this is coordinate conversion, not an astronomical position model.

## Considered options

Runtime NASA requests were rejected because they would make rendering depend on network availability, do not support both hosts consistently, and were not CORS-compatible with the static showcase. JPL Horizons, Skyfield with JPL kernels, SPICE, and Astronomy Engine could provide more precision or arbitrary date coverage, but would add generation machinery or astronomical calculations that Apollo 18 does not need for this visual presentation.

Starting production animation from the current UTC instant was rejected because checked-in annual data would make the showcase expire, require date-dependent failure behavior, and make native release artifacts vary with build time. Giving both hosts the same synodic-month loop was also rejected: the compact native artifact benefits from its established ten-second cycle, while the continuously running web showcase can present the complete source over two minutes.

Interpolating records was rejected because it would fabricate values absent from NASA's source and require wrap rules for longitude and position angle. Forcing either endpoint to blend seamlessly was rejected for the same reason.

## Consequences

Animation is reproducible and inspectable but inherits NASA's hourly precision, changes pose and illumination in discrete steps, has bounded source coverage, and adds approximately 2 MB to project assets and compiled hosts. Nearest-record sampling is intended for visual presentation rather than scientific analysis.

The native host must verify one complete mean synodic month of coverage from the first timestamp before writing frames. The web host accepts any valid non-empty contiguous ephemeris because its timeline is bounded by that source's first and last timestamps. A mean synodic month is not exactly periodic in the source, and the end of an ephemeris need not match its beginning, so both loops deliberately permit discontinuities rather than blending or inventing values.
