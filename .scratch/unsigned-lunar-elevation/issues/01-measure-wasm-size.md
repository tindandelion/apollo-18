# 01: Add repeatable Wasm size measurement

**What to build:** Provide a repeatable repository diagnostic that creates the release web artifact and reports its raw and gzip sizes, giving this and future Wasm-size experiments a trustworthy baseline.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] One unattended repository command builds the release web host and reports the generated Wasm artifact's raw byte count and gzip byte count.
- [ ] The diagnostic writes generated output only under ignored build directories and does not modify the checked-in web distribution.
- [ ] Repeated runs against an unchanged build produce clear, comparable output.
- [ ] The command and interpretation of its metrics are documented with the web testing guidance.
- [ ] The current floating-point-elevation baseline is captured using the diagnostic before the asset replacement begins.
- [ ] The canonical quality gate passes.
