# 24: Make lunar illumination explicit

**What to build:** Let callers render the lunar globe from an explicit Sun direction through a lunar appearance that retains the current identity globe pose, while preserving the existing synthetic lunar-phase showcase unchanged. This establishes the policy/rendering boundary needed for date-dependent illumination without prematurely designing non-identity pose behavior.

**Blocked by:** None (can start immediately)

**Status:** done

## Settled design

- Lunar rendering accepts one explicit lunar appearance and does not accept scene time. A separate shared synthetic-phase policy derives appearance from scene time for the existing native and web animations.
- Lunar appearance keeps the identity lunar globe pose and explicit Sun direction as distinct state without introducing a separate pose type before non-identity pose behavior exists.
- Ticket 24 fixes lunar globe pose at identity and exposes no speculative matrix or rotation operations; later ephemeris tickets will add only the semantic pose behavior they require.
- Sun direction is a normalized world-space direction. Construction rejects non-finite and zero-length vectors.

- [x] The shared renderer accepts an explicit world-space Sun direction through a small deterministic lunar-appearance rendering seam while retaining identity globe pose.
- [x] Identity globe pose and Sun direction remain distinct state, and rasterization does not gain date, clock, NASA-file, or astronomical-period knowledge.
- [x] Invalid non-finite or zero-length directional inputs are rejected without panics or unbounded work.
- [x] Native and web lunar animations continue to present the existing fixed-globe synthetic phase cycle through the explicit appearance seam.
- [x] Existing lunar golden images remain unchanged within their documented tolerance.
- [x] Focused tests prove that equal dimensions, assets, and lunar appearance produce equal framebuffers; equal scene times produce equal synthetic lunar appearances; explicit Sun direction changes rendered illumination; and all tests follow Arrange-Act-Assert.
- [x] Existing native output behavior and browser smoke coverage remain unchanged.
- [x] Relevant learning documentation distinguishes explicit lunar appearance from the policy that selects it and records that pose remains identity at this stage.
- [x] The canonical local quality gate, native smoke tests, and browser smoke tests pass.

## Comments

The post-implementation review clarified that Ticket 24 establishes explicit illumination, not caller-selectable globe pose. Non-identity pose behavior remains deferred until the ephemeris tickets that need it.
