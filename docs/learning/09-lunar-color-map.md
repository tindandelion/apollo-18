# Mapping the lunar globe

The octasphere carries an object-space **globe location** at each vertex. The
software renderer interpolates the three locations with the fragment's
barycentric weights and normalizes the result:

```text
r = normalize(w0 r0 + w1 r1 + w2 r2)
```

Keeping this location in object space makes the lunar color map rotate with
the globe. It also avoids longitude seams in the mesh: no UV coordinates or
duplicated seam vertices are needed. The barycentric sum of unit globe
locations is not itself unit, so interpolation renormalizes once. Map
sampling then consumes that unit globe location without normalizing again.

## From a globe location to a map pixel

Apollo 18 defines lunar north as `+Y` and zero-degree longitude as `-Z`, which
faces the initial camera. For normalized `r = (x, y, z)`:

```text
longitude = atan2(x, -z)
latitude  = asin(y)
u = wrap(1/2 + longitude / 2π)
v = clamp(1/2 - latitude / π, 0, 1)
```

Longitude wraps because `-180°` and `+180°` meet at the same meridian.
Latitude clamps because the poles are boundaries, not another seam. The
nearest-neighbor lookup uses `floor(u × width)` and
`min(floor(v × height), height - 1)`. The lunar elevation map uses this same
lookup so both maps stay on one texel for a given globe location.

The lookup happens per fragment rather than per vertex. Interpolating map
coordinates across a triangle would cross the longitude seam incorrectly;
interpolating the seam-free globe location first avoids that discontinuity.

## sRGB and linear light

JPEG stores display-oriented sRGB channel values. Apollo 18 decodes a sampled
channel `s` to linear light before it enters the renderer's color calculations:

```text
linear(s) = s / 12.92                         when s ≤ 0.04045
linear(s) = ((s + 0.055) / 1.055)^2.4        otherwise
```

When writing the framebuffer, the inverse transfer function converts linear
light back to sRGB. With no lighting or filtering in this milestone, that
decode/encode round trip preserves the sampled 8-bit lunar color while keeping
the representation ready for later Lambertian shading.
