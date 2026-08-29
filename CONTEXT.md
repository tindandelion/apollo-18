# Apollo 18

Apollo 18 explores 3D graphics by building a renderer that presents the Moon using real lunar surface data.

## Language

**Software renderer**:
A renderer whose graphics pipeline runs on the CPU and produces a framebuffer without using GPU rasterization.
_Avoid_: CPU renderer, software rasterizer

**Framebuffer**:
A tightly packed, top-to-bottom array of RGBA pixels produced by the software renderer for image or browser presentation.
_Avoid_: Image buffer, canvas buffer

**Lunar globe**:
A global, three-dimensional depiction of the Moon whose surface appearance is derived from lunar map data.
_Avoid_: Moon model, lunar model

**Lunar color map**:
Surface data that assigns visible color to locations on the lunar globe.
_Avoid_: Texture, Moon image

**Lunar elevation map**:
Surface data that assigns terrain elevation to locations on the lunar globe.
_Avoid_: Height texture, bump map

**Octasphere**:
A spherical triangular mesh formed by repeatedly subdividing an octahedron and projecting the resulting vertices onto a sphere.
_Avoid_: Icosphere, sphere mesh

**Radial direction**:
The unit direction from the center of the lunar globe through a surface location, used to identify its longitude and latitude independently of elevation and lighting normals.
_Avoid_: Surface direction, normal

**Lunar phase**:
The visible pattern of illumination determined by the angle between the viewing direction and the Sun direction.
_Avoid_: Lighting phase

**Terrain normal**:
The local surface orientation derived from lunar elevation gradients for lighting, distinct from the globe's radial direction.
_Avoid_: Radial direction, geometry normal
