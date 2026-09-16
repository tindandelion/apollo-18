# High-resolution lunar maps for 4K displays

## Question

Which NASA lunar color and elevation maps are appropriate when a lunar globe is presented on a 4K display, and how would denser maps affect Apollo 18's browser frame rate?

## Recommendation

NASA SVS's **4096×2048 color map** and **5760×2880 `ldem_16` elevation map** are the smallest official CGI Moon Kit products that can supply roughly one source sample per displayed pixel across a near-full-height globe on a 3840×2160 display. The 8192×4096 color map and `ldem_64` elevation map are unnecessary for this target.

Do not replace both bundled maps without changing the delivery and elevation-cache design:

- At the 1152×1152 backing-resolution cap used for this research, denser maps did not unlock more visible resolution. They only increased download, initialization, and cache costs.
- At that measured cap, the 4K color map alone reduced median measured throughput from 29.74 to 27.27 FPS (8.3%). The `ldem_16` elevation map alone reduced it to 22.83 FPS (23.2%). Together they reduced it to 22.25 FPS (25.2%).
- Raising the backing cap is much more expensive than changing map density. At 2048×2048, the current maps measured 9.75 FPS; the denser pair measured 9.09 FPS. Most of the fall from the roughly 30 FPS measured at 1152×1152 therefore came from processing 3.16 times as many framebuffer pixels, not from the maps themselves.
- The `ldem_16` terrain-normal cache would occupy 199.1 MB instead of 12.4 MB. The measured compressed Wasm payload estimate grew from 3.0 MB to 33.8 MB, and time from document start through the first completed frame grew from a 556 ms median to 1,618 ms.

A staged path is safer: adopt a runtime-compressed derivative of the 4096×2048 color map only when the backing cap is raised, then investigate a multiresolution or tiled terrain-normal representation before adopting `ldem_16` globally.

## NASA resources

NASA's [CGI Moon Kit](https://svs.gsfc.nasa.gov/4720/) is the best fit because these maps are explicitly designed for 3D rendering, use the same 0°-centered equirectangular convention as the current assets, and are already the project's source family. NASA describes the 2025 color map as an aesthetic rather than scientific product. It combines LROC WAC bands and fills latitudes outside the source mosaic's 70°N–70°S coverage with lower-resolution LOLA albedo data.

### Color

The CGI Moon Kit offers the 2025 color map as linear float16 EXR and 16-bit sRGB TIFFs. Its official download list contains:

| Product | Dimensions | Published size | Visible-hemisphere samples | 4K-display fit |
| --- | ---: | ---: | ---: | --- |
| Current `lroc_color_2k.jpg` | 2048×1024 | 447.2 KB | about 1024 | Matches the current approximately 1037-pixel globe, not a near-2160-pixel globe |
| `lroc_color_16bit_srgb_4k.tif` | 4096×2048 | 59.0 MB | about 2048 | Best available match |
| `lroc_color_16bit_srgb_8k.tif` | 8192×4096 | 232.0 MB | about 4096 | More detail than a 4K panel can show across the disk |
| `lroc_color_16bit_srgb_16k.tif` | 16384×8192 | 909.4 MB | about 8192 | Unnecessary |

A globe shows approximately half of an equirectangular map's longitude samples across its disk. A near-full-height globe on a 2160-pixel-tall panel therefore needs roughly 4320 map samples around the equator. The published 4096-wide product is the closest non-oversized option.

Apollo 18 currently decodes an 8-bit JPEG into RGB samples and does not decode 16-bit color TIFF. A production change would need a checked-in, runtime-compressed 8-bit sRGB derivative (for example JPEG or WebP), with provenance recording the NASA TIFF and conversion. The performance experiment used a quality-95 JPEG derivative because on-frame sampling cost depends on the decoded 4096×2048 RGB array, not its transport encoding. It was not evaluated for final visual quality.

The underlying LROC [Hapke mosaic README](https://pds.lroc.asu.edu/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/MDR/WAC_HAPKE/WAC_HAPKE_README.TXT) documents a 400 m/pixel source mosaic assembled from approximately 124,300 WAC images, so the 4K CGI map is a rendering-oriented downsample rather than a limit of the scientific source data.

### Elevation

NASA says the CGI Moon Kit elevation products are reformatted LOLA global cylindrical products at 4, 16, and 64 pixels per degree. Their samples are relative to the 1,737.4 km lunar reference radius. Unsigned products store half-meter values after adding 20,000.

| Product | Dimensions | Published size | Visible-hemisphere samples | Terrain-normal cache in Apollo 18 | 4K-display fit |
| --- | ---: | ---: | ---: | ---: | --- |
| Current `ldem_4_uint.tif` | 1440×720 | 2.0 MB | about 720 | 12.4 MB | Below display resolution |
| `ldem_16_uint.tif` | 5760×2880 | 31.7 MB | about 2880 | 199.1 MB | Smallest official product above the target |
| `ldem_64_uint.tif` | 23040×11520 | 506.3 MB | about 11520 | 3.19 GB | Far beyond the target and impractical for the current Wasm design |

The authoritative PDS [`LDEM_16.LBL`](https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/cylindrical/img/ldem_16.lbl) specifies 5760 samples × 2880 lines, 16 pixels/degree, a 0.5 m scaling factor, the 1,737,400 m reference-radius offset, and approximately 1,895.21 m/pixel map scale.

Although `ldem_16` is the correct source resolution, directly substituting it is a poor runtime representation. Per ADR-0003, Apollo 18 precomputes and retains three `f32` components for every elevation texel. Increasing each map dimension by four therefore multiplies the persistent cache by sixteen. A future implementation should test tiled/lazy caches, packed normals, and mip levels against visual quality and frame-rate goals.

## Performance experiment

### Method

Measurements used the repository's release Playwright timeline diagnostic on 2026-09-15:

- Apple M3 Pro, arm64, Darwin 24.6.0
- Headless Playwright Chromium 151.0.7922.34
- Normal advancing ephemeris-span animation
- Two-second warmup and eight-second measurement
- Three warmed runs per variant
- Exactly one software render, `ImageData` construction, and Canvas presentation per counted frame

The 4096×2048 color candidate was converted from NASA's 16-bit sRGB TIFF to an 8-bit quality-95 JPEG for the experiment. The elevation candidate was NASA's unmodified `ldem_16_uint.tif`. Each candidate temporarily replaced the existing include path; all substitutions and diagnostic changes were reverted after measurement.

At 1152×1152, only map dimensions changed:

| Variant | Completed FPS runs | Median FPS | Complete-frame runs | Median frame | Change from baseline |
| --- | --- | ---: | --- | ---: | ---: |
| Current 2K color + `ldem_4` | 29.82, 28.81, 29.74 | 29.74 | 33.0, 34.2, 33.0 ms | 33.0 ms | — |
| 4K color + `ldem_4` | 27.27, 27.22, 28.24 | 27.27 | 36.1, 36.2, 34.9 ms | 36.1 ms | −8.3% FPS; +9.4% frame time |
| 2K color + `ldem_16` | 22.83, 22.82, 22.87 | 22.83 | 43.4, 43.3, 43.3 ms | 43.3 ms | −23.2% FPS; +31.2% frame time |
| 4K color + `ldem_16` | 22.21, 22.25, 22.20 | 22.25 | 44.35, 44.65, 44.65 ms | 44.65 ms | −25.2% FPS; +35.3% frame time |

All differences exceed the corresponding run-to-run spans. The elevation cache is the dominant map-density cost.

A second diagnostic temporarily raised the cap to 2048×2048 and DPR to 3 so that the 730.625 CSS-pixel canvas reached the cap:

| Variant | Completed FPS runs | Median FPS | Complete-frame runs | Median frame |
| --- | --- | ---: | --- | ---: |
| Current maps | 10.10, 9.72, 9.75 | 9.75 | 96.0, 101.45, 100.4 ms | 100.4 ms |
| 4K color + `ldem_16` | 9.13, 9.09, 8.74 | 9.09 | 107.5, 107.8, 113.4 ms | 107.8 ms |

The denser pair added about 7.4% median frame time at this resolution. More importantly, increasing framebuffer area from 1.33 million to 4.19 million pixels reduced even the current-map median from 29.74 to 9.75 FPS. The two resolution series were sequential rather than interleaved, so this cross-series comparison is directional; the within-series map comparisons are the stronger evidence.

### Delivery, initialization, and memory

| Metric | Current pair | 4K color + `ldem_16` pair |
| --- | ---: | ---: |
| Encoded map bytes embedded for the experiment | about 2.5 MB | about 34.7 MB |
| Release Wasm | 5.1 MB | 37 MB |
| Gzipped Wasm estimate | 3.0 MB | 33.8 MB |
| Decoded color RGB array | 6.3 MB | 25.2 MB |
| Persistent terrain-normal cache | 12.4 MB | 199.1 MB |
| Median document start through first completed frame | 556 ms | 1,618 ms |

Initialization measurements used five fresh pages per variant and include local navigation, Wasm startup, map decoding/cache construction, and the first frame. They isolate neither network transfer nor cache construction, but show the user-visible local startup effect. Real deployment startup would additionally pay for roughly 31 MB more compressed transfer unless maps were split, cached, or streamed.

## Decision guidance

1. **Keep the current assets at the production backing-resolution cap.** A denser map cannot add commensurate visible detail at the current 1024 cap; the research reached the same conclusion at 1152.
2. **If targeting sharper 4K presentation, start with the 4096×2048 color source.** It is the right sampling scale and has a much smaller runtime cost than `ldem_16`.
3. **Do not expect 30 FPS from the current scalar software renderer at a 2048 backing dimension.** The measured result was around 9–10 FPS even before denser maps.
4. **Treat elevation as an architecture task, not an asset swap.** Preserve `ldem_16` as the likely source, but benchmark packed/multiresolution/tiled terrain normals and lower backing-resolution policies before bundling a 199 MB cache.
5. **Offer quality adaptively.** A higher-resolution still mode, reduced animation cadence, or resolution tiers are more plausible near-term than continuously software-rendering a 2K disk at 30 FPS.

## Sources

- NASA Scientific Visualization Studio, [CGI Moon Kit](https://svs.gsfc.nasa.gov/4720/) — official download dimensions/sizes, color-map construction and limitations, elevation formats and resolutions.
- NASA PDS Geosciences Node, [`LDEM_16.LBL`](https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/cylindrical/img/ldem_16.lbl) — authoritative LOLA dimensions, resolution, projection, units, scaling, and reference radius.
- LROC PDS, [WAC Hapke mosaic README](https://pds.lroc.asu.edu/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/MDR/WAC_HAPKE/WAC_HAPKE_README.TXT) — source mosaic coverage, scale, bands, and observation count.
- Apollo 18 [`docs/learning/13-high-density-web-rendering.md`](../learning/13-high-density-web-rendering.md) — current map-to-framebuffer sampling rationale.
- Apollo 18 [`docs/adr/0003-derive-terrain-normals-on-the-reference-sphere.md`](../adr/0003-derive-terrain-normals-on-the-reference-sphere.md) — current persistent terrain-normal representation.
- Apollo 18 [`docs/testing.md`](../testing.md) — performance diagnostic methodology and prior reference measurements.
