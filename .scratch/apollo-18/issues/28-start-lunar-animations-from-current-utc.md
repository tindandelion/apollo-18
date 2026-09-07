# 28: Start lunar animations from current UTC

**What to build:** Start each native and web lunar animation from the current UTC instant captured once at startup, while preserving fixed-epoch tests and failing clearly before rendering when the checked-in NASA data cannot cover the complete ten-second cycle.

**Blocked by:** 27: Apply lunar position angle

**Status:** ready-for-agent

- [ ] Native and web hosts capture one current UTC animation epoch at startup and continue deriving animation progress from explicit monotonic scene time rather than rereading or accumulating wall-clock time per frame.
- [ ] Native production rendering always uses the captured current UTC epoch; it adds no start-time override and writes no epoch sidecar metadata.
- [ ] Before rendering the first frame, each host verifies that annual data covers the animation epoch through one complete mean synodic month.
- [ ] The native host reports an actionable error and writes no frames when coverage is missing or invalid.
- [ ] The web host replaces the canvas with a visible actionable message and logs detailed diagnostics when coverage is missing or invalid.
- [ ] A valid animation maps every ten seconds to the same real sampled month and accepts the resulting reset discontinuity rather than blending or fabricating ephemeris values.
- [ ] The globe remains centered at its existing apparent size; right ascension, declination, Earth-Moon distance, topocentric parallax, and eclipse shadows remain out of scope.
- [ ] Renderer and host tests inject `2026-01-01T00:00:00Z`; automated tests and golden generation never depend on the real wall clock or network.
- [ ] Browser smoke coverage controls the wall clock explicitly and verifies both successful presentation and the visible out-of-coverage failure state.
- [ ] Native smoke coverage verifies that coverage failure occurs before any numbered PNG is written.
- [ ] A repeatable 1152×1152 browser measurement records the realistic animation's preparation, recurring rendering, and complete presentation baseline without imposing a premature performance gate.
- [ ] The performance analysis and pending preparation/optimization tickets are revised to treat changing lunar pose and geographic sampling as real recurring work rather than fixed-view invariants.
- [ ] The project specification and learning documentation explain animation epoch, compressed astronomical time, deterministic tests, annual data maintenance, coverage failure, loop discontinuity, and excluded eclipse behavior.
- [ ] The canonical local quality gate, native smoke tests, browser smoke tests, asset-provenance checks, and ticket-specific performance measurement pass.
