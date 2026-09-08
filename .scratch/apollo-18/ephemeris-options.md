# Moon orientation and illumination data options

> **Historical research note:** The checked-in NASA source and nearest-record recommendation were adopted. The current-UTC playback proposal below was later rejected; [ADR-0006](../../docs/adr/0006-use-checked-in-nasa-lunar-ephemeris-data.md) records the authoritative data-derived synodic-month and ephemeris-span policies.

## Recommendation

Use NASA Scientific Visualization Studio's **Dial-A-Moon annual data** as the ephemeris source. Commit the original annual JSON unchanged with provenance rather than calculating positions locally or depending on a runtime service. It already supplies the values the renderer needs:

- `subearth.lon` / `subearth.lat`: the point at the center of the Earth-facing lunar disk (libration)
- `subsolar.lon` / `subsolar.lat`: the lunar point beneath the Sun
- `posangle`: the lunar north-pole position angle, if the scene should reproduce apparent roll instead of remaining lunar-north-up

NASA publishes hourly annual JSON; for example, [2026 data](https://svs.gsfc.nasa.gov/vis/a000000/a005500/a005587/mooninfo_2026.json) is linked from the official [Moon Phase and Libration, 2026](https://svs.gsfc.nasa.gov/5587) page. The [Dial-A-Moon help](https://svs.gsfc.nasa.gov/help/) documents the fields and a timestamp API.

Do not fetch the endpoint directly from the Apollo 18 browser. As checked on 2026-09-07, its CORS response allows `https://tempo.multiverse.music`, not the Apollo 18 origin. Serve the checked-in original annual JSON with the static application instead. Use the nearest complete hourly record for each frame, resolving exact half-hour ties toward the later record. This keeps subsolar coordinates, sub-Earth coordinates, and position angle together without interpolation or angular-wrap machinery.

The web and native hosts should capture the current UTC instant once when an animation starts. Tests and golden generation should inject the fixed `2026-01-01T00:00:00Z` animation epoch and consume the checked-in data; they should never call NASA over the network. An animation must fail before rendering if the data does not cover the complete following synodic month.

Map the ten-second scene timeline to astronomy time as:

```text
astronomy_time(t) = start_utc + (t / 10 seconds) × 29.530588853 days
```

for `0 <= t < 10 seconds`. This uses a mean synodic month. A real ephemeris is not exactly periodic over that duration, so forcing a seamless loop would introduce a small discontinuity or make the data less faithful. Decide explicitly whether realism or a perfect loop wins.

Shared ephemeris parsing and nearest-record selection belong behind a focused module in the renderer crate so native and web behavior cannot drift; host wall-clock access remains outside it. The rasterizer should receive explicit globe pose and Sun direction. Converting supplied lunar longitude/latitude and position angle to Apollo 18 vectors and transforms is rendering coordinate conversion, not an astronomical position calculation.

## Other options

### JPL Horizons API

[JPL Horizons](https://ssd-api.jpl.nasa.gov/doc/horizons.html) is the best source when exact arbitrary timestamps are preferred. An observer ephemeris for target Moon (`COMMAND='301'`) from the geocenter (`CENTER='500@399'`) can request quantities `14,15`:

- quantity 14: sub-observer longitude/latitude
- quantity 15: sub-solar longitude/latitude

A unitless `STEP_SIZE='300'` divides start-to-stop into 300 equal intervals; Horizons returns both endpoints (301 rows), so omit the final duplicate timeline sample for a 300-frame sequence. The API returns a JSON envelope whose `result` field is a text table; `CSV_FORMAT='YES'` simplifies parsing. The [Horizons manual](https://ssd.jpl.nasa.gov/horizons/manual.html) defines the lunar cartographic conventions.

This is excellent for an offline asset-generation script or native tool, but not for direct use by the GitHub Pages browser: the tested response had no `Access-Control-Allow-Origin` header. It also adds network availability and response-format concerns.

### Skyfield (Python) with JPL kernels

[Skyfield's planetary-frame guide](https://rhodesmill.org/skyfield/planetary.html) gives official examples for both the lunar sub-solar point and lunar libration using a JPL ephemeris plus lunar frame/PCK kernels. It can also return the raw lunar-frame rotation matrix. This is a strong build-time generator when accuracy and control matter, but it requires Python, NumPy, and several kernels and is more setup than Dial-A-Moon or Horizons.

### NAIF SPICE

NAIF SPICE directly exposes [`subpnt_c`](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/subpnt_c.html) and [`subslr_c`](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/subslr_c.html), backed by JPL kernels. It is the most capable and explicit option, but kernel management, C/Rust FFI, time kernels, and WebAssembly integration make it a poor fit for the requested simple solution.

### Astronomy Engine

[Astronomy Engine](https://github.com/cosinekitty/astronomy) is small, browser-capable, and provides lunar libration and body orientation. An unofficial Rust FFI package exists as [`astronomy-engine-bindings`](https://docs.rs/astronomy-engine-bindings/2.1.19/astronomy_engine_bindings/). However, its direct libration result covers the sub-Earth point, while obtaining the sub-solar point requires combining vector/orientation APIs. The Rust package wraps bundled C through unsafe FFI, and its WebAssembly fit would need proving. It is attractive only if fully offline runtime computation becomes more important than keeping astronomical calculations out of Apollo 18.
