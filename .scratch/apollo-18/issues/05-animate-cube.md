# 05: Animate the orthographic cube natively and on the web

**What to build:** Turn the deterministic cube still into a time-driven tracer bullet: derive cube rotation from explicit scene time, generate a reproducible native PNG sequence, and animate the same shared-renderer scene in the Wasm webpage.

**Blocked by:** 04: Render a deterministic orthographic cube

**Status:** done

## Settled native design

- `SceneTime` is an opaque public value type representing non-negative finite elapsed seconds. Its checked constructor owns the invariant and reports one general invalid-time error.
- The shared renderer accepts `SceneTime` explicitly for every cube frame. It does not retain a separate static cube-rendering function.
- The cube keeps a fixed `-20°` pitch. Its yaw starts at `+30°` at zero seconds and completes one revolution every 10 seconds. Scene time is reduced into that period before rotation is calculated.
- The renderer exposes the cube rotation period because it is scene behavior.
- The native cube binary is sequence-only. It generates 300 canonical 800×800 frames at a source-defined 30 FPS, deriving every timestamp independently as `frame_index / 30`.
- The frame at exactly 10 seconds is omitted because it duplicates the initial loop pose.
- The optional output-directory argument defaults to `target/apollo18/cube/frames`. Frames are named `frame-0000.png` through `frame-0299.png`.
- Existing expected frame paths are overwritten. Unrelated files are not removed, and completed frames remain if generation fails.
- The binary reports progress after every 30 frames. Its final summary reports summed software-renderer time and average rendering FPS; PNG encoding and filesystem operations are excluded from those measurements.
- Sequence iteration and naming remain in the cube binary. The native library retains responsibility for PNG encoding.
- Exact golden coverage remains limited to the zero-second pose, with the fixture named `cube_at_zero_seconds.png`.

## Acceptance criteria

- [x] Cube rotation is derived from explicit scene time rather than accumulated frame steps.
- [x] `SceneTime` rejects negative and non-finite seconds at construction, making invalid time unrepresentable at the rendering seam.
- [x] Supplying the same scene time and inputs always produces the same cube framebuffer; zero and 10 seconds produce the same pose, while distinct canonical times produce the expected distinct poses and visible faces.
- [x] The native cube binary generates the fixed 300-frame, 30 FPS, 10-second PNG sequence through the shared per-time renderer without exposing a separate single-frame CLI mode.
- [x] Native sequence timestamps, numbering, output behavior, progress reporting, and renderer-only performance summary follow the settled design.
- [x] The webpage evolves from the triangle to the cube and animates it from the browser's monotonic clock through `requestAnimationFrame`.
- [x] Native and web adapters render the cube through the same shared frame-rendering seam.
- [x] The exact zero-second cube golden and focused deterministic time-based tests protect the animated scene; native unit tests protect sequence timestamps and filenames without generating the full canonical sequence.
- [x] `docs/learning/05-time-derived-animation.md` explains time-derived rotation, loop wrapping, independent frame sampling, and why animation state is not accumulated frame by frame.
- [x] The local quality gate passes.
