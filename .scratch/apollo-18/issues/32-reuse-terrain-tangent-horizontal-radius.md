# 32: Reuse the terrain tangent frame's horizontal radius

**What to build:** Preserve physical terrain-normal derivation while reusing the fragment globe location's tangent-frame horizontal radius instead of recalculating the corresponding latitude cosine.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** done

- [x] Ordinary lunar coordinates derive eastward and northward physical slopes with the existing reference-sphere semantics.
- [x] Polar rows retain their defined tangent frame, one-sided latitude derivative, and zero eastward slope.
- [x] Focused tests cover the equator, antimeridian, near-polar coordinates, and exact poles.
- [x] Realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [x] The retained diagnostic records the isolated complete-frame effect; the change is retained only when it improves the representative workload without regression.
- [x] The release browser performance contract, browser smoke coverage, and local quality gate pass apart from the still-open 30 FPS threshold if the cumulative work has not reached it yet.

## Comments

The tangent frame now retains its already-calculated horizontal radius for
physical terrain-slope derivation, removing the equivalent per-fragment
latitude cosine. Three baseline runs had a 56.0 ms median complete-frame time;
three changed runs had a 54.1 ms median. The 1.9 ms reduction exceeded the
baseline's 1.8 ms run-to-run span, and the measured ranges did not overlap, so
the optimization was retained. The sustained performance contract measured
18.22 FPS and remains below Ticket 19's eventual 30 FPS target as permitted by
this ticket. Focused tests, realistic and exact goldens, native smoke coverage,
browser smoke coverage, and the canonical quality gate passed. The
implementation review found no standards or specification issues.
