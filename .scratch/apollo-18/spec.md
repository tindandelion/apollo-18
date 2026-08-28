Status: ready-for-agent

# Apollo 18: First Complete Lunar Software Renderer

## Problem Statement

The learner wants to understand 3D graphics by building a software renderer in Rust rather than adopting an existing rendering engine. The project needs a sequence of small, visible milestones that begins with triangle rasterization and culminates in a compelling animated lunar globe built from NASA CGI Moon Kit data.

The same rendering implementation must support native image and animation artifacts and an early WebAssembly showcase. The initial work must favor clarity and correctness over performance, avoid speculative GPU abstractions, and preserve enough automated visual evidence to expose regressions as the renderer evolves.

## Solution

Build Apollo 18 as a Rust workspace containing a platform-independent shared rendering library, a native host with a separate binary for each retained milestone, and an evolving one-page WebAssembly host. The shared library will implement triangle rasterization, affine attribute interpolation, orthographic 3D transforms, depth buffering, culling, clipping, octasphere generation, longitude-latitude map lookup from per-fragment radial direction, linear-light Lambertian shading, and terrain normals derived from lunar elevation data.

Development will proceed through complete native-and-web tracer bullets: a flat triangle, an interpolated triangle, a rotating cube, a smooth octasphere, a color-mapped lunar globe, a rotating lit globe, terrain-normal shading, and lunar-phase animation. Native binaries will produce still images and deterministic frame sequences; the webpage will display the same shared-library framebuffer through Canvas 2D.

The first complete renderer will remain CPU-only, single-threaded, orthographic, and intentionally unoptimized. Profiling, CPU/Wasm tuning, perspective projection, and any GPU renderer will be separate follow-up efforts.

## User Stories

1. As a graphics learner, I want to implement rasterization myself, so that I understand how triangles become pixels.
2. As a graphics learner, I want to use `glam` for vector and matrix arithmetic, so that the project focuses on graphics rather than rebuilding foundational math types.
3. As a graphics learner, I want the first render to be a hard-coded 2D triangle, so that rasterization can be understood independently of 3D transformations.
4. As a graphics learner, I want triangle coverage determined by edge functions, so that inside/outside testing is explicit and inspectable.
5. As a graphics learner, I want normalized edge values used as barycentric weights, so that interpolation follows naturally from triangle coverage.
6. As a graphics learner, I want to interpolate vertex colors, so that I can verify barycentric interpolation visually.
7. As a graphics learner, I want shared-edge behavior to be deterministic, so that adjacent triangles do not produce cracks or unstable overlap.
8. As a graphics learner, I want a depth buffer, so that nearer geometry correctly hides farther geometry.
9. As a graphics learner, I want back-face culling, so that triangle winding has a visible and useful role.
10. As a graphics learner, I want clipping against all six view-volume planes, so that partially visible triangles render robustly.
11. As a graphics learner, I want clipping implemented as a general polygon-against-plane operation, so that the renderer avoids a collection of special cases.
12. As a graphics learner, I want a rotating cube milestone, so that transforms, depth, winding, shared edges, and animation are validated before lunar geometry is introduced.
13. As a graphics learner, I want orthographic projection initially, so that the first 3D pipeline remains as simple as possible.
14. As a graphics learner, I want a documented left-handed coordinate convention, so that transform and winding behavior remains consistent.
15. As a graphics learner, I want `+Y` to mean up and `+Z` to mean forward, so that positive depth is intuitive.
16. As a graphics learner, I want normalized depth to run from zero at the near plane to one at the far plane, so that depth semantics remain clear and future GPU work is straightforward.
17. As a graphics learner, I want the framebuffer origin at the top left, so that rendered pixels map naturally to image and browser output.
18. As a graphics learner, I want viewport Y inversion and its winding consequence handled explicitly, so that coordinate conversion does not create hidden behavior.
19. As a graphics learner, I want to generate an octasphere from a subdivided octahedron, so that spherical geometry is built from understandable triangular operations.
20. As a graphics learner, I want octasphere subdivision to be selectable, so that quality and performance can be explored later.
21. As a viewer, I want the initial lunar globe to use a level-5 octasphere, so that its silhouette looks smooth at the canonical resolution.
22. As a viewer, I want the Moon's north aligned with world `+Y`, so that globe rotation follows a stable lunar axis.
23. As a viewer, I want the Moon Kit map's zero-degree longitude initially facing the camera, so that the familiar near side is centered.
24. As a viewer, I want the globe to rotate around lunar north, so that the animation presents the complete lunar surface coherently.
25. As a viewer, I want one globe rotation to last ten seconds, so that features remain inspectable while the showcase stays lively.
26. As a graphics learner, I want lunar map coordinates derived per fragment from radial direction, so that octasphere vertices do not need UV seams or pole duplication.
27. As a graphics learner, I want radial direction distinguished from terrain normal, so that geographic lookup and lighting have separate meanings.
28. As a viewer, I want the 2025 NASA 2048×1024 lunar color map, so that the globe uses a compact, current, visually compelling real-world data source.
29. As a viewer, I want nearest-neighbor lunar color-map sampling initially, so that the first implementation remains simple and inspectable.
30. As a graphics learner, I want longitude to wrap and latitude to clamp during map lookup, so that the full lunar globe samples valid source data.
31. As a graphics learner, I want sRGB lunar color converted to linear light before filtering or shading, so that lighting math is physically coherent.
32. As a viewer, I want final linear colors converted back to sRGB, so that the 8-bit framebuffer displays correctly.
33. As a viewer, I want a directional light representing the Sun, so that the lunar globe has a recognizable illuminated side and terminator.
34. As a graphics learner, I want Lambertian diffuse shading, so that the first lighting model is simple and explicit.
35. As a viewer, I want no ambient or specular contribution, so that the initial lunar lighting remains stark and uncluttered.
36. As a viewer, I want the rotating globe to begin in a gibbous illumination configuration, so that most terrain is visible while the terminator still reveals the spherical form.
37. As a viewer, I want shadows to fall onto a fixed sRGB `#181818` background, so that unlit lunar pixels remain distinguishable from the frame background.
38. As a graphics learner, I want the smooth globe shaded with per-fragment radial normals before elevation is introduced, so that mesh tessellation does not appear faceted.
39. As a graphics learner, I want terrain normals derived from the physical units in NASA's 4-pixels-per-degree floating-point lunar elevation map, so that elevation shading has a meaningful baseline.
40. As a graphics learner, I want neighboring elevation samples used to estimate local gradients, so that small-scale terrain affects lighting without increasing mesh density.
41. As a viewer, I want terrain-normal shading without geometry displacement initially, so that crater detail is visible while the globe silhouette remains simple.
42. As a viewer, I want a lunar-phase animation after terrain-normal shading is complete, so that the elevation detail can be observed under changing illumination.
43. As a viewer, I want a complete lunar-phase cycle to last twenty seconds, so that lighting transitions are easy to inspect.
44. As a viewer, I want the phase animation to keep the camera and globe fixed while changing Sun direction, so that the animation demonstrates illumination rather than object motion.
45. As a native user, I want each retained milestone to be a separate binary, so that triangle, cube, and lunar behavior remain independently runnable.
46. As a native user, I want a binary to render one deterministic frame at a specified scene time, so that any animation frame can be reproduced.
47. As a native user, I want a binary to render a numbered PNG sequence at a requested frame rate, so that external tools can encode an animation.
48. As a project maintainer, I want video encoding delegated to tools such as `ffmpeg`, so that codec complexity stays outside the renderer.
49. As a web viewer, I want the renderer to appear on a single static webpage, so that Apollo 18 can be published without an application server.
50. As a web viewer, I want animation time based on the browser's monotonic clock, so that rotation speed does not depend on rendering frame rate.
51. As a web viewer, I want the framebuffer displayed through Canvas 2D `ImageData`, so that Apollo 18's graphics pipeline remains CPU/Wasm-owned.
52. As a web viewer, I want the page to scale an 800×800 canvas visually without increasing its internal resolution, so that high-DPI displays do not silently multiply rendering cost.
53. As a web viewer, I want the initial page to animate automatically without controls, so that the first showcase remains focused.
54. As a project maintainer, I want local web output beginning with the first triangle, so that Wasm compatibility is continuously validated.
55. As a project maintainer, I want public static deployment immediately after the lunar color map milestone, so that the project becomes shareable as soon as it has a compelling lunar image.
56. As a project maintainer, I want the hosting provider selected only when deployment begins, so that deployment follows the repository's actual remote environment.
57. As a project maintainer, I want both native and web hosts to use the same rendering implementation, so that visual behavior does not drift between platforms.
58. As a project maintainer, I want JPEG and TIFF decoding shared between hosts but separated from rasterization, so that file-format knowledge remains localized.
59. As a project maintainer, I want hosts to provide encoded bytes while the shared image module decodes them, so that filesystem and browser-fetch concerns remain outside shared code.
60. As a project maintainer, I want NASA assets versioned with the project, so that builds and golden renders are reproducible and do not depend on runtime network access.
61. As a project maintainer, I want source URL, retrieval date, and checksum recorded for each NASA asset, so that its provenance is auditable.
62. As a project maintainer, I want NASA assets distinguished from the project's code license, so that provenance and reuse terms are not misrepresented.
63. As a project maintainer, I want milestone scenes rendered deterministically from explicit time and inputs, so that regressions can be reproduced.
64. As a project maintainer, I want small exact golden images for triangle and cube behavior, so that rasterization changes are caught precisely.
65. As a project maintainer, I want realistic lunar golden images, so that changes to mapping, lighting, and terrain normals are reviewed visually.
66. As a project maintainer, I want failed lunar comparisons to produce amplified difference images, so that regressions are easy to locate.
67. As a project maintainer, I want golden updates to require an explicit command, so that expected output cannot change accidentally.
68. As a project maintainer, I want canonical lunar renders to use fixed scene settings, so that image comparison remains meaningful.
69. As a graphics learner, I want concise learning documentation for each graphics stage, so that the reasoning and equations remain available after implementation.
70. As a project maintainer, I want code comments limited to local reasoning, so that implementation and tutorial material do not become interleaved.
71. As a project maintainer, I want Rust edition 2024 and an exact stable toolchain pin, so that native, Wasm, and golden output builds are reproducible.
72. As a project maintainer, I want formatting, linting, tests, and a release web build to form the local quality gate, so that the repository has one repeatable health check.
73. As a project maintainer, I want Trunk to build and serve the web host, so that Wasm packaging does not require custom build scripts.
74. As a project maintainer, I want the code available under MIT OR Apache-2.0, so that its reuse terms follow a familiar Rust convention.
75. As a desktop viewer, I want the published page to work in current Chrome, Firefox, and Safari, so that it is broadly accessible.
76. As a mobile viewer, I want the page not to break on a small screen, even if performance is initially best-effort, so that the showcase degrades gracefully.
77. As a graphics learner, I want phase one to remain single-threaded, so that concurrency does not obscure renderer correctness.
78. As a graphics learner, I want performance exploration to begin only after the complete lunar result exists, so that optimization is driven by a real workload.
79. As a graphics learner, I want the performance phase to target sustained 30 FPS at 800×800 in desktop Wasm, so that “satisfactory” has an observable meaning.
80. As a graphics learner, I want the reference machine and browser documented with performance results, so that measurements remain interpretable.
81. As a graphics learner, I want profiling and reasonable CPU/Wasm optimization attempted before GPU migration, so that the GPU decision is evidence-based.
82. As a future maintainer, I want scene and lunar data concepts kept independent from rasterization internals, so that later renderer changes do not require rewriting project data handling.

## Implementation Decisions

- Apollo 18 will be a Cargo workspace with three packages: a shared renderer library, one native-host package containing multiple binaries, and one WebAssembly host package.
- All workspace packages will live under one common package directory rather than separating libraries and applications by folder category.
- The project will use Rust edition 2024 with the stable `1.97.1` toolchain pinned for reproducibility.
- `glam` will provide vector and matrix arithmetic. Apollo 18 will implement transforms, clipping, rasterization, interpolation, depth buffering, map sampling, and lighting itself.
- The shared library will present one high-leverage deterministic frame-rendering seam. A caller selects a milestone scene and supplies explicit dimensions, scene time, and required assets; the result is an RGBA framebuffer.
- Native and web hosts are adapters at the presentation seam. They own clocks, filesystem or browser loading, command-line handling, PNG output, Canvas 2D display, and deployment concerns; they do not implement rendering behavior.
- The shared image module will decode JPEG and floating-point TIFF bytes into renderer-consumable image data. File-format handling will remain separate from rasterization modules.
- The canonical public render size will be 800×800. Smaller dimensions may be supplied to focused tests.
- The framebuffer will be tightly packed, row-major, top-to-bottom, 8-bit RGBA. Alpha is `255` in phase one.
- The fixed background will be sRGB `#181818` with alpha `255`.
- Triangle rasterization will begin in 2D screen space using bounding-box traversal and pixel-center edge-function tests. Normalized edge values will become barycentric weights for affine interpolation.
- The first triangle will use a flat color. Vertex-color interpolation will follow as a separate visible stage.
- Shared-edge coverage must use one documented ownership rule so that adjacent triangles do not crack or double-fill unpredictably.
- Apollo 18 will use the left-handed coordinate convention recorded in ADR-0001: `+Y` is up and `+Z` points forward.
- The camera will initially use orthographic projection. Perspective projection and perspective-correct interpolation are deferred.
- Normalized depth will map the near plane to `0` and the far plane to `1`; smaller values win.
- Front-facing triangles will be counter-clockwise before viewport conversion. The top-left framebuffer conversion reverses visible winding and must be handled explicitly.
- The first cube will remain within the view volume until basic transforms, culling, depth, and animation are working.
- Clipping will then operate in homogeneous clip space against all six view-volume planes. Clipped polygons will be triangulated for rasterization.
- The lunar globe mesh will be an octasphere generated by repeatedly subdividing an octahedron and normalizing generated vertices.
- The canonical globe uses subdivision level 5, yielding 8,192 triangles. Subdivision remains selectable for later measurement.
- Lunar north aligns with world `+Y`, zero-degree longitude initially faces the camera, and globe rotation is around the north axis.
- The smooth globe uses normalized interpolated radial direction as its per-fragment lighting normal.
- Lunar map lookup derives longitude and latitude per fragment from normalized radial direction. It does not store mesh UV attributes or duplicate seam and pole vertices.
- Longitude wraps horizontally and latitude clamps vertically when accessing lunar maps.
- The initial lunar color map is NASA's 2025 2048×1024 JPEG map centered on zero-degree longitude.
- The initial lunar elevation map is NASA's 1440×720, 4-pixels-per-degree floating-point TIFF. Values are kilometers relative to the 1,737.4 km lunar reference radius.
- Both lunar maps use nearest-neighbor access in phase one. Terrain gradients use direct neighboring elevation texels.
- Lunar color values are decoded from sRGB into linear RGB before lighting. Final linear output is encoded to sRGB for the 8-bit framebuffer.
- Lighting uses one neutral directional Sun light and Lambertian diffuse response. Ambient and specular contributions are zero.
- The first lit showcase uses a gibbous configuration with the Sun direction approximately 30 degrees from the viewing direction.
- The smooth lunar globe rotates once every ten seconds. Rendering is a pure function of explicit elapsed scene time rather than accumulated frame steps.
- Terrain-normal shading leaves octasphere geometry spherical. Per-fragment terrain normals are derived from lunar elevation gradients using the source's physical units and lunar reference radius.
- Geometry displacement is deferred. The silhouette remains spherical in this spec.
- The lunar-phase animation follows terrain-normal shading. The camera and globe remain fixed while Sun direction completes one cycle every twenty seconds.
- The native package will retain separate binaries for the triangle, cube, and lunar milestones. Shared native-host behavior may live behind the package's library interface.
- Native animation output consists of deterministic single frames or numbered PNG sequences. External tools perform video encoding.
- The WebAssembly host will be one evolving webpage rather than a demo selector. Earlier visual milestones remain available through native binaries and tests.
- The web host will use `requestAnimationFrame` for elapsed time and Canvas 2D `ImageData` for display. It will not use WebGL or WebGPU.
- The canvas has a fixed 800×800 internal resolution and may scale responsively through CSS without following device-pixel ratio.
- The initial webpage has no interaction controls beyond responsive presentation.
- Trunk, `wasm-bindgen`, and `web-sys` will provide web build and browser plumbing.
- Local web rendering begins with the first framebuffer milestone. Public static deployment begins immediately after the lunar color-map milestone.
- The deployment provider remains undecided until a remote exists. GitHub Pages is preferred if the eventual remote is GitHub.
- NASA data will be committed as project assets rather than fetched at render time. Each asset will carry source URL, retrieval date, and checksum provenance.
- The code will be dual-licensed under MIT OR Apache-2.0. NASA data provenance and usage terms will be documented separately.
- Current desktop Chrome, Firefox, and Safari are required browser targets. Mobile layout must remain usable, but mobile rendering performance is best-effort.
- Learning documentation will explain the mathematics and rationale of each graphics stage, including edge functions, barycentric interpolation, clipping, depth, spherical lookup, linear light, and terrain-normal derivation.
- Phase one will be single-threaded in native and Wasm builds.
- The local quality gate consists of formatting checks, Clippy with warnings denied, all workspace tests, and a release Trunk build.
- Performance exploration is a follow-up task after functional completion. Its baseline is the canonical 800×800, level-5 lunar scene in a release desktop-browser build.
- Satisfactory follow-up performance is sustained 30 FPS on a documented reference machine and browser; 60 FPS is a stretch goal.
- A GPU renderer will be considered only after profiling and reasonable CPU/Wasm algorithm, memory-access, SIMD, and threading investigations fail to meet the target. No generalized rendering-backend seam will be created in this spec.

## Testing Decisions

- Tests will assert externally observable behavior through the highest practical seam: deterministic scene inputs produce a framebuffer.
- The same shared-library frame-rendering interface used by native and web hosts will drive golden-image tests. This provides leverage without a parallel test-only interface.
- Small triangle and cube fixtures will use exact decoded-pixel comparisons.
- Canonical lunar fixtures will use fixed dimensions, scene times, map versions, subdivision, camera, Sun direction, and background.
- Realistic lunar golden images will compare decoded framebuffer pixels with a very small documented tolerance for platform floating-point differences.
- A failed realistic comparison will emit an amplified visual diff artifact and useful numerical difference statistics.
- Golden images can only be replaced through an explicit update command. Expected-image changes must be reviewed as behavior changes.
- Focused internal tests are justified for edge functions, barycentric weights, shared-edge ownership, clipping, depth comparison, longitude wrapping, latitude clamping, nearest-neighbor sampling, sRGB/linear conversion, image decoding, radial-direction mapping, and terrain-normal derivation.
- Focused tests should validate behavior and invariants, not private call structure or algorithm decomposition.
- Native host smoke tests will verify that milestone binaries can produce valid PNG output without duplicating renderer golden suites.
- One browser smoke test will verify that the Wasm host initializes and presents a framebuffer through Canvas 2D. Browser tests will not duplicate all visual assertions.
- The first browser compatibility target is current desktop Chrome, Firefox, and Safari.
- Animation tests will use explicit timestamps rather than real clocks.
- Asset provenance checks will verify expected checksums so accidental source-data replacement is visible.
- No prior rendering tests exist in the current repository; these seams establish the project's initial testing convention.

## Out of Scope

- Perspective projection and perspective-correct interpolation
- GPU rasterization through WebGL, WebGPU, Vulkan, Metal, Direct3D, or another graphics interface
- A generalized CPU/GPU rendering-backend abstraction
- Multithreaded rendering, browser workers, shared Wasm memory, and cross-origin-isolation deployment
- SIMD and other performance-specific implementation work
- Geometry displacement from lunar elevation
- Adaptive subdivision, level of detail, tessellation, and terrain meshes
- Bilinear, trilinear, anisotropic, or mipmapped map filtering
- Anti-aliasing and multisampling
- Cast shadows, shadow maps, ambient lighting, and specular lighting
- General-purpose shader languages or programmable material systems
- Scene graphs and general-purpose scene serialization
- General-purpose mesh import, including glTF conversion
- Interactive camera, lighting, pause, or quality controls
- Native interactive window creation
- HDR framebuffers and output formats
- Video, GIF, WebM, or MP4 encoding
- Scientific-analysis claims or scientifically validated visualization output
- Runtime downloading of NASA source data
- A literal single-file HTML artifact
- Selecting a public hosting provider before the color-map deployment milestone
- Mobile performance guarantees
- CPU/Wasm performance tuning and GPU migration themselves; these are follow-up efforts after this spec

## Further Notes

- Milestone order is: workspace and framebuffer; flat 2D triangle; barycentric vertex-color triangle; orthographic rotating cube with depth and culling; six-plane clipping; smooth level-5 octasphere; per-fragment lunar color-map lookup; public static deployment; linear-light gibbous lighting and ten-second globe rotation; terrain normals from lunar elevation; twenty-second lunar-phase animation; realistic canonical goldens; then separate performance exploration.
- Golden tests should be introduced alongside the milestone they protect rather than postponed until the final lunar render.
- The NASA CGI Moon Kit describes the selected color data as optimized for aesthetics rather than science. This matches Apollo 18's visually compelling but data-grounded goal.
- Elevation exaggeration is not part of the physical baseline in this spec. Any future artistic exaggeration must be explicit and documented.
- The existing coordinate ADR remains authoritative. Imported right-handed assets, if introduced later, must convert at their import seam.
- The project glossary distinguishes software renderer, lunar globe, lunar color map, lunar elevation map, octasphere, radial direction, terrain normal, and lunar phase. Implementation and documentation should use those canonical terms.
