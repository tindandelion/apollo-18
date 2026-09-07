# 24: Make lunar appearance explicit

**What to build:** Let callers render the lunar globe from an explicit globe pose and Sun direction while preserving the existing synthetic lunar-phase showcase unchanged, so later ephemeris stages can replace one aspect of the appearance at a time without coupling astronomical data to rasterization.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The shared renderer accepts explicit lunar globe pose and Sun direction inputs through a small deterministic rendering seam.
- [ ] Globe pose and Sun direction remain distinct concepts, and rasterization does not gain date, clock, NASA-file, or astronomical-period knowledge.
- [ ] Invalid non-finite or zero-length directional inputs are rejected without panics or unbounded work.
- [ ] Native and web lunar animations continue to present the existing fixed-globe synthetic phase cycle through the explicit appearance seam.
- [ ] Existing lunar golden images remain unchanged within their documented tolerance.
- [ ] Focused tests prove that equal dimensions, assets, scene time, globe pose, and Sun direction produce equal framebuffers and follow Arrange-Act-Assert.
- [ ] Existing native output behavior and browser smoke coverage remain unchanged.
- [ ] Relevant learning documentation distinguishes explicit lunar appearance from the policy that selects it.
- [ ] The canonical local quality gate, native smoke tests, and browser smoke tests pass.
