# Robert C. Martin's module-dependency tools

## Finding

In Matt Pocock's interview **“Uncle Bob on Software Fundamentals in the Age of AI,”** Robert C. Martin describes two custom tools that his agents built for him:

1. **[`arch-view`](https://github.com/unclebob/arch-view)** — an interactive architecture viewer for Clojure projects. It extracts namespace dependencies and displays them as a layered, drill-down diagram. It highlights cycles, shows incoming and outgoing dependency paths, and can open the source behind a leaf namespace. Martin describes it in the interview as “a nice little UML diagram” that shows module structure and dependency direction and lets him drill down to submodules and code. ([Interview, architecture chapter starting at 25:52](https://www.youtube.com/watch?v=zcLPGC-tvgk&t=1552s); [official repository README](https://github.com/unclebob/arch-view/blob/master/README.md))

2. **[`dependency-checker`](https://github.com/unclebob/dependency-checker)** — a deterministic architecture-policy checker for Clojure projects. A project configuration declares allowed and forbidden component dependencies; the checker reports violations and cycles and can fail the build. It also reports fan-in, fan-out, instability, abstractness, distance from the “main sequence,” and zone classification. In the interview, Martin says the specification tells the agents which modules may depend on which others and what direction dependencies should flow; violating agents must repair the design, for example through dependency inversion, an interface, or splitting a module. ([Interview, architecture chapter starting at 25:52](https://www.youtube.com/watch?v=zcLPGC-tvgk&t=1552s); [official repository README](https://github.com/unclebob/dependency-checker/blob/master/README.md))

These are not named commercial products in the interview. They are Martin's own public, agent-built utilities, currently specialized for Clojure. The viewer supports visual human inspection; the checker turns the intended dependency structure into an executable constraint suitable for automation/CI.

## Related but distinct tools

Martin also discusses CRAP analysis and mutation testing in the same interview, but those assess function risk/test quality rather than module dependency structure. His public language-specific CRAP implementations include [`crap4clj`](https://github.com/unclebob/crap4clj), [`crap4java`](https://github.com/unclebob/crap4java), and [`crap4go`](https://github.com/unclebob/crap4go).

## Rust alternatives

The closest combined equivalent is [`cargo-arc`](https://github.com/seflue/cargo-arc). It generates an interactive, collapsible SVG of crates and nested modules, traces module-level `use` dependencies across a workspace, and highlights cycles. Its `cargo arc check` mode also enforces forbidden dependencies, acyclicity, and layer direction from `arc-rules.toml`, making it analogous to both `arch-view` and `dependency-checker`. ([README](https://github.com/seflue/cargo-arc/blob/main/README.md); [v0.3.0 changelog](https://github.com/seflue/cargo-arc/blob/refs/tags/v0.3.0/CHANGELOG.md))

Other related tools are:

- [`cargo-modules`](https://github.com/Dikluwe/cargo-modules) — a mature crate-internal module structure/dependency visualizer with DOT/Graphviz output, JSON export, orphan detection, graph filtering, and an `--acyclic` check. It is less interactive and primarily analyzes one selected Cargo package at a time. ([README](https://github.com/Dikluwe/cargo-modules/blob/main/README.md))
- [`modkei`](https://github.com/levish0/modkei) — an Obsidian-style browser graph of Rust file dependencies, backed by rust-analyzer with a syntax-parser fallback. It is useful for exploration but does not advertise architecture-policy enforcement. ([README](https://github.com/levish0/modkei/blob/main/README.md))
- [`cargo-bylaw`](https://github.com/danielgerlag/cargo-bylaw) — architecture enforcement rather than visualization. It uses rust-analyzer's semantic graph and can constrain dependencies between modules, workspace crates, and external crates, enforce layer directions, and reject cycles in CI or Rust tests. ([README](https://github.com/danielgerlag/cargo-bylaw/blob/main/README.md))
