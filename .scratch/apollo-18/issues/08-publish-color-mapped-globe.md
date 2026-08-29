# 08: Publish the color-mapped lunar globe

**What to build:** Publish the color-mapped lunar globe as a one-page static showcase using the hosting provider appropriate to the repository's remote environment when this ticket begins.

**Blocked by:** 07: Apply the NASA lunar color map

**Status:** ready-for-agent

- [ ] The repository's remote environment is inspected at execution time and the hosting provider is selected and documented; GitHub Pages is preferred for a GitHub remote.
- [ ] A release build produces deployable static HTML, JavaScript, Wasm, and project-owned NASA assets without an application server.
- [ ] The published page loads the lunar globe without cross-origin or missing-asset failures.
- [ ] Current desktop Chrome, Firefox, and Safari display the 800×800 framebuffer correctly.
- [ ] The page remains usable on a small mobile viewport, with rendering performance treated as best-effort.
- [ ] Deployment steps are automated or documented sufficiently to reproduce the publication.
- [ ] The deployed page presents NASA asset provenance and the Apollo 18 code license appropriately.
- [ ] The local quality gate and release deployment build pass.
