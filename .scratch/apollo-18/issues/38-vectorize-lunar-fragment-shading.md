# 38: Vectorize lunar fragment shading to meet the performance contract

**What to build:** Shade four accepted lunar fragments together in Wasm so the representative 1152-pixel lunar globe sustains at least 30 completed, rendered, and presented frames per second without reducing resolution, lunar-map detail, terrain shading, octasphere subdivision, or presentation quality.

**Blocked by:** 37: Vectorize fragment coverage and framebuffer output

**Status:** ready-for-agent

- [ ] Globe-location interpolation and normalization, terrain-normal arithmetic, illumination, and color processing are vectorized where the residual profile demonstrates value.
- [ ] Lunar color-map and elevation-map gathers may remain scalar per lane until measurement shows they prevent the target.
- [ ] Approximate vector geographic functions are introduced only if required by the measured residual and only when lunar-coordinate and sampled-texel behavior remain within existing correctness contracts.
- [ ] Longitude wrapping, latitude clamping, nearest-neighbor lunar-map sampling, polar terrain behavior, terrain-normal rim highlights, phase, orientation, and position angle remain preserved.
- [ ] Exact triangle and cube goldens, realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [ ] Current desktop Chrome, Firefox, and Safari remain supported through the intended release Wasm configuration.
- [ ] Three warmed release-browser runs each sustain at least 30 completed, rendered, and presented FPS at the 1152×1152 backing resolution.
- [ ] Browser smoke coverage and the local quality gate pass.
