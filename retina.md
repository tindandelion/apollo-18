# Retina display support

Ticket: `.scratch/apollo-18/issues/13-support-retina-displays.md`

## Goal

Make the web lunar globe materially sharper on high-density displays by choosing the software renderer's framebuffer dimensions and the Canvas 2D backing dimensions from the canvas's displayed size and the browser's device pixel ratio, while bounding the result by the useful spatial detail of the current lunar color map. Rendering more pixels is expected to reduce frame rate; a separate follow-up ticket restores the sustained performance target without reducing the sharper resolution.

This work changes only the web presentation path. Native rendering and golden fixtures retain their canonical dimensions.

## Current behavior

The web host currently:

- renders an 800×800 framebuffer;
- sets the canvas backing dimensions to 800×800;
- scales the canvas responsively with CSS; and
- applies `image-rendering: pixelated`.

On a Retina display, one CSS pixel can correspond to multiple physical display pixels. Stretching the fixed 800×800 backing image across those pixels makes the globe softer than an image rendered closer to the display's physical resolution.

## Required work

### 1. Select a dynamic web resolution

Measure:

- the canvas's displayed CSS dimensions; and
- `window.devicePixelRatio`.

The ideal backing dimensions are approximately:

```text
backing width  = CSS width  × device pixel ratio
backing height = CSS height × device pixel ratio
```

Round the resulting dimensions to valid integer canvas dimensions. The web host must render the framebuffer at exactly the selected backing dimensions, and those dimensions must agree with the Canvas 2D backing dimensions.

### 2. Introduce a bounded resolution policy

The application must not honor arbitrary device pixel ratios without a limit. Software-rendering cost grows approximately with the number of pixels:

```text
cost ∝ width × height
```

Doubling both dimensions therefore creates roughly four times as much rendering work.

Define and document a resolution or total-pixel-budget policy that:

- produces materially more detail at device pixel ratio 2 than the current 800×800 backing resolution at the same displayed size;
- prevents high device pixel ratios from multiplying rendering work without a measured bound;
- preserves the globe's aspect ratio; and
- caps the framebuffer at 1152 pixels per side, producing an approximately 1037-pixel-diameter globe from the color map's approximately 1024 useful samples across a visible hemisphere.

This is an information-based cap rather than a performance-derived cap. Performance degradation from rendering more pixels is expected and will be addressed by a separate tuning ticket.

### 3. Respond to display changes

Recalculate the selected backing resolution when:

- the browser viewport is resized;
- the canvas's displayed CSS size changes; or
- the effective device pixel ratio changes, such as after browser zoom or moving the window between displays.

When the selected resolution changes:

- update the canvas backing dimensions;
- render the framebuffer at those same dimensions;
- preserve the globe's circular aspect;
- avoid mismatches between `ImageData`, framebuffer, and canvas dimensions; and
- continue the existing animation timeline without resetting scene time.

Canvas dimensions should only be reset when the selected resolution actually changes.

### 4. Stop forcing pixelated scaling

Remove the current CSS rule:

```css
image-rendering: pixelated;
```

The canvas should gain sharpness from an appropriately sized backing framebuffer rather than forced nearest-neighbor enlargement.

### 5. Preserve non-web behavior

Resolution changes must not alter:

- native output dimensions;
- golden fixture dimensions;
- scene-time calculation;
- lunar orientation or phase;
- lunar map lookup; or
- renderer behavior beyond receiving the dimensions selected by the web host.

The fixed canonical dimensions should remain wherever native binaries and golden tests require them. Only the web host should switch from fixed dimensions to display-derived dimensions.

### 6. Expand browser coverage

Update the Playwright browser tests to verify:

- a device-pixel-ratio-2 viewport selects a backing resolution sharper than the current 800×800 result at the same displayed size;
- canvas backing dimensions agree with the dimensions rendered by the software renderer;
- responsive resizing updates the selected backing dimensions;
- the globe remains circular;
- Canvas 2D successfully presents non-background framebuffer pixels;
- the host does not request WebGL, WebGL2, or WebGPU; and
- no page, JavaScript, Wasm, resource, or canvas errors occur.

The existing smoke-test assertion that the canvas always remains 800×800 must be replaced with assertions for the new resolution policy.

The release browser performance test must exercise the representative lit lunar scene under the documented high-density resolution policy and record its baseline without imposing a minimum frame-rate gate for this ticket. The follow-up performance-tuning ticket will restore the sustained 30 FPS requirement at the same capped resolution.

### 7. Document the result

Add a concise learning guide under `docs/learning/` explaining:

- CSS pixels;
- canvas backing pixels;
- physical display pixels;
- device pixel ratio;
- why rendering cost grows with pixel count;
- the chosen resolution or pixel-budget policy; and
- the measured sharpness/performance tradeoff.

Update `docs/testing.md` with the revised smoke-test and performance-test behavior, including the reference viewport, device pixel ratio, machine, browser, selected backing resolution, and observed performance.

The implementation spec currently contains fixed-800 web statements that this follow-up ticket intentionally supersedes. Update `.scratch/apollo-18/spec.md` so it no longer claims that the web canvas always has a fixed 800×800 backing resolution or deliberately ignores device pixel ratio.

## Likely implementation shape

The web host will need a small resolution-selection seam that accepts the displayed CSS dimensions and device pixel ratio, applies the bounded policy, and returns integer framebuffer dimensions. Keeping this calculation separate from browser observation makes the policy easier to test directly.

The animation should retain its original start timestamp independently from resolution selection. Each animation frame should use the currently selected dimensions while deriving scene time from the same uninterrupted browser timestamp sequence.

Browser resize and device-pixel-ratio observation should update resolution state without introducing rendering behavior into JavaScript or the HTML host. The Rust/Wasm web adapter remains responsible for selecting dimensions and passing them to the shared renderer.

## Likely files involved

- `crates/web/src/lib.rs`
- `crates/web/index.html`
- `crates/web/smoke-tests/web-smoke.spec.js`
- `crates/web/smoke-tests/web-performance.spec.js`
- `docs/testing.md`
- a new guide under `docs/learning/`
- `.scratch/apollo-18/spec.md`

Focused Rust tests may be added alongside the web host if the resolution-selection policy can be exercised outside browser APIs.

## Completion checks

Before the ticket can be marked done:

1. Create and work on a dedicated feature branch.
2. Implement every acceptance criterion in the ticket.
3. Run the browser smoke tests.
4. Run the release browser performance test and record the reference environment and result.
5. Run the canonical quality gate from the repository root:

   ```bash
   ./scripts/dev/quality-gate.sh
   ```

6. Review the implementation and documentation.
7. Commit the completed work only after proposing the exact commit message and receiving confirmation.
