# 13: Use color-map-bounded high-density web rendering

**What to build:** Make the web lunar globe sharper on high-density displays by deriving its framebuffer and Canvas 2D backing resolution from the displayed canvas size and device pixel ratio, while capping that resolution at the useful spatial detail of the current lunar color map.

**Blocked by:** 15: Animate the lunar phases

**Status:** ready-for-agent

## Settled resolution policy

- The desired backing dimensions are the canvas's displayed CSS dimensions multiplied by the browser's device pixel ratio and rounded to integer pixels.
- The 2048×1024 equirectangular lunar color map provides approximately 1024 source samples across a visible hemisphere in each direction.
- Because the lunar globe occupies 90% of the shorter framebuffer dimension, a 1152-pixel maximum backing dimension is the practical source-matched cap: it produces an approximately 1037-pixel globe without spending substantially more work on framebuffer detail than the color map can supply.
- If either desired backing dimension exceeds 1152, both dimensions are reduced by the same scale factor so neither exceeds 1152 and the displayed aspect ratio is preserved.
- This is an information-based upper bound, not a promise to match unrestricted native device pixel ratio. Lower-density and smaller displays use their display-derived backing dimensions without being enlarged to the cap.
- Resolution selection is deterministic from CSS dimensions and device pixel ratio; it does not adapt dynamically from measured frame rate.
- Rendering more pixels is expected to reduce frame rate. This ticket establishes the sharper output and records its performance baseline; Ticket 19 separately tunes that output back to the sustained desktop target without reducing its resolution.

- [ ] The web host observes the canvas's displayed CSS dimensions and browser device pixel ratio instead of always rendering an 800×800 framebuffer.
- [ ] A focused, testable resolution-selection policy computes display-derived integer backing dimensions and applies the uniform 1152-pixel cap without upscaling beyond the display's requested density.
- [ ] At device pixel ratio 2 on the reference desktop viewport, the selected backing resolution is materially sharper than the current 800×800 backing resolution while remaining within the color-map-derived cap.
- [ ] Canvas backing dimensions and renderer dimensions agree, preserve the globe's circular aspect, and update correctly after a viewport resize or device-pixel-ratio change.
- [ ] Browser scaling no longer forces nearest-neighbor `pixelated` presentation when backing and displayed dimensions differ.
- [ ] Resolution changes do not reset scene time or alter lunar orientation, map lookup, native output dimensions, or golden render dimensions.
- [ ] The repeatable release browser performance test measures the representative lit lunar scene at the capped high-density resolution, records the reference machine, browser, selected resolution, and observed baseline, and imposes no minimum frame-rate gate in this ticket.
- [ ] Focused tests cover uncapped selection, cap application, aspect-ratio preservation, rounding, and invalid or zero-sized browser measurements.
- [ ] Browser tests cover a device-pixel-ratio-2 viewport, responsive resizing, backing-resolution selection, and successful Canvas 2D presentation.
- [ ] Learning documentation explains CSS pixels, backing pixels, device pixel ratio, the color-map-derived resolution bound, and the measured sharpness/performance tradeoff.
- [ ] The implementation spec and testing documentation are updated to replace the old fixed-800 web-resolution policy.
- [ ] The local quality gate passes.
