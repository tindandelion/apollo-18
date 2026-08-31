# First triangle

Apollo 18 begins with a software renderer that produces a framebuffer: tightly packed RGBA bytes ordered left-to-right within each row, with rows stored top-to-bottom. Each pixel is opaque in this first milestone. The clear color is the fixed sRGB background `#181818`.

The first milestone defined a triangle with `0..1` positions relative to the framebuffer. At that stage, the renderer converted those positions directly into continuous pixel space with:

```text
x = normalized_x * framebuffer_width
y = normalized_y * framebuffer_height
```

Pixels occupy unit squares in that space. Pixel `(0, 0)` covers `x = 0..1` and `y = 0..1`, so its sample point is the pixel center `(0.5, 0.5)`.

## Why sample the pixel center?

A triangle is continuous geometry, while a framebuffer is a grid of discrete square pixels. The renderer therefore needs a coverage rule that turns partial geometric overlap into a binary decision: fill the pixel or leave it unchanged.

Apollo 18 represents each pixel with one sample at its center. For pixel `(x, y)`, that sample is `(x + 0.5, y + 0.5)`. The pixel is filled when this sample lies inside the triangle. A triangle that touches only a corner of a pixel without covering its center does not fill that pixel.

Center sampling does not claim that the entire pixel lies inside the triangle. It is a simple, consistent approximation of coverage. More advanced renderers can take several samples per pixel or calculate covered area to produce anti-aliasing; this first stage deliberately uses one sample so the rasterization rule remains explicit and deterministic.

## Testing the sample against the triangle

Triangle coverage is decided with edge functions. For a directed edge from `A` to `B`, an edge function measures which side of the edge contains a point `P`:

```text
edge(A, B, P) = (B.x - A.x) * (P.y - A.y)
              - (B.y - A.y) * (P.x - A.x)
```

After orienting the triangle consistently, a pixel belongs to the triangle when its center is inside all three directed edges. The evolved showcase instead receives validated normalized device coordinates and performs viewport conversion and culling before this rasterization step; see [Viewport conversion, culling, and depth](03-viewport-culling-and-depth.md).

Samples that land exactly on an edge need a deterministic ownership rule. Apollo 18 uses a top-left rule: an on-edge sample is included only for edges that go upward in framebuffer coordinates, or for horizontal edges that go left-to-right. This keeps later adjacent triangles from both filling the same pixel or leaving a crack along their shared edge. [Barycentric color interpolation](02-barycentric-interpolation.md) develops the normalized edge values and illustrates this ownership rule with adjacent triangles.

## Presenting the framebuffer on the web

The web host compiles Rust to WebAssembly. Its `start()` function runs when the WebAssembly module loads. It queries the page's document for the Apollo 18 canvas, sets the canvas's internal dimensions, obtains its Canvas 2D context, asks the shared software renderer for a framebuffer, and presents those RGBA pixels as `ImageData`.

This keeps the responsibilities separate: the shared software renderer decides pixel colors, while the web host manages browser objects and presentation.

Apollo 18 uses [Trunk](https://trunkrs.dev/) to build and serve the web page. Trunk coordinates the Rust-to-WebAssembly build and packages the resulting module with the page's HTML and CSS.

### Further exploration

Trunk is new technology for this project. A future learning exercise should explore its build pipeline, HTML asset directives, development server, configuration, and release output in more depth.
