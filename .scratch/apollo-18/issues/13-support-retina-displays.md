# 13: Use color-map-bounded high-density web rendering

**What to build:** Make the web lunar globe sharper on high-density displays by deriving its framebuffer and Canvas 2D backing resolution from the displayed canvas size and device pixel ratio, while capping that resolution at the useful spatial detail of the current lunar color map.

**Blocked by:** 15: Animate the lunar phases

**Status:** done

## Settled resolution policy

- The desired backing dimensions are the canvas's displayed CSS dimensions multiplied by the browser's device pixel ratio and rounded to integer pixels.
- The 2048×1024 equirectangular lunar color map provides approximately 1024 source samples across a visible hemisphere in each direction.
- Because the lunar globe occupies 90% of the shorter framebuffer dimension, a 1152-pixel maximum backing dimension is the practical source-matched cap: it produces an approximately 1037-pixel globe without spending substantially more work on framebuffer detail than the color map can supply.
- Resolution selection first computes floating-point desired dimensions from CSS size and device pixel ratio, then applies one uniform scale factor if either dimension exceeds 1152, and finally rounds both scaled dimensions to the nearest integers. Each result is clamped to at least 1 pixel, and neither selected dimension may exceed 1152.
- This is an information-based upper bound, not a promise to match unrestricted native device pixel ratio. Lower-density and smaller displays use their display-derived backing dimensions without being enlarged to the cap.
- Resolution selection is deterministic from CSS dimensions and device pixel ratio; it does not adapt dynamically from measured frame rate.
- A zero CSS width or height means the canvas is temporarily not renderable, so the web host skips rasterization and presentation for that frame. The first animation callback still establishes the animation start time, and skipped frames never pause or reset scene time. Negative or non-finite CSS dimensions, and non-positive or non-finite device pixel ratios, are errors.
- Each animation callback reads the canvas's current bounding rectangle and `window.devicePixelRatio`, then updates the backing resolution only when the selected integer dimensions change. A `ResizeObserver` is deferred unless later profiling or behavior demonstrates that it is necessary.
- The repeatable capped high-density browser scenario uses a 1440×900 CSS-pixel viewport at device pixel ratio 2. Tests measure the canvas's actual CSS size rather than assuming it fills the viewport, and verify that the selected backing resolution and renderer framebuffer are both 1152×1152.
- Rendering more pixels is expected to reduce frame rate. This ticket establishes the sharper output and records its performance baseline; Ticket 19 separately tunes that output back to the sustained desktop target without reducing its resolution.

- [x] The web host observes the canvas's displayed CSS dimensions and browser device pixel ratio instead of always rendering an 800×800 framebuffer.
- [x] A focused, testable resolution-selection policy computes display-derived integer backing dimensions and applies the uniform 1152-pixel cap without upscaling beyond the display's requested density.
- [x] At device pixel ratio 2 on the reference desktop viewport, the selected backing resolution is materially sharper than the current 800×800 backing resolution while remaining within the color-map-derived cap.
- [x] Canvas backing dimensions and renderer dimensions agree, preserve the globe's circular aspect, and update correctly after a viewport resize or device-pixel-ratio change.
- [x] Browser scaling no longer forces nearest-neighbor `pixelated` presentation when backing and displayed dimensions differ.
- [x] Resolution changes do not reset scene time or alter lunar orientation, map lookup, native output dimensions, or golden render dimensions.
- [x] The repeatable release browser performance test measures the representative lit lunar scene at the capped high-density resolution, records the reference machine, browser, selected resolution, and observed baseline, and imposes no minimum frame-rate gate in this ticket.
- [x] Focused tests cover uncapped selection, cap application, aspect-ratio preservation, rounding, and invalid or zero-sized browser measurements.
- [x] Browser tests cover a device-pixel-ratio-2 viewport, responsive resizing, backing-resolution selection, and successful Canvas 2D presentation.
- [x] Learning documentation explains CSS pixels, backing pixels, device pixel ratio, the color-map-derived resolution bound, and the measured sharpness/performance tradeoff.
- [x] The implementation spec and testing documentation are updated to replace the old fixed-800 web-resolution policy.
- [x] The local quality gate passes.

## Comments

- Implementation review against repository standards and this ticket found no outstanding findings. Browser smoke coverage, the capped release performance baseline, and the canonical quality gate all passed.
