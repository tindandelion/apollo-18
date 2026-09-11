# Deriving terrain normals from lunar elevation

The lunar globe's silhouette remains spherical. Crater and mare relief comes from a **terrain normal**: a lighting orientation tilted by elevation gradients on the 1,737.4 km **lunar reference radius**. Elevation changes lighting only; it does not move octasphere vertices.

Apollo 18 derives one normalized object-space terrain normal at every lunar-elevation-map texel center during map construction. Fragment shading then selects that cache with the same nearest-texel lunar-coordinate policy used by the source map. Moving this invariant work out of fragment shading is what makes the capped high-density software renderer practical in WebAssembly.

## Texel-center location and tangent frame

For column `x` and row `y` in a map of width `W` and height `H`, the texel-center longitude and latitude are:

```text
λ = ((x + 1/2) / W - 1/2) · 2π
φ = (1/2 - (y + 1/2) / H) · π
```

The corresponding unit **globe location** `û` and tangent directions are:

```text
û     = (sin λ cos φ, sin φ, -cos λ cos φ)
east  = (cos λ, 0, sin λ)
north = east × û
```

Because texel centers do not lie exactly at either pole, this frame is defined for every cached sample. Polar *rows* still receive special gradient treatment because eastward physical distance approaches zero there.

## Physical slopes on the reference sphere

Finite differences supply `∂h/∂λ` and `∂h/∂φ`, where elevation `h` is measured in kilometers. Physical slopes use only the lunar reference radius `R = 1737.4 km`:

```text
slope_east  = (∂h/∂λ) / (R cos φ)
slope_north = (∂h/∂φ) / R
perturbed   = û − slope_east · east − slope_north · north
n_object    = normalize(perturbed)
```

Using `R + h` would describe displaced geometry, but Apollo 18 draws an undisplaced sphere. Constant elevation produces `n_object = û`.

## Discrete gradients

For the 1440×720, four-pixels-per-degree elevation map:

```text
Δλ = 2π / W
Δφ = π / H

∂h/∂λ ≈ (h_east − h_west) / (2 Δλ)
∂h/∂φ ≈ (h_north − h_south) / (2 Δφ)
```

East and west neighbors wrap across the antimeridian. Interior rows use central latitude differences. The first and last rows use one-sided latitude differences and set `slope_east = 0`, avoiding division by the very small polar-row circumference.

After construction, `LunarElevationMap` retains its dimensions and one three-`f32` normal per texel, but no elevation samples. The canonical normal cache occupies about 12.44 MB, 8.29 MB more than the former 4.15 MB elevation storage. The measured prototype increased median lunar-map setup from 216.5 ms to 251 ms; this one-time startup cost replaces repeated work in every rendered fragment.

## Object-space lighting

A lunar globe pose `Q` rotates object space into world space. Rotations preserve dot products, so rotating the Sun in the opposite direction once per frame is equivalent to rotating every sampled normal into world space:

```text
s_object = Q⁻¹ · s_world = Qᵀ · s_world

diffuse = max(dot(n_object, s_object), 0)
        = max(dot(Q · n_object, s_world), 0)
```

The renderer therefore transforms the world-space **Sun direction** once and performs Lambertian illumination directly in object space. Lunar coordinates still select both color and terrain data, so terrain remains attached through libration and position-angle roll.

Nearest-texel normal sampling intentionally introduces a small lighting quantization relative to deriving a tangent frame at every fragment. Visual review accepted that tradeoff. The color-map resolution, elevation-map resolution, octasphere, phase, orientation, and Canvas 2D presentation path remain unchanged.
