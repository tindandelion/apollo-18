# 02: Use NASA's unsigned lunar elevation TIFF

**What to build:** Replace the repository's floating-point lunar elevation source with NASA's unsigned 16-bit Moon Kit source so native and web rendering retain the same lunar terrain appearance while the delivered Wasm becomes substantially smaller.

**Blocked by:** 01/Add repeatable Wasm size measurement.

**Status:** done

- [x] The direct NASA unsigned 16-bit Moon Kit TIFF is checked in as the sole runtime lunar elevation source, and the superseded floating-point TIFF is removed.
- [x] Adjacent provenance records the NASA source URL, retrieval date, checksum, dimensions, half-meter units, 20,000-sample offset, conversion to kilometers, usage guidance, and a link to the retained format research.
- [x] Native rendering, web rendering, and golden-render tests all consume the same new source asset.
- [x] Asset loading accepts unsigned 16-bit TIFF samples, converts every sample to floating-point kilometers relative to the lunar reference radius, and rejects floating-point, malformed, and unsupported TIFF inputs.
- [x] Decoder unit tests follow Arrange-Act-Assert and cover representative positive, zero, and negative elevations as well as each required rejection case.
- [x] Existing golden fixtures remain untouched and golden comparisons pass. If they initially fail, amplified differences are visually inspected and the proceed-or-roll-back decision is recorded before completion.
- [x] On the same machine and toolchain, the release Wasm is at least 2.0 MB smaller raw and 0.8 MB smaller with gzip than the baseline captured by Ticket 01.
- [x] Repeated cold-load measurements through first framebuffer presentation are compared with the baseline and recorded; any clear initialization regression is investigated before completion.
- [x] Native smoke tests, web smoke tests, relevant warmed web performance checks, and the canonical quality gate pass.

## Comments

Implemented and committed with this ticket update. The release Wasm decreased by
2,062,252 raw bytes and 935,569 gzip bytes. Golden, native smoke, web smoke,
timeline performance, and canonical quality-gate checks passed. The strict
30 FPS performance contract was below threshold for both the unchanged baseline
and changed build in the same measurement session; three-run comparisons
showed overlapping ranges and no regression, and this accepted environmental
result is recorded in `docs/testing.md`.
