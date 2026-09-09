# 39: Verify and document high-density web performance

**What to build:** Provide the final reproducible evidence that the quality-preserving software renderer satisfies Ticket 19's high-density desktop Wasm performance contract and explain which measured optimizations made it possible.

**Blocked by:** 38: Vectorize lunar fragment shading to meet the performance contract

**Status:** ready-for-agent

- [ ] Performance documentation records the reference machine, operating system, browser and version, viewport, device pixel ratio, canvas CSS dimensions, 1152×1152 backing resolution, warmup, measurement duration, baseline, and final results.
- [ ] Documentation distinguishes retained optimizations from reverted, inconclusive, regressive, and fidelity-changing experiments.
- [ ] Profile evidence explains how the retained changes address measured renderer bottlenecks and records the final residual costs.
- [ ] The release browser performance test sustains at least 30 completed, rendered, and presented FPS in three warmed runs.
- [ ] The instrumentation cannot pass by skipping software rendering, lunar shading, framebuffer handoff, or Canvas 2D presentation on counted frames.
- [ ] Browser smoke coverage passes at device pixel ratio 2 and after responsive resizing.
- [ ] Native output dimensions, deterministic native output, exact goldens, realistic lunar goldens, lunar-map sampling, terrain shading, octasphere subdivision, and the resolution-selection policy remain unchanged.
- [ ] The local quality gate passes.
