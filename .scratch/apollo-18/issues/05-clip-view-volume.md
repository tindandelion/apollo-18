# 05: Clip the rotating cube against the view volume

**What to build:** Make partially visible geometry render robustly by clipping the cube's triangles against every view-volume plane before rasterization, with a native and web demonstration that crosses those planes safely.

**Blocked by:** 04: Add native and browser smoke tests

**Status:** ready-for-agent

- [ ] Triangles are clipped in homogeneous clip space against all six view-volume planes.
- [ ] Intersections preserve the attributes needed by subsequent rasterization.
- [ ] Resulting polygons are triangulated with consistent winding.
- [ ] Fully outside geometry disappears, fully inside geometry remains unchanged, and partially visible geometry has no explosive coordinates or invalid memory access.
- [ ] The cube demonstration visibly intersects clip planes in deterministic native frames and the evolving webpage.
- [ ] Focused tests cover each plane, corner intersections, complete rejection, complete retention, and generated triangles.
- [ ] Golden coverage protects representative clipped-cube frames.
- [ ] Learning documentation explains homogeneous clipping and polygon re-triangulation.
- [ ] The local quality gate passes.
