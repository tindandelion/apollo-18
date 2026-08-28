# 01: Render the first triangle natively and on the web

**What to build:** Make Apollo 18 render its first complete image through the shared software renderer: an 800×800 flat-color triangle that can be saved by a native binary and displayed by the local static webpage from the same RGBA framebuffer.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The Rust edition 2024 workspace builds with the pinned stable 1.97.1 toolchain.
- [ ] The shared software renderer returns a tightly packed, top-to-bottom 8-bit RGBA framebuffer with opaque pixels and the fixed sRGB `#181818` background.
- [ ] A native triangle binary writes a valid 800×800 PNG containing the flat-color triangle.
- [ ] A Trunk-served Wasm webpage displays the same rendered framebuffer through Canvas 2D without WebGL or WebGPU.
- [ ] The canvas retains an 800×800 internal resolution while scaling responsively for display.
- [ ] An exact decoded-pixel golden test protects the triangle output.
- [ ] The code is licensed under MIT OR Apache-2.0.
- [ ] Formatting, Clippy with warnings denied, workspace tests, and the release web build all pass.
