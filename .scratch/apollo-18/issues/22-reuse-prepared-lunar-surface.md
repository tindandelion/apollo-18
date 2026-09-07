# 22: Reuse the prepared lunar surface across animation frames

**What to build:** Make native lunar sequences and the web lunar-phase showcase reuse the shared renderer's prepared fixed lunar view between frames, including correct reconstruction when the browser selects a different canvas backing resolution, while retaining a clear path for future camera or globe motion.

**Blocked by:** 21: Introduce a prepared lunar-surface rendering seam

**Status:** ready-for-agent

## Why

The [high-density web performance analysis](../performance-analysis.md) identifies repeated fixed-view preparation as the largest structural bottleneck. This ticket realizes that optimization in both presentation adapters while preserving explicit invalidation for resolution or pose changes.

- [ ] Native lunar sequences prepare one fixed lunar view per requested output resolution and pose, then reuse it for every deterministic frame timestamp while those inputs remain unchanged.
- [ ] The web showcase prepares the fixed lunar view after loading the lunar maps and reuses it across animation callbacks while the canvas backing resolution, camera, and globe pose remain unchanged.
- [ ] A changed canvas backing resolution rebuilds prepared state exactly once before rendering at the new dimensions; temporarily zero CSS dimensions continue to skip rendering without resetting scene time.
- [ ] Any future camera or globe pose change has an explicit rebuild or general-rendering fallback path and cannot silently reuse stale screen-space data; this ticket does not implement libration or approximate cached-image warping.
- [ ] Native output dimensions, frame naming, sequence timing, and deterministic pixels remain unchanged.
- [ ] Canonical lunar goldens remain unchanged within their documented tolerance.
- [ ] Browser smoke coverage verifies reuse at device pixel ratio 2, correct framebuffer presentation, and reconstruction after responsive resizing and device-pixel-ratio changes.
- [ ] Repeatable measurements report initial preparation time, recurring render time, complete presented-frame time, and prepared-state memory use at 1152×1152.
- [ ] The existing release performance test still counts only callbacks that complete both software rendering and Canvas 2D presentation.
- [ ] The performance analysis is updated with the post-preparation profile, the measured reduction from the original baseline, and the fixed-view cache's implications for future libration or other pose animation.
- [ ] The canonical local quality gate, native smoke tests, and browser smoke tests pass.
