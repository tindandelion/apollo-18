# 27: Apply lunar position angle

**What to build:** Complete the Moon's date-dependent orientation by rolling the librating lunar globe according to NASA's lunar position angle, so native and web output present the apparent Earth-centered disk orientation relative to celestial north.

**Blocked by:** 26: Apply the sub-Earth point

**Status:** ready-for-agent

- [ ] Shared ephemeris data handling validates and samples NASA's lunar position angle for each selected astronomical instant.
- [ ] Lunar position angle follows the shortest angular interpolation path across its wrap boundary.
- [ ] Framebuffer up represents celestial north, and the Moon's north-pole axis appears at NASA's counterclockwise lunar position angle under the project's top-left framebuffer convention.
- [ ] Position-angle roll composes with sub-Earth centering without moving the globe center or changing its apparent size.
- [ ] The subsolar point and terrain normals are transformed consistently with the rolled globe so surface illumination remains geographically correct.
- [ ] Native and web hosts present matching apparent roll from the fixed canonical animation epoch.
- [ ] Canonical goldens protect representative position angles, including a wrap-boundary case.
- [ ] Focused tests cover sign convention, transform composition, shortest-path interpolation, and deterministic output in Arrange-Act-Assert form.
- [ ] The project specification and learning guide explain lunar position angle, celestial north, and the framebuffer-space sign convention.
- [ ] The canonical local quality gate, native smoke tests, and browser smoke tests pass.
