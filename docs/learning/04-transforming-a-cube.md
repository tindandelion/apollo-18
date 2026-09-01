# Transforming an orthographic cube

The cube milestone adds three-dimensional positions before Apollo 18's existing normalized-device-coordinate (NDC) pipeline. Each cube vertex passes through object, view, and orthographic projection transformations before viewport conversion, culling, and depth testing.

## Left-handed coordinates

Apollo 18 uses a left-handed world coordinate system:

- `+X` points right;
- `+Y` points up;
- `+Z` points forward, away from the camera.

The canonical camera sits at `(0, 0, -3)` and looks along `+Z`. A point at the world origin is therefore three units in front of the camera. Cube faces are wound clockwise when seen from outside. This is the front-face convention established by the [viewport and culling stage](03-viewport-culling-and-depth.md).

## Object transformation

The unit cube begins centered at the world origin, with every coordinate ranging from `-0.5` to `0.5`. Its object transformation rotates it by `+30°` around Y and then by `-20°` around X:

```text
world_position = rotation_x(-20°) * rotation_y(+30°) * object_position
```

Matrix multiplication is written for column vectors, so the rightmost Y rotation acts first. The fixed rotation exposes the front, right, and top faces and gives the still image an unambiguous orientation.

## View transformation

A view transformation expresses world positions relative to the camera. The canonical camera has no rotation, so its view transformation is the inverse of its translation:

```text
view_position = translation(0, 0, +3) * world_position
```

For example, the world origin becomes `(0, 0, 3)` in view space. Positive view-space Z is forward depth under the left-handed convention.

Keeping object and view transformations separate matters even in this simple scene: object transformation poses the cube, while view transformation describes where the observer is.

## Orthographic projection

Orthographic projection preserves apparent size with depth. Apollo 18 uses `glam`'s left-handed orthographic matrix constructor, which has the required `[0, 1]` depth range. For horizontal bounds `left` and `right`, vertical bounds `bottom` and `top`, and depth bounds `near` and `far`, that matrix maps view space into NDC with:

```text
ndc_x = 2 * (view_x - left) / (right - left) - 1
ndc_y = 2 * (view_y - bottom) / (top - bottom) - 1
ndc_z =     (view_z - near) / (far - near)
```

The canonical bounds are:

```text
left = bottom = -1.25
right = top   = +1.25
near = 2
far  = 4
```

Thus the horizontal and vertical boundaries map to `[-1, 1]`, while the near and far planes map to normalized depths `0` and `1`. Unlike perspective projection, no division by view-space Z occurs, so screen-space barycentric interpolation remains affine.

## Entering the rasterizer

Each face duplicates its four vertices so it can carry one solid face color. The quad becomes two clockwise triangles. After object, view, and projection transformations, each projected position is validated as an NDC vertex and enters the established pipeline:

1. viewport conversion maps NDC into top-left framebuffer coordinates and reverses winding;
2. back-face culling rejects triangles whose screen-space signed area is non-positive;
3. edge functions determine covered pixel centers;
4. barycentric weights interpolate normalized depth and linear RGB;
5. the strict-less depth test keeps the nearest fragment.

Mirrored fixed views expose opposite side faces in tests. This verifies that cube winding, culling, and depth behavior remain coherent rather than merely producing one plausible still image.
