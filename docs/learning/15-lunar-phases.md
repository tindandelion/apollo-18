# Animating lunar phases

A **lunar phase** is the visible pattern of illumination set by the angle between the viewing direction and the **Sun direction**. Apollo 18 keeps the camera and lunar globe fixed for this stage. Only the Sun direction changes, so the animation demonstrates illumination rather than object motion.

## Sun–Moon–viewer geometry

The camera sits on the world `-Z` side of the lunar globe and looks toward `+Z`. The center of the visible lunar disk therefore has an outward terrain normal near `-Z`. Recall that Apollo 18 defines Sun direction as the unit direction from the lunar globe toward the Sun.

At full Moon, the Sun is on the viewer's side:

```text
Sun direction = (0, 0, -1)
```

Visible terrain normals point generally toward the Sun, so Lambert's dot product illuminates almost the entire disk. At new Moon, the Sun is on the far side:

```text
Sun direction = (0, 0, +1)
```

Visible terrain normals then point generally away from the Sun, leaving the disk dark. Quarter phases place the Sun along the world `X` axis and split the visible disk near its vertical centerline.

## A ten-second cycle

Let `t` be explicit scene time in seconds and `T = 10` seconds. The cycle angle is

```text
θ = 2π · ((t mod T) / T)
```

The Sun moves at constant angular speed around lunar north (`+Y`) in the world `XZ` plane:

```text
s(t) = (-sin θ, 0, -cos θ)
```

This gives the canonical north-up progression:

| Scene time | Sun direction | Appearance |
| --- | --- | --- |
| `0s` | `-Z` | full |
| `1.25s` | between `-Z` and `-X` | waning gibbous |
| `2.5s` | `-X` | left-lit quarter |
| `3.75s` | between `-X` and `+Z` | waning crescent |
| `5s` | `+Z` | new |
| `7.5s` | `+X` | right-lit quarter |
| `10s` | `-Z` | full again |

The globe's object rotation remains the identity transformation. Zero-degree longitude stays facing the camera, map lookup stays fixed, and the terrain normal at each geographic location does not move in world space.

## Terrain shading through the cycle

Each fragment retains the terrain-normal Lambertian response from the previous stage:

```text
diffuse = max(dot(n_terrain, s(t)), 0)
linear_output = linear_lunar_color · diffuse
```

There is no ambient or specular term. Apollo 18 also does not add a separate smooth-sphere illumination mask, so tilted terrain normals can affect the apparent terminator. Geometry displacement, self-shadowing, and cast shadows remain outside this renderer stage.

## Deterministic host timing

The shared renderer derives the Sun direction only from scene time. Native sequences derive that time from absolute frame index divided by requested frame rate. The web host converts monotonic `requestAnimationFrame` timestamps into elapsed scene time. Neither host accumulates phase updates, so equal scene times produce equal framebuffers regardless of frame rate.
