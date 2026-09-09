# 30: Remove WebAssembly rounding calls from sRGB encoding

**What to build:** Preserve the existing linear-to-sRGB framebuffer output while quantizing non-negative encoded channels without the costly per-channel WebAssembly rounding helper.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** ready-for-agent

- [ ] The replacement quantizer is equivalent to round-half-away-from-zero for every finite encoded value in the displayable range.
- [ ] Focused tests cover exact integers, values immediately around half-code thresholds, clamping boundaries, and the complete range used by the sRGB lookup table.
- [ ] Exact triangle and cube output, realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [ ] The retained diagnostic demonstrates a repeatable complete-frame improvement larger than measurement noise; otherwise the experiment is reverted and its result documented.
- [ ] The release browser performance contract is rerun and its cumulative result recorded.
- [ ] Browser smoke coverage and the local quality gate pass.
