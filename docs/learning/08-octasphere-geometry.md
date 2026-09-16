# Building an octasphere

The first **lunar globe** replaces the cube with an **octasphere**: a triangular mesh made by subdividing an octahedron and projecting every generated vertex onto a sphere. This produces understandable spherical geometry without importing a model.

## Begin with an octahedron

A unit octahedron has six vertices on the coordinate axes and eight triangular faces. Apollo 18 aligns two of those vertices with the lunar axis:

- lunar north points along world `+Y`;
- lunar south points along world `-Y`;
- zero-degree longitude points toward the initial camera along world `-Z`.

The remaining equatorial vertices point along `+X`, `+Z`, and `-X`. Faces retain Apollo 18's outward-facing winding convention, which becomes clockwise when each face is viewed from outside under the renderer's left-handed coordinates.

Subdivision level zero is this original eight-triangle octahedron.

## Split one triangle into four

For a triangle with unit globe locations `a`, `b`, and `c`, calculate one midpoint on each edge:

```text
ab = normalize(a + b)
bc = normalize(b + c)
ca = normalize(c + a)
```

Normalization is the spherical projection. A plain average lies inside the sphere; dividing it by its length moves it back to radius one. The original triangle is replaced by four triangles:

```text
(a,  ab, ca)
(ab, b,  bc)
(ca, bc, c)
(ab, bc, ca)
```

Their ordering preserves the parent's winding. Adjacent triangles share edge midpoints through an indexed mesh, so the same geometric location receives one vertex index rather than being calculated twice.

Every subdivision replaces each triangle with four. Starting from eight faces, level `n` therefore has

```text
triangles(n) = 8 × 4ⁿ
```

The canonical level-5 globe contains 8,192 triangles and 4,098 shared vertices.

## Place the globe in the frame

Mesh positions remain unit globe locations. Each host supplies a **frame-relative radius** `r`, and the object transformation scales the mesh by that value while keeping it centered at the world origin. Native output and rendering fixtures use `r = 0.45`, leaving 10% of the shorter framebuffer dimension as total padding. The web host uses `r = 0.5`, producing an edge-to-edge globe. The camera remains at `(0, 0, -3)` and looks along `+Z`.

The orthographic view spans one world-space unit across the shorter framebuffer dimension. If `s` is that shorter dimension, its aspect-corrected half-extents are

```text
half_width  = width  / (2 × s)
half_height = height / (2 × s)
```

A globe with frame-relative radius `r` therefore has projected diameter

```text
diameter_pixels = 2 × r × s
```

The host can choose padding without changing the projection, and the globe remains circular in square, landscape, and portrait framebuffers.

## Rotate from scene time

The lunar globe rotates around lunar north, the `+Y` axis, once every 10 seconds. For scene time `t`, its yaw is

```text
loop_time = t mod 10 s
yaw(t) = 360° × loop_time / 10 s
```

At zero seconds, zero-degree longitude points toward the camera along object-space `-Z`. The generated colors remain attached to their object-space globe locations, so their movement makes the rotation visible.

As with the earlier cube animation, each frame is a pure function of explicit scene time. Native sequence timestamps come independently from frame index and frame rate, while the web adapter converts monotonic `requestAnimationFrame` timestamps into elapsed scene time. No host accumulates rotation frame by frame.

## Generate a color from globe location

The octasphere milestone gives every shared vertex a deterministic sRGB color derived from its globe location `(x, y, z)`:

```text
red   = (x + 1) / 2
green = (y + 1) / 2
blue  = (1 - z) / 2
```

Each channel maps the directional range `[-1, 1]` into sRGB `[0, 1]`. Thus lunar north is green and the camera-facing zero-degree-longitude direction is blue. This spatial rule gives every canonical mesh vertex a distinct color without storing an arbitrary palette.

## Enter the established pipeline

Each indexed triangle is expanded into three NDC vertices and sent through the existing viewport, back-face-culling, edge-function, and depth-buffer stages. The generated sRGB vertex colors are decoded to linear RGB before barycentric interpolation and encoded back to sRGB for the framebuffer. The resulting gradients make interpolation and orientation visible while the dense level-5 silhouette demonstrates subdivision. Later stages will replace these generated colors with lunar color-map samples derived per fragment from globe location.
