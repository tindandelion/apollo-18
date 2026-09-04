# Deriving terrain normals from lunar elevation

The lunar globe's silhouette stays a sphere. Crater and mare relief comes from
a **terrain normal**: a lighting orientation tilted by elevation gradients on
the 1,737.4 km **lunar reference radius**. Object-space **globe location**
still selects both the lunar color map and the lunar elevation map. Elevation
tilts that location into a perturbed radial in the same object space. Globe
rotation carries the perturbed radial into world space, where one normalize
produces the lighting vector for Lambert's dot with the fixed Sun.

NASA publishes the source file as a displacement map. Apollo 18 does not move
vertices. Elevation only changes the lighting vector, so the octasphere remains
a perfect sphere.

## Physical slopes on the reference sphere

A fragment has unit globe location `û` and elevation `h` in kilometers. With
longitude `λ` and latitude `φ` from the same convention as color-map lookup:

```text
λ = atan2(x, -z)
φ = asin(y)
```

the east and north unit tangents can be derived directly from the globe
location. Let `ρ = hypot(x, z)`, the location's radius around the lunar north
axis:

```text
east  = (-z / ρ, 0, x / ρ)
north = east × û
```

This is algebraically equivalent to reconstructing the tangents from `λ` and
`φ`, but avoids additional trigonometric functions. At an exact pole `ρ = 0`,
longitude—and therefore the tangent frame—is inherently ambiguous. Apollo 18
uses the antimeridian convention `east = -X`; crossing with `û` then gives the
corresponding north tangent.

Finite differences supply `∂h/∂λ` and `∂h/∂φ`. Physical slopes use the
reference radius `R = 1737.4 km` only, not the local radius `R + h`, because
the rasterized surface is still that sphere:

```text
slope_east  = (∂h/∂λ) / (R cos φ)
slope_north = (∂h/∂φ) / R
perturbed   = û − slope_east · east − slope_north · north
n           = normalize(perturbed)
```

A constant `h` leaves the perturbed radial equal to `û`, so `n = û`. A slope
that rises toward the Sun brightens; the opposite wall darkens.

## Discrete gradients

The elevation map is 4 pixels per degree, so `Δλ = Δφ = 0.25°`. Each fragment
calculates `λ` and `φ` once and shares those coordinates between the lunar
color and elevation map lookups. The fragment's `û` still provides `û`, east,
north, and `cos φ`. Only `h` is quantized with the shared nearest-neighbor
lookup: `floor` into a texel, wrap longitude, clamp latitude.

Interior texels use 4-connected central differences of that texel's neighbors:

```text
∂h/∂λ ≈ (h_east − h_west) / (2 Δλ)
∂h/∂φ ≈ (h_north − h_south) / (2 Δφ)
```

East and west wrap, so a crater that crosses 180° longitude has a continuous
gradient. Polar rows have no latitude neighbor on one side and `cos φ → 0`.
Those rows take a one-sided latitude difference and set `slope_east = 0`, which
keeps the polar normal in the local meridian and avoids dividing by zero.

## Lighting

`n` is the object-space **terrain normal**. Yaw is a pure rotation, so
normalizing `perturbed` before rotating it is the same lighting vector as
rotating first and normalizing once:

```text
n_world = normalize(R · perturbed)
diffuse = max(dot(n_world, s), 0)
```

Map lookup continues to use unrotated globe location, so geography stays
painted on the surface while the Sun stays fixed in the world.
