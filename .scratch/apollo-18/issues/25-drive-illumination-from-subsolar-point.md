# 25: Drive illumination from the subsolar point

**What to build:** Replace the synthetic Sun orbit with NASA's date-dependent subsolar point across the shared renderer, native lunar sequence, and web showcase, using a fixed canonical animation epoch first so the complete data path is deterministic and reviewable before host clocks are introduced.

**Blocked by:** 24: Make lunar appearance explicit

**Status:** done

- [x] NASA's original 2026 hourly Moon Phase and Libration JSON is committed unchanged as a project asset with its source URL, retrieval date, checksum, and reuse terms documented separately from the code license.
- [x] Shared ephemeris data handling parses the annual NASA JSON, validates required timestamps and subsolar coordinates, and reports malformed, incomplete, duplicate, or non-hourly data as errors rather than silently inventing samples.
- [x] The canonical animation epoch is `2026-01-01T00:00:00Z`, and scene time maps one ten-second cycle to the mean synodic month of 29.530588853 days.
- [x] Each astronomical instant uses the nearest hourly subsolar coordinates without interpolation; an exact half-hour tie selects the later record.
- [x] Each sampled subsolar point produces the corresponding Sun direction without Apollo 18 implementing a solar or lunar position model.
- [x] The lunar globe remains fixed at this stage while native and web hosts present the same date-dependent illumination.
- [x] The old synthetic Sun orbit is removed rather than retained as another lunar-animation mode.
- [x] Canonical lunar goldens use the fixed animation epoch and are identified by represented UTC astronomical instant at minute precision and scene time rather than synthetic phase names.
- [x] Focused tests cover exact hourly samples, nearest-record boundaries, the mean-synodic-month mapping, deterministic out-of-order frame requests, and malformed source data in Arrange-Act-Assert form.
- [x] The project specification, asset documentation, ADR, and learning guide record the checked-in annual-data decision, equations, accuracy boundary, and the deliberate possibility of a discontinuity at the ten-second reset.
- [x] Asset-provenance checks, the canonical local quality gate, native smoke tests, and browser smoke tests pass.
