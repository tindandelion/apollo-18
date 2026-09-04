# NASA CGI Moon Kit lunar elevation map

- File: `ldem_4.tif`
- Dimensions: 1440×720, 32-bit floating-point, 4 pixels per degree
- Units: kilometers relative to the 1,737.4 km lunar reference radius
- Source page: <https://svs.gsfc.nasa.gov/4720/>
- Intended direct download: <https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_4.tif>
- Retrieved: 2026-09-04
- SHA-256: `d876c867612e8941d775a005b2bc1ebaef5c15f97e04a43022a71fc21f5c9d65`

NASA's Scientific Visualization Studio publishes this product as an uncompressed
floating-point TIFF centered on 0° longitude. The SVS file returned HTTP 403 at
retrieval time, so this copy was reconstructed from the same LOLA gridded data
product NASA documents as the Moon Kit source:

- PDS Geosciences Node `LDEM_4` (`LRO-L-LOLA-4-GDR-V1.0`)
- Direct download: <https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/cylindrical/img/ldem_4.img>
- Label: <https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/cylindrical/img/ldem_4.lbl>
- Conversion: signed 16-bit little-endian half-meter samples divided by 2000 to
  kilometers, then rolled 180° in longitude so 0° sits at the image center,
  matching the Moon Kit map that shares this project's lunar color map.

## Usage and credit

NASA's Scientific Visualization Studio asks users to credit this item to
“NASA's Scientific Visualization Studio.” LOLA gridded data is archived by the
PDS Geosciences Node. NASA's general
[media usage guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/)
state that NASA content is generally not subject to copyright in the United
States, while NASA identifiers and any credited third-party material have
separate restrictions.

This data provenance and usage guidance is independent of Apollo 18's
`MIT OR Apache-2.0` code license.
