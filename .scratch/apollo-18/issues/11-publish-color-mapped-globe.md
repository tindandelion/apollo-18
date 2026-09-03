# 11: Publish the color-mapped lunar globe

**What to build:** Publish the color-mapped lunar globe as a one-page static showcase using the hosting provider appropriate to the repository's remote environment when this ticket begins.

**Blocked by:** 09: Apply the NASA lunar color map

**Status:** done

- [x] The repository's remote environment is inspected at execution time and the hosting provider is selected and documented; GitHub Pages is preferred for a GitHub remote.
- [x] A release build produces deployable static HTML, JavaScript, Wasm, and project-owned NASA assets without an application server.
- [x] The published page loads the lunar globe without cross-origin or missing-asset failures.
- [x] Current desktop Chrome, Firefox, and Safari display the 800×800 framebuffer correctly.
- [x] The page remains usable on a small mobile viewport, with rendering performance treated as best-effort.
- [x] Deployment steps are automated or documented sufficiently to reproduce the publication.
- [x] The deployed page presents NASA asset provenance and the Apollo 18 code license appropriately.
- [x] The local quality gate and release deployment build pass.

## Comments

- 2026-09-03: GitHub Pages was selected for the GitHub-hosted repository and deployed at <https://www.tindandelion.com/apollo-18/>. The user manually verified the live globe in current desktop Chrome, Firefox, and Safari, checked a small mobile viewport, and confirmed there were no runtime, cross-origin, or missing-asset failures. The local quality gate and browser smoke test passed, the standards and specification reviews reported no findings, and the release deployment completed successfully in [Deploy Apollo 18 Website](https://github.com/tindandelion/apollo-18/actions/runs/33741703739).
