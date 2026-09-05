# 16: Publish the lunar globe animation with versioned releases

**What to build:** Let maintainers publish a permanent, README-visible lunar globe animation by pushing a version tag. A healthy tagged revision renders the canonical native showcase, encodes it as a lossless animated WebP, and publishes it as a GitHub Release asset without changing the existing GitHub Pages deployment flow.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] A reusable deployment script renders 300 deterministic native lunar-globe frames at 800×800 and 30 frames per second.
- [ ] The deployment script encodes the frames as a lossless WebP using FFmpeg's maximum compression effort, with an infinite loop and no duplicated endpoint frame.
- [ ] The deployment script lives under the repository's deployment-script area, removes stale frame output before rendering, fails clearly when FFmpeg is unavailable, and writes the release-ready animation under ignored build output.
- [ ] Pushing a tag in strict stable `MAJOR.MINOR.PATCH` form, without a `v` prefix or prerelease suffix, triggers release automation; unrelated tags do not publish releases.
- [ ] Release automation rejects a version that is not semantically greater than every existing published release.
- [ ] Release automation runs the complete repository quality gate before rendering the animation.
- [ ] A successful run publishes a normal, non-draft, non-prerelease GitHub Release named after the tag, uses GitHub-generated release notes, and attaches the animation as `lunar-globe.webp`.
- [ ] A failed quality gate, render, encode, or upload does not report a successful release; rerunning a tag that already has a published release fails rather than replacing its asset.
- [ ] Release automation uses the runner-provided FFmpeg, has no manual-dispatch trigger, and adds no release concurrency policy.
- [ ] The existing `main`-based GitHub Pages deployment behavior remains unchanged and does not render the release animation.
- [ ] `README.md` contains only an embedded lunar globe image whose stable URL resolves to `lunar-globe.webp` from GitHub's latest published release.
- [ ] Deployment documentation explains the version-tag release path, permanent animation asset, README indirection, and the current ten-second frame-count convention.
- [ ] The local quality gate passes.
