# 27: Apply lunar position angle

**What to build:** Complete the Moon's date-dependent orientation by rolling the librating lunar globe according to NASA's lunar position angle, so native and web output present the apparent Earth-centered disk orientation relative to celestial north.

**Blocked by:** 26: Apply the sub-Earth point

**Status:** done

- [x] Shared ephemeris data handling validates and samples NASA's lunar position angle for each selected astronomical instant.
- [x] Lunar position angle comes unchanged from the same nearest hourly record as the sub-Earth and subsolar points.
- [x] Framebuffer up represents celestial north, and the Moon's north-pole axis appears at NASA's counterclockwise lunar position angle under the project's top-left framebuffer convention.
- [x] Position-angle roll composes with sub-Earth centering without moving the globe center or changing its apparent size.
- [x] The subsolar point and terrain normals are transformed consistently with the rolled globe so surface illumination remains geographically correct.
- [x] Native and web hosts present matching apparent roll from the fixed canonical animation epoch.
- [x] Canonical goldens protect representative position angles selected from the hourly source.
- [x] Focused tests cover sign convention, transform composition, nearest-record selection, and deterministic output in Arrange-Act-Assert form.
- [x] The project specification and learning guide explain lunar position angle, celestial north, and the framebuffer-space sign convention.
- [x] The canonical local quality gate, native smoke tests, and browser smoke tests pass.

## Comments

The completed implementation was reviewed against repository standards and this ticket. No blocking findings remained. The positive-roll sign was cross-checked against NASA's north-up `2026-01-01T00:00Z` reference frame, and the updated canonical render has the matching apparent orientation. `./scripts/dev/quality-gate.sh` and `./scripts/dev/web-smoke-test.sh` pass; the quality gate includes native smoke coverage and the ephemeris checksum check.
