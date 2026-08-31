# 04: Animate the orthographic cube natively and on the web

**What to build:** Turn the deterministic cube still into a time-driven tracer bullet: derive cube rotation from explicit scene time, render reproducible poses through the native host, and animate the same shared-renderer scene in the Wasm webpage.

**Blocked by:** 03: Render a deterministic orthographic cube

**Status:** ready-for-agent

- [ ] Cube rotation is derived from explicit elapsed scene time rather than accumulated frame steps.
- [ ] Supplying the same scene time and inputs always produces the same cube framebuffer, while distinct canonical times produce the expected distinct poses.
- [ ] The native cube binary can render a deterministic frame at a specified scene time without duplicating rendering behavior in the host.
- [ ] The webpage evolves from the triangle to the cube and animates it from the browser's monotonic clock through `requestAnimationFrame`.
- [ ] Native and web adapters render the cube through the same shared frame-rendering seam.
- [ ] Deterministic time-based tests and any required exact cube goldens protect the animated scene's canonical poses.
- [ ] Learning documentation explains time-derived rotation and why animation state is not accumulated frame by frame.
- [ ] The local quality gate passes.
