---
document_type: demo-evidence-report
product: "slideforge-diagrams"
pipeline_run: "2026-06-08"
story_id: STORY-079
demo_type: library
recording_tool: vhs
status: complete
---

# Demo Evidence Report — STORY-079: SVG DoS Hardening

## Product: slideforge-diagrams
## Pipeline Run: 2026-06-08
## Story: STORY-079 — SVG DoS hardening — byte-size cap + nesting-depth guard

---

## Summary

STORY-079 adds two DoS guards to `usvg_normalize` in `crates/slideforge-diagrams/src/normalize.rs`:

- **SEC-002 (CWE-400)** — `MAX_SVG_BYTES = 50 MiB` byte-size cap fires before `usvg::Tree::from_str` to prevent heap exhaustion on oversized input.
- **SEC-001 (CWE-674)** — `MAX_SVG_NESTING_DEPTH = 64` iterative DFS depth guard fires after parse but before re-serialization to prevent stack exhaustion on pathologically nested group trees.

Both guards return `DiagramError::SvgNormalizationFailed` whose display includes `E-EXP-004`.

This is a backend security hardening story with no user-facing CLI surface. Demo evidence is terminal recordings of the cargo-nextest test suite running the six SEC guard tests, with each recording scoped to the AC(s) it demonstrates.

---

## Per-AC Demo Recordings

| AC | Description | Tape | GIF | WebM | Tests Covered | Status |
|----|-------------|------|-----|------|---------------|--------|
| AC-001 | Oversized SVG rejected before parse (SEC-002, CWE-400) | [AC-001.tape](AC-001-oversize-svg-rejected.tape) | [AC-001.gif](AC-001-oversize-svg-rejected.gif) | [AC-001.webm](AC-001-oversize-svg-rejected.webm) | `test_bc_1_12_003_sec_dos_svg_oversize_rejected`, `test_bc_1_12_003_sec_size_exact_boundary_passes`, `test_bc_1_12_003_sec_size_one_over_boundary_fails` | recorded |
| AC-002 | Deeply nested SVG rejected after parse, before re-serialisation (SEC-001, CWE-674) | [AC-002.tape](AC-002-deep-nesting-rejected.tape) | [AC-002.gif](AC-002-deep-nesting-rejected.gif) | [AC-002.webm](AC-002-deep-nesting-rejected.webm) | `test_bc_1_12_003_sec_dos_svg_deep_nesting_rejected`, `test_bc_1_12_003_sec_depth_exact_boundary_passes` | recorded |
| AC-003 | Valid SVG within both limits passes through unaffected | [AC-003.tape](AC-003-valid-svg-passes.tape) | [AC-003.gif](AC-003-valid-svg-passes.gif) | [AC-003.webm](AC-003-valid-svg-passes.webm) | `test_bc_1_12_003_sec_dos_svg_valid_input_unaffected` | recorded |

---

## AC Coverage Details

### AC-001: Oversized SVG rejected before parse

**Guard:** `MAX_SVG_BYTES = 52_428_800` (50 MiB). Check fires before `font_db()` and before `usvg::Tree::from_str`.

**Tests shown in recording:**

| Test | Input | Expected | Asserts |
|------|-------|----------|---------|
| `test_bc_1_12_003_sec_size_exact_boundary_passes` | SVG of exactly `MAX_SVG_BYTES` bytes | `Ok(NormalizedDiagramSvg)` | boundary is exclusive |
| `test_bc_1_12_003_sec_size_one_over_boundary_fails` | SVG of `MAX_SVG_BYTES + 1` bytes | `Err(SvgNormalizationFailed)` | +1 over limit rejects |
| `test_bc_1_12_003_sec_dos_svg_oversize_rejected` | SVG of `MAX_SVG_BYTES + 1` bytes | `Err(SvgNormalizationFailed)` with `"bytes exceeds"` + `"E-EXP-004"` | error message non-vacuous |

Error path: recording shows all three tests PASS, demonstrating that 50 MiB passes and 50 MiB + 1 byte fails with E-EXP-004.

---

### AC-002: Deeply nested SVG rejected after parse, before re-serialisation

**Guard:** `MAX_SVG_NESTING_DEPTH = 64`. Iterative stack-based DFS over parsed `usvg::Tree` via `Group::children()`. Root group counts as depth 1.

**Tests shown in recording:**

| Test | Input | Expected | Asserts |
|------|-------|----------|---------|
| `test_bc_1_12_003_sec_depth_exact_boundary_passes` | SVG with exactly 64 nested `<g>` levels | `Ok(NormalizedDiagramSvg)` | boundary is exclusive |
| `test_bc_1_12_003_sec_dos_svg_deep_nesting_rejected` | SVG with 65 nested `<g>` levels | `Err(SvgNormalizationFailed)` with `"nesting depth"` + `"64-level limit"` | iterative DFS detects depth; error is non-vacuous |

Error path: recording shows both tests PASS, demonstrating depth-64 passes and depth-65 fails.

---

### AC-003: Valid SVG within limits passes through unaffected

**Test shown in recording:**

| Test | Input | Expected | Asserts |
|------|-------|----------|---------|
| `test_bc_1_12_003_sec_dos_svg_valid_input_unaffected` | `simple_geometry_svg()` fixture (real-world Mermaid flowchart) | `Ok(NormalizedDiagramSvg)` | guards do not regress existing behaviour |

---

## Edge Case Coverage (per story spec)

| ID | Description | Covered by Test | AC |
|----|-------------|-----------------|-----|
| EC-001 | SVG exactly at `MAX_SVG_BYTES` passes | `test_bc_1_12_003_sec_size_exact_boundary_passes` | AC-001 |
| EC-002 | SVG at `MAX_SVG_BYTES + 1` fails | `test_bc_1_12_003_sec_size_one_over_boundary_fails` | AC-001 |
| EC-003 | SVG with exactly `MAX_SVG_NESTING_DEPTH` levels passes | `test_bc_1_12_003_sec_depth_exact_boundary_passes` | AC-002 |
| EC-004 | SVG with `MAX_SVG_NESTING_DEPTH + 1` levels fails | `test_bc_1_12_003_sec_dos_svg_deep_nesting_rejected` | AC-002 |
| EC-006 | Valid Mermaid flowchart SVG passes | `test_bc_1_12_003_sec_dos_svg_valid_input_unaffected` | AC-003 |

---

## Toolchain

| Tool | Version | Status |
|------|---------|--------|
| VHS | 0.10.0 | installed |
| cargo-nextest | workspace | installed |
| FiraCode Nerd Font Mono | system | installed |

---

## PR Embedding Snippet

```markdown
### STORY-079 Demo Evidence — SVG DoS Hardening

**AC-001 — SEC-002: Oversized SVG rejected before parse**
![AC-001](docs/demo-evidence/STORY-079/AC-001-oversize-svg-rejected.gif)

**AC-002 — SEC-001: Deeply nested SVG rejected after parse**
![AC-002](docs/demo-evidence/STORY-079/AC-002-deep-nesting-rejected.gif)

**AC-003 — Valid SVG passes through unaffected**
![AC-003](docs/demo-evidence/STORY-079/AC-003-valid-svg-passes.gif)
```

---

## Notes

- No example binary was added. The existing unit tests in `normalize.rs#[cfg(test)] mod tests` are the canonical executable evidence; VHS tapes drive `cargo nextest` directly against those tests.
- WebM is the primary archival format; GIF is provided for inline PR embedding.
- `MAX_SVG_BYTES` and `MAX_SVG_NESTING_DEPTH` constants match `crates/slideforge-pdf/src/svg_embed.rs` exactly (defense-in-depth invariant).
