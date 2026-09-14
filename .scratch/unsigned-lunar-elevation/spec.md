# Unsigned lunar elevation source

## Goal

Reduce the release Wasm artifact while preserving lunar rendering behavior by replacing the current 32-bit floating-point lunar elevation TIFF with NASA Scientific Visualization Studio's unsigned 16-bit Moon Kit TIFF.

## Decisions

- NASA's `ldem_4_uint.tif` becomes the single canonical lunar elevation source for native rendering, web rendering, and golden-render tests.
- Runtime elevation storage remains floating-point kilometers. Asset loading converts each unsigned sample with `(sample - 20,000) / 2,000`.
- The decoder is dedicated to NASA's unsigned 16-bit Moon Kit representation and rejects floating-point, malformed, and unsupported TIFF inputs.
- The old floating-point asset, decoder path, and provenance document are removed rather than retained as alternatives.
- Existing golden fixtures should remain unchanged. If a comparison fails, inspect the amplified differences visually before deciding whether to accept the behavior or roll back the replacement.
- Measure raw and gzip Wasm sizes with reusable repository tooling. The replacement must reduce raw Wasm by at least 2.0 MB and gzip size by at least 0.8 MB on the same machine and toolchain.
- Compare repeated cold-load time through first framebuffer presentation. Record the result rather than imposing a fixed CI timing threshold, and investigate any clear regression.
- Retain the elevation-format research note and cross-link it from the selected asset's provenance document.
- This source-format selection does not introduce a domain term and does not warrant an ADR.

## Source

- NASA CGI Moon Kit: <https://svs.gsfc.nasa.gov/4720/>
- Direct asset: <https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_4_uint.tif>
- Verified dimensions: 1440×720
- Verified representation: uncompressed unsigned 16-bit TIFF
- Verified SHA-256 on 2026-09-14: `e6668bec27fc9b8fbb02d198c7ddfb08eedeeb790167b494f95e6b34201da05e`

NASA documents that the unsigned representation adds 20,000 to signed LOLA half-meter samples. Subtracting that offset and dividing by 2,000 converts a sample to kilometers relative to the 1,737.4 km lunar reference radius.
