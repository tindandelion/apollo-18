---
status: superseded by ADR-0005
---

# Gate terrain shading to the lunar day side

A fragment is eligible for direct lunar illumination only when its world-space globe location has a positive dot product with the Sun direction; eligible fragments retain terrain-normal Lambertian intensity. Terrain normals can otherwise point toward the Sun from the undisplaced sphere's geometric night side, producing rim highlights at exact new Moon that displaced geometry, visibility, and shadowing might reject. The gate preserves ADR-0003's spherical geometry and terrain relief while making the large-scale lunar phase follow the globe location.

## Considered options

Preserving terrain-normal Lambertian shading unchanged was rejected because it leaves visible illumination at exact new Moon. Multiplying by smooth-sphere Lambertian intensity was rejected because it applies additional cosine attenuation across the day side, and a smooth transition was deferred because its angular width would be arbitrary. Geometry displacement and shadowing remain outside the first renderer's scope.

## Consequences

The geometric terminator is a hard illumination boundary: terrain normals vary brightness within the day side but cannot move highlights onto the night side. Exact new Moon has no directly illuminated visible lunar fragments. Canonical phase renders and learning material must distinguish geometric day-side eligibility from local terrain-normal intensity.
