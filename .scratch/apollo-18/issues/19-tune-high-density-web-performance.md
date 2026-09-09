# 19: Tune high-density web rendering performance

**What to build:** Optimize the representative web lunar globe at the color-map-bounded high-density resolution introduced by Ticket 13 so it sustains at least 30 FPS in desktop WebAssembly without reducing output resolution, lunar-map detail, or presentation quality.

**Blocked by:** 13: Use color-map-bounded high-density web rendering

**Status:** ready-for-agent

## Performance contract

- The workload is the representative lit lunar scene rendered at the 1152-pixel high-density cap selected by Ticket 13.
- Success is sustained throughput of at least 30 completed, rendered, and presented frames per second after warmup in the repeatable release browser performance test.
- The reference machine, operating system, browser and version, viewport, device pixel ratio, selected backing resolution, warmup duration, measurement duration, baseline, and final result are recorded together.
- The target must be reached through measured CPU/Wasm or presentation-path optimization. Lowering the selected backing resolution, reducing octasphere subdivision or lunar-map detail, skipping render or presentation work for counted frames, or introducing WebGL/WebGPU does not satisfy the ticket.

## Available investigation tools

The reference development machine currently has the following local tools. These are investigation aids, not project build dependencies:

- The project-local Playwright 1.62.1 installation and its bundled Chrome for Testing 151.0.7922.34 can run the release performance contract and collect Chrome DevTools Protocol CPU profiles, timeline traces, long tasks, garbage-collection events, and runtime metrics.
- Browser User Timing and `performance.now()` can measure the software render, `ImageData` construction, `putImageData`, and complete animation-frame boundaries without changing the counted work.
- Rust 1.97.1 and Cargo can run a focused native harness for the same 1152×1152 `render_lunar_globe` workload, separating shared-renderer cost from Wasm transfer and Canvas 2D presentation.
- `cargo-flamegraph` 0.6.14 can produce native sampling flame graphs; `cargo-instruments` 0.4.10 and macOS `sample` provide alternative native profiling paths.
- `wasm-tools` 1.258.0 can validate and inspect the release module, including function indexes, imports and exports, generated instructions, metadata, and SIMD usage after runtime profiling identifies where to look.
- Binaryen `wasm-opt` 132 can provide controlled original-versus-post-optimized Wasm experiments, static metrics, feature inspection, and exploratory SIMD/autovectorization probes. Every variant must be checked for both FPS and rendering correctness.
- The renderer tests, golden fixtures, browser smoke tests, ticket-specific performance test, and `./scripts/dev/quality-gate.sh` can verify that retained optimizations preserve required behavior.

Optional tools such as Samply or Hyperfine may be useful if the available profilers or measurement loops prove insufficient. The agent must explain the need and receive the user's explicit approval before installing any additional tool. Standalone `wasm2wat` is unnecessary while `wasm-tools print` supplies the required text-format inspection.

- [ ] The capped high-density baseline produced by Ticket 13 is reproduced and the dominant renderer, framebuffer-transfer, or Canvas 2D presentation costs are profiled before optimization.
- [ ] Profiling evidence and attempted optimizations are documented sufficiently to explain why the retained changes address measured bottlenecks.
- [ ] The release browser performance test verifies at least 30 completed, rendered, and presented frames per second at the 1152-pixel cap after warmup.
- [ ] Performance instrumentation cannot pass by counting animation callbacks that skip either software rendering or Canvas 2D presentation.
- [ ] The optimized output preserves the resolution-selection policy, lunar orientation and phase, lunar color-map and elevation-map sampling, terrain shading, octasphere subdivision, and Canvas 2D presentation path.
- [ ] Native output dimensions, deterministic native output, and golden fixtures remain unchanged.
- [ ] Browser smoke coverage continues to pass at device pixel ratio 2 and after responsive resizing.
- [ ] Performance documentation records the reference environment, baseline, final measurement, profile findings, and the quality-preserving tradeoffs made.
- [ ] The local quality gate, browser smoke tests, and ticket-specific release browser performance test pass.
