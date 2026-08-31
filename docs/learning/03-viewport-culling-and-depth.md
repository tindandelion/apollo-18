# Viewport conversion, culling, and depth

The third triangle stage moves the scene into normalized device coordinates (NDC) and adds two visibility decisions: back-face culling removes triangles facing away from the viewer, and a depth buffer prevents farther fragments from replacing nearer ones.

## Normalized device coordinates

An NDC position is valid only when all components are finite and:

```text
-1 <= x <= 1
-1 <= y <= 1
 0 <= depth <= 1
```

`+Y` points up. Depth `0` is the near plane and depth `1` is the far plane. The renderer rejects invalid NDC positions instead of clamping them, because clamping would silently move geometry and hide an error in an earlier pipeline stage.

The types on either side of viewport conversion are deliberately distinct. An NDC vertex cannot be mistaken for a vertex measured in framebuffer pixels.

## From NDC to the viewport

For a framebuffer with dimensions `width` and `height`, viewport conversion produces continuous pixel coordinates:

```text
screen_x = (ndc_x + 1) * width / 2
screen_y = (1 - ndc_y) * height / 2
```

The left and right NDC boundaries map to `0` and `width`. The top and bottom boundaries map to `0` and `height`. These are continuous boundaries, not necessarily pixel centers. Rasterization still samples pixel `(x, y)` at `(x + 0.5, y + 0.5)` and bounds traversal to valid pixel indices.

## From NDC winding to screen-space culling

Clockwise winding in NDC defines a front-facing triangle. Viewport conversion inverts `Y`, so the converted front face has positive signed area in top-left framebuffer coordinates. The rasterizer uses this known reversal to classify the converted triangle with one screen-space signed-area calculation:

```text
screen_area = edge(screen_v0, screen_v1, screen_v2)
```

A positive screen-space area corresponds to a clockwise, front-facing NDC triangle. A negative area corresponds to a counter-clockwise, back-facing NDC triangle. Zero is degenerate, including a very small NDC triangle whose vertices collapse together because of floating-point rounding during viewport conversion. Non-positive areas are discarded before any pixels are visited.

The accepted triangle already has the orientation expected by the edge-function rasterizer: its area and its interior edge values are positive. The same positive area is passed directly into rasterization for barycentric normalization. No vertex reordering or sign adjustment is needed between culling and coverage.

## Affine orthographic depth

The same screen-space barycentric weights used for color interpolate normalized depth:

```text
z = w0 * z0 + w1 * z1 + w2 * z2
```

This affine interpolation is appropriate for Apollo 18's orthographic pipeline. A future perspective projection would require perspective-correct treatment of interpolated attributes.

The depth buffer contains one floating-point value per framebuffer pixel and starts at positive infinity. A covered fragment passes only when:

```text
fragment_depth < stored_depth
```

Strictly smaller depth wins. Positive infinity allows fragments on the far plane at depth `1` to pass initially, while the complete normalized range remains usable. If two fragments have exactly equal depth, the first one remains the owner. A failed depth test changes neither the framebuffer color nor the stored depth.

The depth buffer belongs to the internal rasterizer. It is visibility state, not presentation data, so the returned framebuffer remains the same tightly packed RGBA interface used by the native and web hosts.

## The evolved triangle scene

The retained triangle showcase now draws partially overlapping near and far triangles. The near triangle is submitted first, so the later far triangle would incorrectly overwrite it without depth testing. A separate counter-clockwise triangle is also submitted and culled. This makes culling and depth behavior visible while retaining linear-light barycentric vertex-color interpolation.
