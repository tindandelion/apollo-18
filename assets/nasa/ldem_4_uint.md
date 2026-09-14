# NASA CGI Moon Kit lunar elevation map

- File: `ldem_4_uint.tif`
- Dimensions: 1440×720, unsigned 16-bit, 4 pixels per degree
- Source units: half meters relative to the 1,737.4 km lunar reference radius
- Source encoding: 20,000 added to each signed half-meter sample
- Runtime conversion: `(sample - 20,000) / 2,000` kilometers
- Source page: <https://svs.gsfc.nasa.gov/4720/>
- Direct download: <https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_4_uint.tif>
- Retrieved: 2026-09-14
- SHA-256: `e6668bec27fc9b8fbb02d198c7ddfb08eedeeb790167b494f95e6b34201da05e`
- Format research: [`docs/research/nasa-lunar-elevation-formats.md`](../../docs/research/nasa-lunar-elevation-formats.md)

NASA's Scientific Visualization Studio publishes this uncompressed TIFF as
part of the CGI Moon Kit. It is centered on 0° longitude and supplies the
canonical lunar elevation map for Apollo 18's native host, web host, and golden
render tests. Asset loading converts the unsigned source samples to
floating-point kilometers; consumers should not interpret the stored integers
as elevations directly.

## Usage and credit

NASA's Scientific Visualization Studio asks users to credit this item to
“NASA's Scientific Visualization Studio.” NASA's general
[media usage guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/)
state that NASA content is generally not subject to copyright in the United
States, while NASA identifiers and any credited third-party material have
separate restrictions.

This data provenance and usage guidance is independent of Apollo 18's
`MIT OR Apache-2.0` code license.
