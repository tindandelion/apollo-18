# 10: Accelerate sRGB framebuffer encoding

## Current problem

The canonical 800×800 lunar globe does not meet the sustained 30 FPS desktop
WebAssembly target. Precomputing the lunar color map in linear RGB improves
performance from approximately 9.7 FPS to 15.2 FPS, but every accepted
fragment still encodes three linear color channels to sRGB with the nonlinear
transfer function before writing the framebuffer.

A controlled probe that bypassed only this output transfer reached
approximately 49.4 FPS. The per-fragment power operations therefore consume
about 45 ms per frame, or 69% of current frame time, and are the largest
remaining measured bottleneck.

## Proposed solution

Keep color interpolation, shading, and future lighting calculations in linear
RGB. Optimize only the final display boundary by replacing repeated evaluation
of the nonlinear linear-to-sRGB power function with a precomputed one-dimensional
lookup table.

The table samples the standard linear-to-sRGB transfer curve over the clamped
linear range from zero to one. To encode a channel, scale its clamped value to
the table domain, select the two neighboring samples, and linearly interpolate
between them before rounding to the nearest 8-bit sRGB value. Values below zero
or above one retain the existing clamping behavior.

Choose the smallest practical table resolution through measurement rather than
assuming a fixed size. The selected table must preserve the renderer's existing
color accuracy and golden output while removing power-function evaluation from
the per-fragment output path. Retain construction-time conversion of the lunar
color map to linear RGB so input decoding is also absent from the sampling hot
path.

**What to build:** Make the canonical lunar globe sustain the desktop
WebAssembly performance target by using accurate lookup-table interpolation at
the linear-RGB-to-sRGB framebuffer boundary without changing the renderer's
linear-light color model.

**Blocked by:** 09: Apply the NASA lunar color map

**Status:** done

- [x] Interpolation, shading, and future lighting continue to operate entirely
      in linear RGB.
- [x] Final framebuffer encoding uses lookup-table interpolation and performs
      no power-function evaluation for accepted fragments.
- [x] Output channels retain the standard sRGB transfer curve, clamp to the
      zero-to-one linear range, and round to the nearest 8-bit value.
- [x] Focused tests cover zero, one, the piecewise transfer boundary,
      representative mid-range values, out-of-range clamping, and interpolation
      error across the table domain.
- [x] The selected table resolution and its measured worst-case output error
      are documented.
- [x] Existing canonical golden renders remain unchanged within their current
      comparison tolerance.
- [x] The repeatable release browser performance test sustains at least 30 FPS
      for the canonical 800×800 lunar scene on the documented reference
      machine and browser.
- [x] The local quality gate passes.
