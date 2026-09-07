# Apollo 18

Apollo 18 explores 3D graphics by building a renderer that presents the Moon using real lunar surface data.

## Language

**Software renderer**:
A renderer whose graphics pipeline runs on the CPU and produces a framebuffer without using GPU rasterization.
_Avoid_: CPU renderer, software rasterizer

**Framebuffer**:
A tightly packed, top-to-bottom array of RGBA pixels produced by the software renderer for image or browser presentation.
_Avoid_: Image buffer, canvas buffer

**Canvas backing resolution**:
The width and height of the HTML canvas's stored pixel grid, which matches the presented framebuffer dimensions independently of the canvas's displayed size.
_Avoid_: Canvas internal resolution, canvas size

**Canvas CSS dimensions**:
The displayed width and height of the HTML canvas in CSS pixels, independently of its backing resolution.
_Avoid_: Displayed resolution, canvas size

**Scene time**:
Non-negative finite elapsed seconds supplied explicitly to a scene render, used to derive deterministic animation state independently of frame rate or host clock.
_Avoid_: Current time, frame time

**Animation epoch**:
The UTC instant captured once when a lunar animation starts. Together with scene time, it identifies the astronomical instant presented by a frame.
_Avoid_: Scene time, current time

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
Surface data that assigns terrain elevation to locations on the lunar globe, measured relative to the lunar reference radius.
_Avoid_: Height texture, bump map, displacement map

**Lunar reference radius**:
The 1,737.4 km spherical radius relative to which lunar elevation is measured.
_Avoid_: Mean radius, sea level

**Octasphere**:
A spherical triangular mesh formed by repeatedly subdividing an octahedron and projecting the resulting vertices onto a sphere.
_Avoid_: Icosphere, sphere mesh

**Globe location**:
The unit direction from the center of the lunar globe through a surface location, expressed in the globe's object space before rotation. It identifies longitude and latitude independently of elevation and the terrain normal.
_Avoid_: Radial direction, surface direction, normal

**Lunar phase**:
The visible pattern of illumination determined by the angle between the viewing direction and the Sun direction.
_Avoid_: Lighting phase

**Lunar globe pose**:
The rotation from lunar-globe object space into world space.
_Avoid_: Globe transform, object rotation

**Lunar appearance**:
The rendering state that combines lunar globe pose and world-space Sun direction for one lunar globe frame.
_Avoid_: Lunar scene, phase settings

**Sub-Earth point**:
The location on the lunar globe directly facing Earth's center at a given UTC instant, expressed as lunar longitude and latitude. Its motion presents geocentric lunar libration without depending on an observer's location on Earth.
_Avoid_: Moon center position, observer position

**Lunar position angle**:
The apparent counterclockwise angle from celestial north to the Moon's north-pole axis at a given UTC instant. It describes the lunar disk's apparent roll in an Earth-centered view.
_Avoid_: Libration angle, globe yaw

**Synodic month**:
The interval over which the Moon returns to the same phase relative to the Sun and Earth. Apollo 18 uses its mean duration of 29.530588853 days when presenting one lunar-phase cycle.
_Avoid_: Lunar rotation period, sidereal month

**Sun direction**:
The world-space unit direction from the lunar globe toward the Sun, used as the incoming-light direction for lunar illumination.
_Avoid_: Light direction, direction of light travel

**Terrain normal**:
The local surface orientation at a lunar-globe location, derived from elevation gradients on the lunar reference radius.
_Avoid_: Globe location, geometry normal, displacement normal
