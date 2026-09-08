# 21: Introduce a prepared lunar-surface rendering seam

**What to build:** Let the shared software renderer prepare pose-independent lunar surface data once from the canonical lunar maps, then use it while rendering deterministic frames whose ephemeris-driven globe pose and Sun direction may both change.

**Blocked by:** 28: Drive lunar animations from the ephemeris start

**Status:** ready-for-agent

## Why

The [high-density web performance analysis](../performance-analysis.md) found that recurring lunar fragment shading dominates frame time, but its original fixed-view premise no longer holds: sub-Earth libration and lunar position angle change the globe pose throughout both realistic animations. Screen-space coverage, visible geography, map lookup, and world-space terrain normals must therefore be treated as recurring work. Preparation may retain only invariants that remain valid across arbitrary lunar appearances, such as canonical mesh data or map-derived object-space quantities.

- [ ] A small shared-renderer interface prepares pose-independent lunar surface data from the canonical lunar color and elevation maps, then renders an RGBA framebuffer for explicit dimensions and lunar appearance.
- [ ] The interface and domain naming make its invariants explicit; prepared data is not described as a fixed view and does not capture one framebuffer resolution or lunar globe pose.
- [ ] Profiling of the 1152×1152 ephemeris-span animation identifies which map-derived or mesh-derived recurring calculations can actually move into preparation before a representation is selected.
- [ ] Preparation performs only measured invariant work, such as reusable octasphere construction, linear lunar albedo preparation, or elevation-gradient preparation; it does not cache screen-space coverage, per-pixel geography, or world-space terrain normals across changing poses.
- [ ] Prepared state retains only data needed by recurring lunar rendering, with its memory representation and expected high-density memory cost documented.
- [ ] The existing one-shot lunar-globe rendering interface and general rasterization path remain available and preserve their error behavior and deterministic output.
- [ ] Prepared and one-shot rendering produce matching output across representative ephemeris records, position angles, phases, square and non-square dimensions, and repeated calls in different scene-time orders.
- [ ] Focused tests prove prepared data remains correct across pose changes and cannot silently depend on one prior framebuffer resolution or appearance.
- [ ] Invalid or empty framebuffer dimensions remain rejected without unbounded allocation or panics.
- [ ] Tests follow Arrange-Act-Assert and verify observable output rather than private cache structure or call counts.
- [ ] The performance analysis records the settled prepared-state design, profile evidence, and quality or memory tradeoffs discovered during implementation.
- [ ] The canonical local quality gate passes.
