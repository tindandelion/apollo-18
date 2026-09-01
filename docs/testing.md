# Testing Apollo 18

The workspace quality gate covers Rust formatting, linting, tests, and the release web build:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd crates/web && trunk build index.html --release
```

## Native smoke tests

The native smoke tests execute the retained milestone binaries and validate the structure of their PNG artifacts. Temporary directories keep generated images out of the working tree.

From the repository root, run:

```bash
cargo test -p apollo18-native --test native_smoke
```

## Browser smoke test

The browser smoke test uses Playwright and headless Chromium to build, serve, and load the release web host. It verifies that the Wasm application initializes without runtime or resource errors, requests Canvas 2D rather than a GPU context, retains the canonical 800×800 canvas resolution, and presents non-background framebuffer pixels.

Install its Node dependencies and Chromium once:

```bash
cd crates/web/smoke-tests
npm ci
npx playwright install chromium
```

Then run the smoke test from the repository root:

```bash
scripts/web-smoke-test.sh
```

The script forwards additional Playwright arguments, such as `--headed`, after `npm test`. Playwright starts and stops a release-mode Trunk server automatically. Trunk output, Playwright results, and installed Node packages are written only to ignored directories.
