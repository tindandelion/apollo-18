# 26: Apply the sub-Earth point

**What to build:** Add geocentric lunar libration to the date-dependent native and web animation by orienting the globe so NASA's interpolated sub-Earth point occupies the center of the visible disk while illumination continues to come from the matching subsolar point.

**Blocked by:** 25: Drive illumination from the subsolar point

**Status:** ready-for-agent

- [ ] Shared ephemeris data handling validates and samples the NASA sub-Earth longitude and latitude for the astronomical instant selected by each frame.
- [ ] Sub-Earth latitude is interpolated linearly, while longitude follows the shortest angular path across its wrap boundary.
- [ ] The sampled sub-Earth point is centered toward the Earth-centered camera, producing geocentric longitudinal and latitudinal libration without depending on a terrestrial observer location.
- [ ] Lunar north remains upright at this intermediate stage; lunar position angle is not applied prematurely.
- [ ] The subsolar point is transformed consistently with the changing globe pose so illumination remains attached to the correct lunar locations.
- [ ] Map lookup, terrain-normal lookup, day-side gating, culling, and shading use the changed globe pose correctly rather than reusing stale fixed-pose data.
- [ ] Native and web hosts present matching libration from the fixed canonical animation epoch without changing globe centering or apparent size.
- [ ] Canonical goldens protect representative longitudinal and latitudinal libration states and are named by animation epoch and scene time.
- [ ] Focused tests cover exact and interpolated sub-Earth samples, centering, longitude sign, latitude sign, Sun/globe consistency, and deterministic frame ordering in Arrange-Act-Assert form.
- [ ] The project specification and learning guide explain the sub-Earth point, geocentric libration, and the globe-pose transform.
- [ ] The canonical local quality gate, native smoke tests, and browser smoke tests pass.
