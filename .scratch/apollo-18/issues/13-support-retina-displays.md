# 13: Support sharp rendering on Retina displays

**What to build:** Make the web lunar globe materially sharper on high-density displays by choosing its framebuffer and Canvas 2D backing resolution from the displayed canvas size and device pixel ratio, without giving up the sustained 30 FPS desktop WebAssembly target.

**Blocked by:** 15: Animate the lunar phases

**Status:** ready-for-agent

**Scheduling note:** This follows the complete terrain-shaded lunar-phase showcase so resolution and performance decisions are measured against the final representative lunar workload rather than an intermediate globe milestone.

- [ ] The web host observes the canvas's displayed CSS dimensions and browser device pixel ratio instead of always rendering an 800×800 framebuffer.
- [ ] A documented resolution or pixel-budget policy prevents high device pixel ratios from multiplying rendering work without a measured bound.
- [ ] At device pixel ratio 2 on the reference desktop viewport, the globe is materially sharper than the current 800×800 backing resolution at the same displayed size.
- [ ] Canvas backing dimensions and renderer dimensions agree, preserve the globe's circular aspect, and update correctly after a viewport resize or device-pixel-ratio change.
- [ ] Browser scaling no longer forces nearest-neighbor `pixelated` presentation when the backing resolution differs from the displayed size.
- [ ] Resolution changes do not alter scene time, lunar orientation, map lookup, or native and golden render dimensions.
- [ ] The representative lit lunar scene sustains at least 30 FPS in the repeatable release browser performance test on the documented reference machine and browser.
- [ ] Browser tests cover a device-pixel-ratio-2 viewport, responsive resizing, backing-resolution selection, and successful Canvas 2D presentation.
- [ ] Learning documentation explains CSS pixels, backing pixels, device pixel ratio, and the measured sharpness/performance tradeoff.
- [ ] The local quality gate passes.
