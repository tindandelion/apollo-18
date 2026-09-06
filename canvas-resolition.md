# Canvas resolution and Retina displays

The confusing part is that a canvas has **two different sizes**, and the screen introduces a third.

## The three sizes

### 1. CSS display size

This controls how large the canvas appears in the webpage:

```css
canvas {
  width: 800px;
  height: 800px;
}
```

These are **CSS pixels**, which are browser layout units. They are not necessarily physical screen pixels.

### 2. Canvas backing resolution

This is the canvas's internal pixel grid:

```html
<canvas width="800" height="800"></canvas>
```

Or in JavaScript:

```js
canvas.width = 800;
canvas.height = 800;
```

This determines how many actual pixels the canvas can store. In Apollo 18, it must also match the framebuffer resolution produced by the software renderer.

### 3. Physical display size

The browser eventually displays the CSS box using the monitor's physical pixels. Device pixel ratio, or DPR, describes the relationship:

```text
physical pixels ≈ CSS pixels × DPR
```

## Example on a normal DPR-1 display

Suppose we have:

```text
CSS display size:     800 × 800 CSS pixels
Canvas backing size:  800 × 800 pixels
Device pixel ratio:   1
```

The canvas occupies approximately:

```text
800 × 800 physical pixels
```

Everything maps cleanly:

```text
1 canvas pixel → 1 CSS pixel → 1 physical pixel
```

## The same canvas on a DPR-2 Retina display

Now suppose the device has DPR 2:

```text
CSS display size:     800 × 800 CSS pixels
Canvas backing size:  800 × 800 pixels
Device pixel ratio:   2
```

The same CSS box occupies approximately:

```text
1600 × 1600 physical pixels
```

But the canvas contains only 800×800 pixels. The browser must enlarge it:

```text
1 canvas pixel → 1 CSS pixel → 2 × 2 physical pixels
```

Conceptually:

```text
One canvas pixel:

+---+

Displayed using four physical pixels:

+---+---+
|   |   |
+---+---+
|   |   |
+---+---+
```

The canvas remains the same apparent size, but its image can look soft or blocky because it does not contain enough pixels for the display.

## A Retina-sized backing canvas

To make the same 800-CSS-pixel canvas sharp at DPR 2:

```text
CSS display size:     800 × 800 CSS pixels
Canvas backing size: 1600 × 1600 pixels
Device pixel ratio:   2
Physical display:    1600 × 1600 physical pixels
```

Now the mapping is approximately:

```text
1 canvas pixel → 1 physical pixel
```

The canvas does **not** become larger on the page. It only gains a denser internal pixel grid.

A conventional Canvas 2D application might configure that as:

```js
const cssWidth = 800;
const cssHeight = 800;
const dpr = window.devicePixelRatio;

canvas.style.width = `${cssWidth}px`;
canvas.style.height = `${cssHeight}px`;

canvas.width = Math.round(cssWidth * dpr);
canvas.height = Math.round(cssHeight * dpr);
```

## How this applies to Apollo 18

Apollo 18 has an additional stage because it creates every pixel in its Rust software renderer:

```text
Lunar maps
    ↓
Rust software renderer
    ↓
RGBA framebuffer
    ↓
Canvas backing pixels
    ↓
CSS-sized canvas
    ↓
Physical display pixels
```

The framebuffer and canvas backing dimensions must agree.

Currently, the web host does approximately this:

```text
Rust framebuffer:     800 × 800
Canvas backing size:  800 × 800
CSS display size:     responsive
```

For Retina support, it would instead do something like:

```text
Canvas CSS size:      700 × 700 CSS pixels
Device pixel ratio:   2
Selected resolution: 1400 × 1400 pixels

Rust framebuffer:    1400 × 1400
Canvas backing size: 1400 × 1400
```

Rust renders 1,960,000 pixels instead of 640,000 pixels. The browser then displays those pixels inside the same 700×700 CSS box.

## Why CSS size must be measured

Apollo 18's canvas is responsive:

```css
canvas {
  width: 100%;
  height: auto;
}
```

Therefore, `800px` is only its maximum area—not necessarily its actual displayed size.

For example, on a smaller browser window:

```text
Actual CSS size:     500 × 500
DPR:                 2
Ideal backing size: 1000 × 1000
```

The application should use the canvas's measured dimensions, commonly obtained with:

```js
const rect = canvas.getBoundingClientRect();

const idealWidth = Math.round(rect.width * window.devicePixelRatio);
const idealHeight = Math.round(rect.height * window.devicePixelRatio);
```

## Canvas resolution versus lunar-map resolution

These are separate resolutions:

```text
Lunar color map:       2048 × 1024 source samples
Lunar elevation map:   1440 × 720 source samples
Framebuffer:           selected render resolution
Canvas backing store:  same as framebuffer
CSS display size:      page layout size
Physical display:      CSS size multiplied by DPR
```

Increasing the framebuffer resolution improves how accurately Apollo 18 rasterizes:

- The circular silhouette
- Triangle boundaries
- The terminator
- Terrain-lighting transitions

It does not create more lunar source data. If the framebuffer becomes much denser than the lunar maps, multiple framebuffer pixels will sample the same lunar-map value.

## Why we need a resolution cap

A full DPR calculation can become expensive. For an 800×800 CSS canvas:

| DPR | Backing resolution | Pixel count |
|---:|---:|---:|
| 1 | 800×800 | 640,000 |
| 1.5 | 1200×1200 | 1,440,000 |
| 2 | 1600×1600 | 2,560,000 |
| 3 | 2400×2400 | 5,760,000 |

DPR 2 requires four times as many rendered pixels as DPR 1. DPR 3 requires nine times as many.

Because Apollo 18 renders those pixels on the CPU rather than the GPU, unrestricted native DPR may be too expensive. The ticket therefore calls for a cap, conceptually:

```text
desired resolution = CSS size × DPR
selected resolution = desired resolution limited by pixel budget
```

For Apollo 18, an 800×800 CSS canvas at DPR 2 ideally wants 1600×1600. The color-map-derived policy instead caps it at 1152×1152 because denser rendering would provide little additional lunar surface detail from the current map. That does not achieve perfect one-to-one physical-pixel mapping, but it should still look noticeably sharper than 800×800. The initial frame-rate reduction is expected and is handled by a separate performance-tuning ticket.

The key distinction is:

> **CSS dimensions control how large the canvas looks. Backing dimensions control how much visual detail it can contain. DPR tells us how dense the physical display is.**
