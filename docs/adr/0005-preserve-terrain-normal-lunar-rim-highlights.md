# Preserve terrain-normal lunar rim highlights

Apollo 18 preserves terrain-normal Lambertian shading without a separate globe-location illumination mask. At exact new Moon, a small number of visible fragments near the rim can therefore be illuminated when their terrain normals point toward the far-side Sun, even though the undisplaced sphere places them on its geometric night side. This is accepted as an honest and visually preferable consequence of showing elevation through lighting without displaced geometry, visibility, or shadows.

## Considered options

A hard globe-location day-side gate removed the highlights but produced an objectionably sharp geometric terminator. A two-degree one-sided `smoothstep` fade kept exact new Moon black but still made the terminator look less natural than terrain-normal shading alone. Geometry displacement, self-shadowing, and cast shadows could resolve visibility more coherently, but remain outside the first renderer's scope.

## Consequences

ADR-0004 is superseded. Exact new Moon may retain sparse rim highlights, and terrain normals may move the apparent terminator away from the reference sphere's geometric terminator. Learning material and golden renders should present this as a limitation of terrain-normal shading on undisplaced geometry rather than a rasterization or phase-timing defect.
