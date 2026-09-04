# Lighting and rotating the lunar globe

The color-mapped lunar globe now combines two directions with different jobs.
Its object-space **globe location** continues to locate a fragment in the
lunar color map. A rotated, normalized copy supplies the smooth lighting
normal in world space, so the map turns beneath a fixed directional Sun.

## Smooth spherical lighting normals

For barycentric weights `w0`, `w1`, and `w2`, interpolate the three rotated
vertex directions and normalize the result:

```text
n = normalize(w0 n0 + w1 n1 + w2 n2)
```

Normalization matters because a weighted sum of unit vectors is generally
shorter than one. Using the unnormalized sum would incorrectly darken the
interiors of triangles and reveal the octasphere tessellation.

This lighting normal is derived from the globe's spherical shape. Later,
per-fragment terrain normals from lunar elevation replace that spherical
lighting vector; see [deriving terrain normals](14-terrain-normals.md).
The Lambertian response and globe rotation in this guide stay the same.

## Linear-light Lambertian shading

Apollo 18 defines **Sun direction** `s` as the unit direction from the lunar
globe toward the Sun. Lambertian diffuse intensity is:

```text
diffuse = max(dot(n, s), 0)
```

A surface facing the Sun receives full intensity, a perpendicular surface
receives zero, and a surface facing away remains unlit. The neutral Sun scales
all three sampled linear RGB channels equally:

```text
shaded_linear_rgb = lunar_map_linear_rgb × diffuse
```

The lunar map is decoded from sRGB before this multiplication. The shaded
linear result is encoded to sRGB only when written to the framebuffer.
Multiplying encoded sRGB values would perform lighting in a nonlinear display
representation and produce incorrect brightness. Ambient and specular terms
are both zero.

At scene time zero, the Sun is 30 degrees toward camera-right from the
globe-to-camera direction:

```text
s = (sin(30°), 0, -cos(30°))
```

That lights most of the visible disk while leaving a terminator on the left.
Unlit lunar fragments approach black, while untouched pixels retain the
opaque sRGB `#181818` background.

## Rotation from scene time

The globe rotates around lunar north (`+Y`) once every ten seconds. For scene
time `t`, its yaw is derived directly:

```text
loop_time = t mod 10 s
yaw(t) = 360° × loop_time / 10 s
```

The native host samples `t = frame_index / frames_per_second`; the web host
derives `t` from the browser's monotonic `requestAnimationFrame` timestamp.
Neither host accumulates rotation updates, so equal scene times always produce
equal framebuffers regardless of frame rate or dropped browser frames.
