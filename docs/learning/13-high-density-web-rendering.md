# High-density web rendering

An HTML canvas has a displayed size and a stored image size. Its **canvas CSS dimensions** determine how much page layout space it occupies, measured in CSS pixels. Its **canvas backing resolution** determines how many pixels the canvas stores. Apollo 18 presents one software-renderer framebuffer pixel in each backing pixel, so the framebuffer dimensions and backing resolution must agree.

A CSS pixel is a layout unit rather than necessarily one physical display pixel. The browser reports the relationship through the device pixel ratio (DPR). A canvas displayed at width `w_css` and height `h_css` ideally requests

```text
w_desired = w_css × DPR
h_desired = h_css × DPR
```

backing pixels. Thus, a 600×600 CSS-pixel canvas on a DPR-2 display ideally has a 1200×1200 backing resolution without becoming any larger on the page.

## Bounding useful detail

Software-rendering work grows approximately with pixel count:

```text
work ∝ width × height
```

Doubling both dimensions therefore creates about four times as many fragments to process. Unrestricted DPR would make the workload depend heavily on display density.

Apollo 18's 2048×1024 equirectangular lunar color map contains approximately 1024 source samples across a visible hemisphere. The globe occupies 90% of the shorter framebuffer dimension, so a maximum backing dimension of 1152 produces an approximately 1037-pixel globe. Rendering substantially beyond that cannot add comparable lunar-map detail.

The web host computes the desired floating-point dimensions, applies one uniform scale factor when either exceeds 1152, and then rounds both dimensions to integers:

```text
scale = min(1, 1152 / max(w_desired, h_desired))
w_backing = round(w_desired × scale)
h_backing = round(h_desired × scale)
```

Uniform scaling preserves aspect ratio. Positive results are clamped to at least one pixel. A zero CSS width or height skips rendering until the canvas has an area; malformed measurements are errors.

## Following responsive layout

CSS continues to decide the canvas's displayed dimensions. Each animation callback measures those dimensions and DPR, then changes the backing resolution only if the selected integer dimensions differ. Rasterization and Canvas 2D presentation are skipped while the CSS canvas is zero-sized, but scene time continues from the first animation callback. Resizing therefore changes detail without restarting the lunar phase.

The backing image is no longer presented with forced nearest-neighbor `pixelated` scaling. Its display sharpness instead comes from rendering near the density requested by the display, within the lunar-map-derived cap.

## Measured tradeoff

In the reference release-browser workload, a 730.625×730.625 CSS-pixel canvas at DPR 2 requested more than the cap and selected a 1152×1152 backing resolution. The initial high-density implementation measured 15.42 FPS, establishing the cost of sharper output. Subsequent profiling and renderer optimization raised the same bounded workload above the project's 30 FPS target without reducing its backing resolution.
