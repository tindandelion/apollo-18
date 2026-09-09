# 29: Retain high-density browser performance diagnostics

**What to build:** Provide a repeatable release-browser diagnostic for Ticket 19 that measures the complete 1152-pixel high-density rendering path after warmup and separates software rendering, `ImageData` construction, and Canvas 2D presentation without changing the counted work.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] The diagnostic verifies the 1152×1152 canvas backing resolution on the reference viewport and device pixel ratio.
- [x] Measurements begin after a documented warmup and report medians over enough completed frames to distinguish meaningful changes from noise.
- [x] A frame is counted only after software rendering, framebuffer handoff, and Canvas 2D presentation complete.
- [x] Output records the machine, operating system, browser and version, viewport, device pixel ratio, canvas CSS dimensions, backing resolution, warmup, sample size, and stage timings.
- [x] The current baseline is reproduced and recorded alongside the existing sustained-FPS contract result.
- [x] Browser smoke coverage and the local quality gate pass.
