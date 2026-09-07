# 23: Optimize recurring lunar frames and meet 30 FPS

**What to build:** Optimize the remaining measured recurring lunar-frame work so the release web showcase sustains at least 30 completed, rendered, and presented frames per second at the 1152-pixel canvas backing-resolution cap without reducing visual quality.

**Blocked by:** 22: Reuse the prepared lunar surface across animation frames

**Status:** ready-for-agent

## Why

The [high-density web performance analysis](../performance-analysis.md) measures linear-to-sRGB encoding at approximately 12–15 ms per canonical frame and proposes it as the next target after fixed-view preparation. It also explains why presentation, allocation, SIMD, and threading should not be optimized first.

- [ ] The post-preparation 1152×1152 baseline is reproduced and profiled before further optimization, with recurring Lambertian shading, sRGB encoding, framebuffer writes, and Canvas 2D presentation measured separately where practical.
- [ ] Linear-to-sRGB output encoding uses a measured faster strategy while preserving linear-light rendering and remaining within one quantized output code of the exact transfer function for every tested input.
- [ ] An exhaustive focused test compares the retained sRGB encoding strategy with the exact transfer function across the displayable linear range, including clamping and transfer-function boundaries.
- [ ] If encoding does not provide sufficient headroom, only additional changes tied to a newly measured dominant cost are retained and documented.
- [ ] The release browser performance test verifies at least 30 completed, rendered, and presented frames per second after warmup at 1152×1152.
- [ ] Instrumentation cannot pass by skipping software rendering, Lambertian shading, framebuffer writes, or Canvas 2D presentation for counted frames.
- [ ] Resolution selection, octasphere subdivision, lunar-map detail and sampling, lunar orientation and phases, terrain shading, and the Canvas 2D presentation path remain unchanged.
- [ ] Native dimensions and deterministic output remain unchanged, and canonical lunar goldens stay within their documented tolerance.
- [ ] Browser smoke coverage continues to pass at device pixel ratio 2 and after responsive resizing.
- [ ] The performance analysis records the controlled reference environment, original baseline, post-preparation baseline, final result, final profile, retained optimizations, and rejected experiments.
- [ ] The canonical local quality gate, native and browser smoke tests, and ticket-specific release browser performance test pass.
