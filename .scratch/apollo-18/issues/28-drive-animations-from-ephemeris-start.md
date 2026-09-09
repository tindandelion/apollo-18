# 28: Drive lunar animations from the ephemeris start

**What to build:** Replace the shared fixed-epoch showcase policy with two deterministic, data-derived timelines: the web showcase presents the complete checked-in ephemeris over a two-minute loop, while native rendering continues to present the first mean synodic month over a ten-second loop.

**Blocked by:** 27: Apply lunar position angle

**Status:** done

- [x] Shared ephemeris data handling exposes the first and last validated timestamps needed by animation policy without exposing source-format details to hosts.
- [x] Both animation policies derive their animation epoch from the first validated ephemeris timestamp and neither native nor web production rendering reads the UTC wall clock.
- [x] The native host uses a synodic-month animation that maps every ten seconds from the ephemeris start across 29.530588853 days, accepts the reset discontinuity, and selects the nearest complete hourly record without interpolation.
- [x] Native sequence arguments remain flexible; the canonical release artifact remains 300 frames at 30 FPS, samples scene time from zero through `299 / 30` seconds, and omits the duplicate ten-second endpoint.
- [x] Before writing its first frame, the native host verifies that the ephemeris covers one complete mean synodic month from its first timestamp; failure is actionable and leaves no numbered PNGs written.
- [x] The web host uses an ephemeris-span animation that maps each 120-second cycle linearly from the first validated timestamp through the last validated timestamp, then resets without blending or fabricating a seamless boundary.
- [x] Web sampling selects the nearest complete hourly record, chooses the later record at an exact half-hour tie, and never interpolates subsolar point, sub-Earth point, or lunar position angle values. The first and last records consequently receive half the display interval of interior records.
- [x] The web host initializes monotonic scene time when its first animation frame is ready to render, presents the first ephemeris record in that frame, and derives later progress from explicit monotonic elapsed time rather than accumulated frame steps.
- [x] Browser stalls and suspension may skip ephemeris samples when presentation resumes; rendering cadence does not slow or pause the two-minute timeline.
- [x] Any valid non-empty contiguous ephemeris is presented over one 120-second web cycle regardless of its calendar span; the current 8,760-record source runs from `2026-01-01T00:00:00Z` through `2026-12-31T23:00:00Z`.
- [x] The globe remains centered at its existing apparent size; right ascension, declination, Earth-Moon distance, topocentric parallax, and eclipse shadows remain out of scope.
- [x] The web host replaces the canvas with a visible actionable message and logs detailed diagnostics when ephemeris data is empty, malformed, duplicate, or non-hourly.
- [x] Renderer and host tests derive the expected `2026-01-01T00:00:00Z` epoch from the checked-in fixture; automated tests and golden generation never depend on the real wall clock or network.
- [x] Focused tests cover both timeline mappings, nearest-record and tie selection, the native coverage boundary, the web loop boundary, the guaranteed first web frame, and deterministic output in Arrange-Act-Assert form.
- [x] Browser smoke coverage controls the monotonic clock explicitly and verifies successful first-record presentation, later ephemeris-span progress, the reset boundary, and the visible invalid-data failure state.
- [x] Native smoke coverage verifies the deterministic first-month sequence and that coverage failure occurs before any numbered PNG is written.
- [x] A repeatable 1152×1152 browser measurement records the ephemeris-span animation's preparation, recurring rendering, and complete presentation baseline without imposing a premature performance gate.
- [x] The performance analysis and pending preparation/optimization tickets treat changing lunar pose and geographic sampling as real recurring work rather than fixed-view invariants.
- [x] The project specification, glossary, ADR, and learning documentation define animation epoch, ephemeris-span animation, synodic-month animation, compressed astronomical time, deterministic tests, coverage failure, both loop discontinuities, and excluded eclipse behavior.
- [x] The canonical local quality gate, native smoke tests, browser smoke tests, asset-provenance checks, and ticket-specific performance measurement pass.

## Comments

The completed implementation was reviewed against repository standards and this ticket with no findings. `./scripts/dev/quality-gate.sh`, native smoke tests, browser smoke tests, and `./scripts/dev/web-timeline-performance-test.sh` pass. The non-gating 1152×1152 ephemeris-span baseline is recorded in `.scratch/apollo-18/performance-analysis.md`.
