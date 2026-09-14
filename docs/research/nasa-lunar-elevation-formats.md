# NASA lunar elevation formats suitable for Apollo 18

Retrieved and verified: 2026-09-14.

## Finding

NASA provides two representations that are approximately half the size of Apollo 18's current 32-bit floating-point TIFF while preserving the source elevation precision and the same 1440×720 sampling grid.

### NASA SVS unsigned 16-bit TIFF

The NASA Scientific Visualization Studio CGI Moon Kit offers [`ldem_4_uint.tif`](https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_4_uint.tif) alongside the floating-point [`ldem_4.tif`](https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_4.tif). The [CGI Moon Kit page](https://svs.gsfc.nasa.gov/4720/) describes the unsigned files as 16-bit half-meter samples formed by adding 20,000 to the source signed values. Therefore, elevation relative to Apollo 18's 1,737.4 km lunar reference radius is:

```text
elevation_km = (unsigned_sample - 20_000) / 2_000
```

Inspection of the downloaded file found:

- Dimensions: 1440×720
- Sample type: unsigned 16-bit integer
- TIFF compression: none
- File size: 2,076,866 bytes
- SHA-256: `e6668bec27fc9b8fbb02d198c7ddfb08eedeeb790167b494f95e6b34201da05e`
- gzip-9 size: 1,914,611 bytes
- Brotli-11 size: 1,654,937 bytes

Every decoded sample agrees with Apollo 18's current floating-point TIFF to within `4.731e-7 km`; no sample differs by as much as `1e-6 km`. This is floating-point representation noise around the half-meter source values, not a meaningful terrain change.

### NASA PDS signed 16-bit IMG

The underlying PDS product is [`LDEM_4.IMG`](https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/cylindrical/img/ldem_4.img), documented by its [`LDEM_4.LBL`](https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/cylindrical/img/ldem_4.lbl). The label specifies:

- 720 lines × 1440 samples
- `SAMPLE_TYPE = LSB_INTEGER`
- `SAMPLE_BITS = 16`
- `SCALING_FACTOR = 0.5` meter
- `OFFSET = 1737400` meters
- uncompressed fixed-length records

Inspection of the downloaded file found:

- File size: exactly 2,073,600 bytes (`1440 × 720 × 2`)
- SHA-256: `c04632eba6449af49e3108ed7c25b3b1c450600abd3690df4fc815853a1af476`
- gzip-9 size: 1,912,979 bytes
- Brotli-11 size: 1,604,612 bytes

After rolling each row by 180° longitude to match Apollo 18's 0°-centered map, every sample agrees with the current TIFF to within `4.731e-7 km`.

## Size comparison

| Representation | Raw bytes | gzip-9 bytes | Notes |
|---|---:|---:|---|
| Current Apollo 18 float TIFF | 4,147,334 | 2,851,344 | 32-bit float, uncompressed TIFF |
| NASA SVS `ldem_4_uint.tif` | 2,076,866 | 1,914,611 | 16-bit unsigned TIFF, already centered at 0° |
| NASA PDS `ldem_4.img` | 2,073,600 | 1,912,979 | Headerless signed 16-bit data; requires longitude roll |

Either NASA representation saves approximately 2.07 MB of raw Wasm and 0.94 MB of gzip transfer compared with the current asset.

## Recommendation

Use NASA SVS `ldem_4_uint.tif` for the smallest behavior-preserving change: it is an official Moon Kit download, already has the orientation Apollo 18 expects, and only requires extending the decoder to convert unsigned half-meter samples using the documented 20,000 offset.

Use the PDS `LDEM_4.IMG` instead if minimizing decoder and container overhead is more important. It is 3,266 bytes smaller and can be decoded without the general-purpose TIFF crate, but Apollo 18 must explicitly implement and test the PDS dimensions, little-endian signed samples, half-meter scale, and 180° longitude roll.

The Moon Kit's `ldem_3_8bit.jpg` is much smaller (the NASA page lists 108.9 KB), but it has a lower 1024×512 spatial grid and only 8-bit values. It is lossy and should not be treated as a behavior-preserving replacement.
