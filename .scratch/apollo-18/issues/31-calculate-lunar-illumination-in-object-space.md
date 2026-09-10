# 31: Calculate lunar illumination in object space

**What to build:** Produce the same geographically consistent terrain-normal Lambertian illumination while transforming the Sun direction into lunar-globe object space once per frame instead of transforming every terrain normal into world space.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** done

- [x] Object-space illumination is mathematically equivalent for every valid lunar globe pose and Sun direction.
- [x] Focused tests cover identity and rotated lunar appearances, illuminated and unlit terrain normals, and terrain-normal rim highlights.
- [x] Lunar orientation, phase, color-map sampling, elevation-map sampling, and terrain shading remain unchanged in realistic goldens.
- [x] Native output dimensions and deterministic native output remain unchanged.
- [x] The retained diagnostic records the isolated complete-frame effect; the change is retained only when it improves the representative workload without regression.
- [x] The release browser performance contract, browser smoke coverage, and local quality gate pass apart from the still-open 30 FPS threshold if the cumulative work has not reached it yet.

## Comments

The object-space illumination experiment was mathematically equivalent and
passed focused rendering tests, realistic goldens, native smoke coverage,
browser smoke coverage, and the canonical quality gate. Three baseline runs
had a 55.9 ms median complete-frame time; three changed runs had a 54.9 ms
median. The 1.0 ms difference was smaller than observed run-to-run variation,
the timing ranges overlapped, and the result remained inside Ticket 29's noise
bound, so the implementation was reverted. The attempted variant measured
18.52 FPS in the release performance contract; the expected 30 FPS threshold
remains open for Ticket 19. The implementation review found no standards or
specification issues.
