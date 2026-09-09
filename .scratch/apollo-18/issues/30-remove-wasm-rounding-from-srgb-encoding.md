# 30: Remove WebAssembly rounding calls from sRGB encoding

**What to build:** Preserve the existing linear-to-sRGB framebuffer output while quantizing non-negative encoded channels without the costly per-channel WebAssembly rounding helper.

**Blocked by:** 29: Retain high-density browser performance diagnostics

**Status:** done

- [x] The replacement quantizer is equivalent to round-half-away-from-zero for every finite encoded value in the displayable range.
- [x] Focused tests cover exact integers, values immediately around half-code thresholds, clamping boundaries, and the complete range used by the sRGB lookup table.
- [x] Exact triangle and cube output, realistic lunar goldens, native output dimensions, and deterministic native output remain unchanged.
- [x] The retained diagnostic demonstrates a repeatable complete-frame improvement larger than measurement noise; otherwise the experiment is reverted and its result documented.
- [x] The release browser performance contract is rerun and its cumulative result recorded.
- [x] Browser smoke coverage and the local quality gate pass.

## Comments

Implementation review found no standards or specification issues. Three changed
browser diagnostic runs reduced median complete-frame time from the baseline
64.3 ms to 55.4 ms. The cumulative performance contract improved to 17.95 FPS
but remains below Ticket 19's eventual 30 FPS target. Browser smoke coverage,
native smoke coverage, exact and realistic goldens, and the canonical quality
gate passed.
