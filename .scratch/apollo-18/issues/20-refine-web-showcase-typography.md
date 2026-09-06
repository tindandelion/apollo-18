# 20: Refine the web showcase typography

**What to build:** Match the font of the related 3D Rasterizer in Rust project, tighten the showcase description, and present the credits as dimmed full-width footer text.

**Status:** done

- [x] The webpage loads Montserrat weights 400 and 700 from Google Fonts with swap rendering behavior.
- [x] Montserrat applies to all webpage text, with the existing system sans-serif stack retained as fallback.
- [x] The showcase description reads “Elevation-shaded lunar globe drawn by the software renderer in Rust”.
- [x] The dimmed footer spans the viewport, aligns provenance left and licensing right above 60rem, and stacks both centered when the viewport reaches the globe’s width-constrained range.
- [x] The release web build and browser smoke tests pass.
- [x] The canonical local quality gate passes.
