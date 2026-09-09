# Exploring architecture with `cargo-arc`

[`cargo-arc`](https://github.com/seflue/cargo-arc) generates an interactive SVG that shows Cargo workspace crates, their nested Rust modules, and the `use` dependencies between them. It is useful as an architectural map: it helps us discover dependency direction, highly connected modules, and cycles. Its rule checker can enforce decisions after maintainers decide which dependencies are desirable.

## Install the tool

`cargo-arc` requires a stable Rust toolchain:

```bash
cargo install cargo-arc --version 0.3.1 --locked
```

Verify the installation and inspect the options supported by the installed version:

```bash
cargo arc --version
cargo arc --help
```

If the active toolchain is not stable, install with stable explicitly:

```bash
rustup update stable
cargo +stable install cargo-arc --version 0.3.1 --locked
```

## Generate a first diagram

Run the project script from the Apollo 18 repository root:

```bash
./scripts/dev/render-architecture.sh
```

It expands the graph through nested rasterizer modules, writes `target/apollo18/architecture/dependencies.svg`, and opens the result in the platform's default viewer. Additional arguments are forwarded to `cargo arc`; for example:

```bash
./scripts/dev/render-architecture.sh --externals
```

Set `ARCHITECTURE_OUTPUT` to override the output path. The script opens the result with macOS `open`.

Files under `target/` are disposable exploration artifacts and do not need to be committed.

## Read the diagram

The workspace appears as nested boxes:

- A top-level box represents a crate.
- Boxes inside a crate represent its modules.
- An arc represents a dependency inferred primarily from Rust `use` relationships.
- Collapsing a node combines its children's dependencies into summary arcs.
- Selecting a node or arc highlights related dependencies.
- Circular dependencies are detected and highlighted.

At the crate level, Apollo 18 should have approximately this shape:

```text
apollo18-native ───────▶ apollo18-renderer
apollo18-web ──────────▶ apollo18-renderer
```

This direction keeps platform-independent rendering in `apollo18-renderer`. The native and web crates adapt that renderer to their respective environments. The renderer should not depend back on either adapter.

Inside `apollo18-renderer`, the diagram includes modules such as:

```text
lunar_appearance
lunar_ephemeris
lunar_phase_animation
octasphere
rasterizer
├── color
├── framebuffer
├── shader
├── triangle
└── vertex
scene_time
```

## Explore progressively

### Start with crates only

```bash
cargo arc \
  --expand-level 0 \
  -o target/apollo18/architecture/crates.svg

open target/apollo18/architecture/crates.svg
```

This view is useful for asking:

- Which crates are architectural foundations?
- Which crates are entry points or adapters?
- Is dependency direction one-way?
- Are any crates unexpectedly coupled?

### Expand direct modules

```bash
cargo arc \
  --expand-level 1 \
  -o target/apollo18/architecture/modules.svg
```

In the browser:

1. Select a crate or module to highlight its relationships.
2. Expand a collapsed node to inspect its children.
3. Select an arc to identify its source and destination.
4. Look for highlighted cycles.
5. Collapse unrelated areas when the diagram becomes noisy.

### Include external crates

```bash
cargo arc \
  --expand-level 1 \
  --externals \
  -o target/apollo18/architecture/externals.svg
```

This reveals where Apollo 18 modules depend on libraries such as `glam`, `png`, `wasm-bindgen`, and `web-sys`.

Transitive external dependencies can also be included:

```bash
cargo arc \
  --expand-level 0 \
  --externals \
  --transitive-deps \
  -o target/apollo18/architecture/all-externals.svg
```

The transitive view is much noisier. Use it to investigate dependency details rather than as the primary architecture overview.

## Interpret dependency shapes

### Fan-out: what does this module depend on?

A module with many outgoing dependencies has high fan-out. It may be coordinating several concepts, doing too many jobs, or legitimately serving as a composition boundary. High fan-out is a reason to investigate, not automatically a defect.

### Fan-in: what depends on this module?

A module with many incoming dependencies has high fan-in. It is likely foundational and therefore expensive to change carelessly. Modules such as `rasterizer::color`, `rasterizer::framebuffer`, or `scene_time` may have this role in the renderer.

### Direction: where does policy live?

A useful initial hypothesis for Apollo 18 is:

```text
native and web adapters
          ↓
public rendering API
          ↓
scene and lunar concepts
          ↓
geometry, rasterization, and image primitives
```

The real graph may be more nuanced. A dependency that repeatedly points against the intended direction is a prompt to inspect the design and vocabulary around that boundary.

## Enforce selected dependency rules

Place an `arc-rules.toml` file at the workspace root to turn deliberate architecture decisions into executable checks. Apollo 18 requires production dependencies to remain acyclic, prevents the renderer from depending on either host adapter, and keeps the rasterizer independent of its sibling renderer modules.

```toml
[config]
version = 1
default_severity = "error"

[[rules]]
type = "no-cycles"
name = "production modules remain acyclic"
scope = "**"

[[rules]]
type = "forbidden-dependency"
name = "renderer must not depend on native host"
from = "apollo18-renderer"
to = "apollo18-native"

[[rules]]
type = "forbidden-dependency"
name = "renderer must not depend on web host"
from = "apollo18-renderer"
to = "apollo18-web"

[[rules]]
type = "forbidden-dependency"
name = "rasterizer facade must not depend on sibling modules"
from = "apollo18-renderer::rasterizer"
to = "apollo18-renderer::*"

[[rules]]
type = "forbidden-dependency"
name = "rasterizer internals must not depend on sibling modules"
from = "apollo18-renderer::rasterizer::**"
to = "apollo18-renderer::*"
```

Run the checks from the workspace root:

```bash
cargo arc check
```

An error-level violation produces a nonzero exit status. The canonical quality gate runs this command so local and automated checks reject forbidden dependencies and cycles.

Rule patterns use qualified crate and module names. A bare crate name includes its descendants, an exact module name selects only that module, `::*` selects direct children, `::**` selects all descendants, and bare `**` selects the complete workspace. The rasterizer constraint therefore needs one entry for its facade module and another for its descendants. Confirm names in the generated diagram before adding a rule: a pattern that matches nothing cannot report a violation.

## Understand the limits

An arc or rule violation is not proof of harmful coupling. The diagram does not directly describe:

- runtime call frequency,
- data flow,
- performance cost,
- conceptual cohesion,
- the quality of a public API, or
- every dependency that can be expressed without a visible `use`.

Use `cargo-arc` to formulate architectural questions, then answer them by reading the relevant interfaces and implementation.

## First exploration exercise

Generate the direct-module view and inspect these relationships:

1. `apollo18-web → apollo18-renderer`
2. `apollo18-native → apollo18-renderer`
3. `lunar_phase_animation → lunar_ephemeris`
4. `octasphere → rasterizer`
5. Dependencies entering `rasterizer::color` and `rasterizer::framebuffer`
6. Any cycle reported inside `apollo18-renderer`

Record a short observation for each category:

```text
Expected dependency:
Surprising dependency:
Possible cycle:
Module with highest apparent fan-in:
Module with highest apparent fan-out:
```

Add architecture rules only after understanding the current graph. Keep the rule set small and tied to recorded decisions rather than attempting to encode every current dependency.
