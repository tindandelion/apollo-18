# Apollo 18

Apollo 18 explores 3D graphics by building a renderer that presents the Moon using real lunar surface data.

## Language

**Software renderer**:
A renderer whose graphics pipeline runs on the CPU and produces a framebuffer without using GPU rasterization.
_Avoid_: CPU renderer, software rasterizer

**Framebuffer**:
A tightly packed, top-to-bottom array of RGBA pixels produced by the software renderer for image or browser presentation.
_Avoid_: Image buffer, canvas buffer

**Scene time**:
Non-negative finite elapsed seconds supplied explicitly to a scene render, used to derive deterministic animation state independently of frame rate or host clock.
_Avoid_: Current time, frame time

**Normalized device coordinates (NDC)**:
The post-projection coordinate space where the visible horizontal and vertical ranges are `[-1, 1]`, `+Y` points up, and normalized depth ranges from near `0` to far `1`.
_Avoid_: Normalized framebuffer coordinates, screen coordinates

**Depth buffer**:
A per-pixel record of the nearest accepted normalized depth, used to prevent farther fragments from replacing nearer ones and kept distinct from the presentation framebuffer.
_Avoid_: Z-buffer, depth framebuffer

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
The unit direction from the center of the lunar globe through a surface location, expressed in the globe's object space before rotation. It identifies longitude and latitude independently of elevation and lighting normals.
_Avoid_: Surface direction, normal

**Lunar phase**:
The visible pattern of illumination determined by the angle between the viewing direction and the Sun direction.
_Avoid_: Lighting phase

**Terrain normal**:
The local surface orientation derived from lunar elevation gradients for lighting, distinct from the globe's radial direction.
_Avoid_: Radial direction, geometry normal
