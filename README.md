# Apollo 18

Apollo 18 is a 3D software renderer in Rust for exploring 3D graphics algorithms by presenting the Moon using real lunar surface data from NASA. It can be considered as a follow-up to [rust-3d-rasterizer](https://github.com/tindandelion/rust-3d-rasterizer).

I continue the exploration of 3D graphics and rasterization through a more focused rendering exercises. A secondary goal is to experiment with running a Rust application on the Web using WebAssembly.

## Our current progress

#### From a triangle to a terrain-shaded Moon

Apollo 18 began with a single flat triangle and a question: how much of the graphics pipeline could we build ourselves before drawing the Moon? We added barycentric color interpolation, depth buffering and culling, transformed a rotating cube, and then shaped an octasphere into our first globe.

From there, NASA's [CGI Moon Kit](https://svs.gsfc.nasa.gov/4720/) supplied the lunar color and elevation maps that gave the globe its familiar geography and terrain detail. Lambertian lighting revealed craters and ridges, and NASA's [2026 Moon Phase and Libration data](https://svs.gsfc.nasa.gov/5587/) brought the Moon's changing phase, libration, and apparent roll into the animation.

The result of that journey is the [published lunar globe showcase](https://www.tindandelion.com/apollo-18/) and the [rendered WebP animation](https://github.com/tindandelion/apollo-18/releases/latest/download/lunar-globe.webp) shown below.

[![Apollo 18 lunar globe animation](https://github.com/tindandelion/apollo-18/releases/latest/download/lunar-globe.webp)](https://www.tindandelion.com/apollo-18/)
