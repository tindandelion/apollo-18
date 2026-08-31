# 05: Add native and browser smoke tests

**What to build:** Add the end-to-end smoke coverage identified by the ticket-01 code review after the rotating cube is available: verify that retained native milestone binaries produce valid PNG artifacts and that the release Wasm host initializes and presents the shared framebuffer through Canvas 2D in a real browser.

**Blocked by:** 04: Animate the orthographic cube natively and on the web

**Status:** ready-for-agent

- [ ] Native smoke tests execute the retained triangle and cube binaries with temporary output paths and verify that each produces a decodable 800×800 8-bit RGBA PNG.
- [ ] Native smoke tests verify artifact structure and successful execution without duplicating the renderer's exact-pixel golden assertions.
- [ ] A browser smoke test builds and serves the release web host, loads it in a headless current desktop browser, and fails on page, JavaScript, or Wasm initialization errors.
- [ ] The browser smoke test verifies that the Canvas 2D canvas retains an 800×800 internal resolution and presents non-background pixels from the shared framebuffer without using WebGL or WebGPU.
- [ ] The smoke-test setup and commands are documented for local use and leave no generated artifacts in the working tree.
- [ ] Formatting, Clippy with warnings denied, workspace tests, the smoke tests, and the release web build all pass.
