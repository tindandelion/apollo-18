# 31: Calculate lunar illumination in object space

**What to build:** Produce the same geographically consistent terrain-normal Lambertian illumination while transforming the Sun direction into lunar-globe object space once per frame instead of transforming every terrain normal into world space.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** ready-for-agent

- [ ] Object-space illumination is mathematically equivalent for every valid lunar globe pose and Sun direction.
- [ ] Focused tests cover identity and rotated lunar appearances, illuminated and unlit terrain normals, and terrain-normal rim highlights.
- [ ] Lunar orientation, phase, color-map sampling, elevation-map sampling, and terrain shading remain unchanged in realistic goldens.
- [ ] Native output dimensions and deterministic native output remain unchanged.
- [ ] The retained diagnostic records the isolated complete-frame effect; the change is retained only when it improves the representative workload without regression.
- [ ] The release browser performance contract, browser smoke coverage, and local quality gate pass apart from the still-open 30 FPS threshold if the cumulative work has not reached it yet.
