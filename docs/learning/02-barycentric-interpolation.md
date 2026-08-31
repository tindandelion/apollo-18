# Barycentric color interpolation

The second triangle stage uses the values that decide pixel coverage to interpolate a color at every covered pixel. Its vertices keep the first stage's normalized positions and carry sRGB red, green, and blue. The software renderer converts those inputs to linear RGB before interpolation and encodes the result back to sRGB when writing the framebuffer.

## One edge calculation, two jobs

For a directed edge from `A` to `B`, Apollo 18 evaluates a pixel-center sample `P` with:

```text
edge(A, B, P) = (B.x - A.x) * (P.y - A.y)
              - (B.y - A.y) * (P.x - A.x)
```

The triangle is oriented so its signed area is positive:

```text
area = edge(V0, V1, V2)
```

The three edge values opposite the vertices are:

```text
e0 = edge(V1, V2, P)
e1 = edge(V2, V0, P)
e2 = edge(V0, V1, P)
```

A sample is covered when all three values are positive, with exact zero handled by the shared-edge rule below. Dividing those same values by the triangle area produces barycentric weights:

```text
w0 = e0 / area
w1 = e1 / area
w2 = e2 / area
```

For a covered sample, the weights are non-negative and sum to one. Each weight is one at its vertex and zero along the opposite edge. Reusing the coverage values makes this relationship explicit and prevents separate coverage and interpolation calculations from disagreeing.

## Affine interpolation

Given linear vertex colors `C0`, `C1`, and `C2`, the sample color is their weighted sum:

```text
C = w0 * C0 + w1 * C1 + w2 * C2
```

This is affine interpolation in screen space. Orthographic rendering can use it directly. A later perspective renderer would need perspective-correct interpolation for attributes whose screen-space variation is not affine.

The complete vertex, including its color, moves when the rasterizer normalizes triangle winding. Swapping only positions would detach colors from their vertices and change the image.

## Why interpolate in linear RGB?

sRGB values are encoded for storage and display; their numeric values are not proportional to light intensity. Averaging encoded bytes therefore produces a result that is too dark. For example:

```text
50% linear intensity -> about 0.735 sRGB -> byte 188
50% sRGB byte value  -> 0.5 sRGB       -> byte 128
```

Apollo 18 treats sRGB as its external representation and linear RGB as the renderer's working representation. An sRGB channel `S` normalized to `0..1` is decoded with:

```text
L = S / 12.92                         when S <= 0.04045
L = ((S + 0.055) / 1.055) ^ 2.4       otherwise
```

After interpolation, a linear channel `L` is clamped to `0..1` and encoded with:

```text
S = 12.92 * L                          when L <= 0.0031308
S = 1.055 * L ^ (1 / 2.4) - 0.055     otherwise
```

The encoded value is multiplied by `255` and rounded to the nearest integer. Alpha remains `255` and is not interpolated.

## The top-left ownership rule

Two triangles that form a surface can share an edge. A sample exactly on that edge satisfies both triangles' geometric boundary. If both triangles include it, the second draw overwrites the first. If both exclude it, the background shows through as a crack. A deterministic rule must assign the sample to exactly one triangle.

Apollo 18 uses the **top-left rule**. In top-left framebuffer coordinates, where `Y` increases downward, a directed edge owns its on-edge samples when it:

- goes upward (`end.y < start.y`), or
- is horizontal and goes left-to-right.

Consider a quad split along `A-C`:

```text
A-------B
| T0  / |
|   /   |
| /  T1 |
D-------C
```

The triangles are `A-B-C` and `A-C-D`, oriented for positive edge values in framebuffer coordinates. They traverse the shared diagonal in opposite directions:

| Triangle | Directed shared edge | Classification | Owns on-edge samples? |
| --- | --- | --- | --- |
| `A-B-C` | `C -> A` | upward | yes |
| `A-C-D` | `A -> C` | downward | no |

The possible policies differ at pixel centers on the diagonal:

```text
both inclusive:  T0 and T1 write -> overlap depends on draw order
both exclusive:  neither writes  -> background crack
top-left rule:   only T0 writes  -> one stable owner
```

Reversing draw order therefore produces the same framebuffer. Horizontal shared edges work the same way: the left-to-right traversal owns exact samples, while the opposite traversal does not.

The term “top-left” names the convention, not a test that an entire edge literally appears only at the top or left of a triangle. The directed-edge classification is the precise rule.

## Degenerate and off-screen triangles

A triangle with exactly zero signed area has no interior, so it performs no writes. Clockwise and counter-clockwise triangles are both accepted at this 2D stage; negative-area input is reoriented before coverage and interpolation.

The rasterizer clamps the triangle's traversal bounds to the framebuffer. A fully off-screen triangle visits no pixels, while a partially visible triangle visits only in-bounds candidates. This bounds work and prevents invalid framebuffer writes without adding geometric clipping to this stage.
