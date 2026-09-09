# 32: Reuse the terrain tangent frame's horizontal radius

**What to build:** Preserve physical terrain-normal derivation while reusing the fragment globe location's tangent-frame horizontal radius instead of recalculating the corresponding latitude cosine.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** ready-for-agent

- [ ] Ordinary lunar coordinates derive eastward and northward physical slopes with the existing reference-sphere semantics.
- [ ] Polar rows retain their defined tangent frame, one-sided latitude derivative, and zero eastward slope.
- [ ] Focused tests cover the equator, antimeridian, near-polar coordinates, and exact poles.
- [ ] Realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [ ] The retained diagnostic records the isolated complete-frame effect; the change is retained only when it improves the representative workload without regression.
- [ ] The release browser performance contract, browser smoke coverage, and local quality gate pass apart from the still-open 30 FPS threshold if the cumulative work has not reached it yet.
