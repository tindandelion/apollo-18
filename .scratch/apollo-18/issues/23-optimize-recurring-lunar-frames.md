# 23: Optimize recurring lunar frames and meet 30 FPS

**What to build:** Optimize the remaining measured recurring lunar-frame work so the release web showcase sustains at least 30 completed, rendered, and presented frames per second at the 1152-pixel canvas backing-resolution cap without reducing visual quality.

**Blocked by:** 22: Reuse the prepared lunar surface across animation frames

**Status:** ready-for-agent

## Why

The [high-density web performance analysis](../performance-analysis.md) measures linear-to-sRGB encoding at approximately 12–15 ms per canonical frame, but its original fixed-view recommendation was invalidated by ephemeris-driven libration and roll. This ticket must optimize the dominant cost measured after pose-independent preparation rather than assuming screen-space lunar data can be cached.

- [ ] The post-preparation 1152×1152 ephemeris-span baseline is reproduced and profiled before further optimization, with pose transformation, rasterization, geographic lookup, recurring Lambertian shading, sRGB encoding, framebuffer writes, and Canvas 2D presentation separated where practical.
- [ ] The first retained optimization targets the newly measured dominant recurring cost; prior measurements may guide hypotheses but do not override the realistic profile.
- [ ] If linear-to-sRGB output encoding remains a dominant cost, it uses a measured faster strategy while preserving linear-light rendering and remaining within one quantized output code of the exact transfer function for every tested input.
- [ ] Any retained sRGB strategy has an exhaustive focused test against the exact transfer function across the displayable linear range, including clamping and transfer-function boundaries.
- [ ] Only additional changes tied to a measured dominant cost are retained and documented.
- [ ] The release browser performance test verifies at least 30 completed, rendered, and presented frames per second after warmup at 1152×1152.
- [ ] Instrumentation cannot pass by skipping software rendering, Lambertian shading, framebuffer writes, or Canvas 2D presentation for counted frames.
- [ ] Resolution selection, octasphere subdivision, lunar-map detail and sampling, ephemeris-driven libration and roll, lunar phases, terrain shading, and the Canvas 2D presentation path remain unchanged; no fixed-view shortcut freezes or approximates changing pose.
- [ ] Native dimensions and deterministic output remain unchanged, and canonical lunar goldens stay within their documented tolerance.
- [ ] Browser smoke coverage continues to pass at device pixel ratio 2 and after responsive resizing.
- [ ] The performance analysis records the controlled reference environment, original baseline, post-preparation baseline, final result, final profile, retained optimizations, and rejected experiments.
- [ ] The canonical local quality gate, native and browser smoke tests, and ticket-specific release browser performance test pass.
