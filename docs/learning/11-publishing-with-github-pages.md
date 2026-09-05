# Publishing a custom static site with GitHub Pages

A static website is a directory of files that a web server can return without
running application code on the server. Apollo 18's browser host compiles to
HTML, JavaScript, and WebAssembly. The lunar color map is embedded in the
WebAssembly application, so the published site needs no runtime filesystem,
database, or NASA network request.

Trunk turns the source entry point into that deployable directory:

```text
crates/web/index.html + Rust crates + embedded assets
                         |
                         v
                  crates/web/dist/
                         |
                         v
              HTML + JavaScript + WebAssembly
```

GitHub Pages serves the resulting files over HTTPS. It does not compile the
Rust project itself; GitHub Actions builds the artifact and hands it to Pages.

## Separating build from deployment

Apollo 18's workflow uses two jobs with different responsibilities:

1. The build job checks out the repository, installs the pinned Rust and Trunk
   tools, runs the quality gate, creates the release site, and uploads the
   `dist` directory as a Pages artifact.
2. The deploy job waits for the build, then publishes that exact artifact to
   the protected `github-pages` environment.

This separation means a failed format check, lint, test, or release build
cannot replace the live site. It also gives the deploy job only the permissions
needed to publish:

```text
build:  contents: read
deploy: pages: write, id-token: write
```

The identity token lets GitHub verify that the deployment came from the
authorized workflow without storing a long-lived deployment credential in the
repository.

## Why the public URL matters

A site hosted at a repository URL normally lives below a path:

```text
https://example.github.io/apollo-18/
```

If generated HTML requests `/application.js`, the browser looks at the domain
root and misses the repository path. Trunk therefore needs the Pages base path
when it generates JavaScript and WebAssembly URLs:

```text
trunk build index.html --release \
  --public-url "${Pages base path}/"
```

The `actions/configure-pages` step discovers that path. For a repository site
it may be `/apollo-18`; for a custom domain it may be empty. Supplying the
discovered value keeps the same workflow valid in both environments.

## Keeping the pipeline repeatable and fast

The project pins Rust in `rust-toolchain.toml` and requests a specific Trunk
version. Installing Trunk with `cargo install` compiled the tool and all of its
dependencies on every fresh runner, consuming five minutes of a six-minute
build. Downloading Trunk's precompiled release with checksum verification
reduced that step to one second and the complete build job to about a minute
and a half.

Pinning application tools controls which compiler and bundler produce the
site. Pinning or deliberately versioning workflow actions separately controls
the automation that performs the build and deployment.

## Verifying more than a successful upload

A green deployment job proves that Pages accepted the artifact, not that the
application works in a browser. Publication is checked at several boundaries:

- The local quality gate proves the Rust workspace and release web build pass.
- The browser smoke test proves that JavaScript and WebAssembly initialize,
  resources load, Canvas 2D receives the framebuffer, and rendered pixels
  appear.
- The live URL proves DNS, HTTPS, the Pages base path, and the deployed
  artifact work together.
- Desktop-browser and small-viewport checks cover compatibility and responsive
  presentation that a build alone cannot establish.

The deployed page also carries its own provenance boundary: NASA lunar data and
its usage guidance are credited separately from Apollo 18's `MIT OR
Apache-2.0` source-code license.

## Publishing the native animation with a version

Website deployment and project releases have different lifecycles. Pushes to
`main` continue to update GitHub Pages, while a stable semantic-version tag such
as `0.1.0` starts the release workflow. Keeping these paths separate avoids
encoding a large animation for an ordinary website update and limits release
permissions to the tag-triggered workflow.

Before publishing, the release workflow checks that the tag has exact
`MAJOR.MINOR.PATCH` form, is newer than every published release, and passes the
same formatting, linting, test, and release web-build gate as deployment. It
then invokes the reusable deployment script to render the native lunar-globe
sequence and encode it with FFmpeg as a lossless animated WebP. The normal,
published GitHub Release uses the tag as its name, carries generated release
notes, and retains the animation as `lunar-globe.webp`.

The current rotating showcase lasts ten seconds. Its 300 frames sample scene
time at 30 frames per second from zero through `299 / 30` seconds. Omitting the
frame at exactly ten seconds avoids repeating the initial pose at the loop
boundary. The WebP itself loops indefinitely.

The repository README contains only an image reference through GitHub's stable
latest-release download URL. Each newer release therefore updates the visible
animation without committing a generated binary or rewriting the README. The
URL does not resolve until the first versioned release has been published.

## Further reading

- [Using custom workflows with GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
- [Configuring a publishing source](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)
- [Trunk 0.21.14 build configuration](https://docs.rs/crate/trunk/0.21.14/source/site/content/configuration.md)
