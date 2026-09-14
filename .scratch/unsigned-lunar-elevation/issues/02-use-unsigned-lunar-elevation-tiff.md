# 02: Use NASA's unsigned lunar elevation TIFF

**What to build:** Replace the repository's floating-point lunar elevation source with NASA's unsigned 16-bit Moon Kit source so native and web rendering retain the same lunar terrain appearance while the delivered Wasm becomes substantially smaller.

**Blocked by:** 01/Add repeatable Wasm size measurement.

**Status:** ready-for-agent

- [ ] The direct NASA unsigned 16-bit Moon Kit TIFF is checked in as the sole runtime lunar elevation source, and the superseded floating-point TIFF is removed.
- [ ] Adjacent provenance records the NASA source URL, retrieval date, checksum, dimensions, half-meter units, 20,000-sample offset, conversion to kilometers, usage guidance, and a link to the retained format research.
- [ ] Native rendering, web rendering, and golden-render tests all consume the same new source asset.
- [ ] Asset loading accepts unsigned 16-bit TIFF samples, converts every sample to floating-point kilometers relative to the lunar reference radius, and rejects floating-point, malformed, and unsupported TIFF inputs.
- [ ] Decoder unit tests follow Arrange-Act-Assert and cover representative positive, zero, and negative elevations as well as each required rejection case.
- [ ] Existing golden fixtures remain untouched and golden comparisons pass. If they initially fail, amplified differences are visually inspected and the proceed-or-roll-back decision is recorded before completion.
- [ ] On the same machine and toolchain, the release Wasm is at least 2.0 MB smaller raw and 0.8 MB smaller with gzip than the baseline captured by Ticket 01.
- [ ] Repeated cold-load measurements through first framebuffer presentation are compared with the baseline and recorded; any clear initialization regression is investigated before completion.
- [ ] Native smoke tests, web smoke tests, relevant warmed web performance checks, and the canonical quality gate pass.
