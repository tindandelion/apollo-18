# 21: Introduce a prepared lunar-surface rendering seam

**What to build:** Let the shared software renderer prepare a fixed view of the lunar surface for a selected framebuffer resolution and globe pose, then render deterministic lunar-phase frames from that prepared state without making the fixed-pose optimization the renderer's only path or changing existing native or web behavior yet.

**Blocked by:** 28: Start lunar animations from current UTC

**Status:** ready-for-agent

## Why

The [high-density web performance analysis](../performance-analysis.md) found that recurring lunar fragment shading consumes approximately 39–42 ms of the roughly 60.4 ms frame callback. It recommends preparing the fixed lunar view once because geographic lookup, lunar-map sampling, terrain-normal derivation, and coverage do not change while only the Sun direction animates.

- [ ] A small shared-renderer interface prepares a fixed lunar view from framebuffer dimensions, globe pose, and the canonical lunar maps, then renders an RGBA framebuffer from explicit scene time.
- [ ] The interface and domain naming make the fixed-view invariant explicit rather than presenting the prepared state as valid for arbitrary camera or globe motion.
- [ ] Preparation performs invariant coverage, lunar-map lookup, and terrain-normal work once so rendering multiple phase frames at the same pose does not repeat that work.
- [ ] Prepared state records enough of its resolution and pose dependencies to prevent stale reuse; a changed camera or globe pose must rebuild the view or use the general rendering path.
- [ ] Pose-independent preparation, such as canonical octasphere or lunar-map-derived data, remains separate from screen-space fixed-view data where doing so provides present value without implementing libration prematurely.
- [ ] Prepared state retains only the data needed for recurring lunar-phase rendering, with its memory representation and expected high-density memory cost documented.
- [ ] The existing one-shot lunar-globe rendering interface and general rasterization path remain available and preserve their error behavior and deterministic output; fixed-view assumptions do not leak into the general rasterizer or lunar-map modules.
- [ ] Prepared and one-shot rendering produce matching output across every canonical lunar phase, representative non-square dimensions, and repeated calls in different scene-time orders.
- [ ] Focused tests prove that prepared state cannot silently render after an incompatible resolution or pose change.
- [ ] Invalid or empty framebuffer dimensions remain rejected without unbounded allocation or panics.
- [ ] Tests follow Arrange-Act-Assert and verify observable output rather than private cache structure or call counts.
- [ ] The performance analysis records the settled prepared-state design and any quality or memory tradeoffs discovered during implementation.
- [ ] The canonical local quality gate passes.
