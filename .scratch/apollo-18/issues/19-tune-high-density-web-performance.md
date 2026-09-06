# 19: Tune high-density web rendering performance

**What to build:** Optimize the representative web lunar globe at the color-map-bounded high-density resolution introduced by Ticket 13 so it sustains at least 30 FPS in desktop WebAssembly without reducing output resolution, lunar-map detail, or presentation quality.

**Blocked by:** 13: Use color-map-bounded high-density web rendering

**Status:** ready-for-agent

## Performance contract

- The workload is the representative lit lunar scene rendered at the 1152-pixel high-density cap selected by Ticket 13.
- Success is sustained throughput of at least 30 completed, rendered, and presented frames per second after warmup in the repeatable release browser performance test.
- The reference machine, operating system, browser and version, viewport, device pixel ratio, selected backing resolution, warmup duration, measurement duration, baseline, and final result are recorded together.
- The target must be reached through measured CPU/Wasm or presentation-path optimization. Lowering the selected backing resolution, reducing octasphere subdivision or lunar-map detail, skipping render or presentation work for counted frames, or introducing WebGL/WebGPU does not satisfy the ticket.

- [ ] The capped high-density baseline produced by Ticket 13 is reproduced and the dominant renderer, framebuffer-transfer, or Canvas 2D presentation costs are profiled before optimization.
- [ ] Profiling evidence and attempted optimizations are documented sufficiently to explain why the retained changes address measured bottlenecks.
- [ ] The release browser performance test verifies at least 30 completed, rendered, and presented frames per second at the 1152-pixel cap after warmup.
- [ ] Performance instrumentation cannot pass by counting animation callbacks that skip either software rendering or Canvas 2D presentation.
- [ ] The optimized output preserves the resolution-selection policy, lunar orientation and phase, lunar color-map and elevation-map sampling, terrain shading, octasphere subdivision, and Canvas 2D presentation path.
- [ ] Native output dimensions, deterministic native output, and golden fixtures remain unchanged.
- [ ] Browser smoke coverage continues to pass at device pixel ratio 2 and after responsive resizing.
- [ ] Performance documentation records the reference environment, baseline, final measurement, profile findings, and the quality-preserving tradeoffs made.
- [ ] The local quality gate, browser smoke tests, and ticket-specific release browser performance test pass.
