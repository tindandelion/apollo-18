# 36: Add four-fragment rendering with a scalar fallback

**What to build:** Render existing scenes through a four-fragment batch seam that preserves current scalar behavior, allowing later Wasm SIMD kernels to replace bounded work without creating a lunar-specific rendering bypass.

**Blocked by:** 35: Reprofile the optimized scalar renderer

**Status:** ready-for-agent

- [ ] Fragment batches represent per-lane coordinates, coverage, depth, barycentric weights, shader inputs, and outputs without weakening their domain boundaries.
- [ ] Existing scalar fragment shaders work through a default fallback and do not require scene-specific duplication.
- [ ] Partially covered batches shade and write only accepted lanes.
- [ ] Shared-edge ownership, strict depth behavior, framebuffer encoding, and deterministic traversal remain unchanged.
- [ ] Exact triangle and cube goldens, realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [ ] The scalar batched implementation introduces no material regression in the retained browser diagnostic.
- [ ] Browser smoke coverage and the local quality gate pass.
