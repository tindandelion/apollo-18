# 22: Reuse the prepared lunar surface across animation frames

**What to build:** Make native lunar sequences and the web ephemeris-span showcase reuse the shared renderer's prepared pose-independent lunar data between frames while rendering every frame's changing ephemeris-driven globe pose correctly.

**Blocked by:** 21: Introduce a prepared lunar-surface rendering seam

**Status:** ready-for-agent

## Why

The [high-density web performance analysis](../performance-analysis.md) identifies repeated per-fragment lunar work as the largest measured bottleneck. Reuse must now respect the realistic animation: sub-Earth libration and position angle change visible geography and screen-space coverage, so a fixed screen-space lunar view is not reusable across frames.

- [ ] Native lunar sequences prepare pose-independent lunar data once and reuse it for every deterministic frame timestamp.
- [ ] The web showcase prepares pose-independent lunar data after loading the lunar maps and reuses it across animation callbacks.
- [ ] Every frame still applies its sampled lunar globe pose and Sun direction; no cached image, screen-space coverage, geographic lookup, or world-space normal is reused when that would freeze or approximate libration and roll.
- [ ] A changed canvas backing resolution renders at the new dimensions without rebuilding resolution-independent lunar data; temporarily zero CSS dimensions continue to skip rendering without resetting monotonic scene time.
- [ ] Any resolution-dependent framebuffer or rasterization storage is explicitly separate from pose-independent lunar preparation and is reconstructed when dimensions change.
- [ ] Native output dimensions, frame naming, sequence timing, and deterministic pixels remain unchanged.
- [ ] Canonical lunar goldens remain unchanged within their documented tolerance.
- [ ] Browser smoke coverage verifies reuse at device pixel ratio 2, changing ephemeris-driven pose, correct framebuffer presentation, and responsive resolution changes.
- [ ] Repeatable measurements report initial preparation time, recurring rendering time, complete presented-frame time, and prepared-state memory use at 1152×1152.
- [ ] The release performance test still counts only callbacks that complete both software rendering and Canvas 2D presentation.
- [ ] The performance analysis is updated with the post-preparation profile, the measured reduction from the original baseline, and any remaining pose-dependent bottlenecks.
- [ ] The canonical local quality gate, native smoke tests, and browser smoke tests pass.
