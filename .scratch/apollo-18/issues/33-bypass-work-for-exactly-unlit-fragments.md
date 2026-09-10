# 33: Bypass downstream work for exactly unlit fragments

**What to build:** Present exactly unlit lunar fragments as opaque black without unnecessary lunar color-map sampling or general sRGB encoding, while retaining terrain-normal illumination and the complete software-render-and-present path.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** done

- [x] The fast path is selected only after terrain-normal Lambertian intensity is known to be exactly zero.
- [x] Terrain-normal lunar rim highlights required by ADR-0005 remain eligible for illumination.
- [x] Lit fragments continue to use the lunar color map, linear-light shading, and normal sRGB encoding.
- [x] Focused tests distinguish exactly unlit, barely lit, and terrain-normal rim-highlight fragments.
- [x] Realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [x] Measurements cover representative illuminated and unlit phases and record both the aggregate benefit and its phase dependence; the change is retained only without a representative-workload regression.
- [x] The release browser performance contract, browser smoke coverage, and local quality gate pass apart from the still-open 30 FPS threshold if the cumulative work has not reached it yet.
