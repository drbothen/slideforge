---
document_type: demo-evidence-report
product: "slideforge"
story_id: STORY-072
title: "shape: Gradient Fills (FillSpec::Gradient)"
pipeline_run: "2026-06-08"
demo_type: "library"
recording_tool: "vhs"
status: complete
---

# Demo Evidence Report — STORY-072

## Product: slideforge
## Story: STORY-072 — shape: Gradient Fills (`FillSpec::Gradient`)
## Pipeline Run: 2026-06-08
## Demo Type: library (test-harness demos — no end-to-end DSL path per shape-pipeline-wiring dependency note)
## Recording Tool: VHS 0.10.0

---

## Scope Note

The end-to-end DSL path from a `.sf` source file through to rendered output is **not** wired in
production (deferred to FU-SHAPE-PIPELINE-WIRING). Gradient rendering is verified by directly
constructing `ShapeSpec { fill: FillSpec::Gradient { from, to }, ... }` in test fixtures. All demos
use `cargo nextest run` against the live crate under test. No example binaries were added (no
Cargo.toml modification needed; test suite is the demo vehicle).

---

## Per-AC Demo Recordings

| AC | Story | Description | Recording (.gif) | Recording (.webm) | Tests Run | Status |
|----|-------|-------------|------------------|-------------------|-----------|--------|
| AC-001 | STORY-072 | DSL parser accepts `fill gradient "#FF0000" to "#0000FF"`; interim AST string is `gradient #FF0000 to #0000FF` (unquoted); E-PAR-016 removed | [AC-001.gif](AC-001-dsl-gradient-syntax-accepted.gif) | [AC-001.webm](AC-001-dsl-gradient-syntax-accepted.webm) | 3 (slideforge-syntax) | recorded |
| AC-002 | STORY-072 | `FillSpec::Gradient { from: Rgb, to: Rgb }` IR construction; u8 channels (no f64); Hash+Eq+Clone; distinct from SolidColor/None; EC-005 same-from-to | [AC-002.gif](AC-002-fillspec-gradient-ir-construction.gif) | [AC-002.webm](AC-002-fillspec-gradient-ir-construction.webm) | 5 (slideforge-types) | recorded |
| AC-003 | STORY-072 | `layout::run()` passes `FillSpec::Gradient` through to `ShapeFrame.fill` verbatim; from+to colors preserved; alt text preserved | [AC-003.gif](AC-003-layout-gradient-passthrough.gif) | [AC-003.webm](AC-003-layout-gradient-passthrough.webm) | 12 (slideforge-layout) | recorded |
| AC-004 (PPTX) | STORY-072 | Native `a:gradFill` emission via ooxmlsdk typed builders; two `a:gs` stops at pos=0/100000; `a:lin ang=5400000` (top-to-bottom) | [AC-004-pptx.gif](AC-004-pptx-gradFill-emission.gif) | [AC-004-pptx.webm](AC-004-pptx-gradFill-emission.webm) | 9 (slideforge-pptx) | recorded |
| AC-004 (HTML) | STORY-072 | SVG-native `<defs><linearGradient id="sf-grad-...">` + `fill="url(#sf-grad-...)"` on `<rect>` (NOT CSS `background`) | [AC-004-html.gif](AC-004-html-svg-linearGradient.gif) | [AC-004-html.webm](AC-004-html-svg-linearGradient.webm) | 8 (slideforge-html) | recorded |
| AC-004 (DOCX) | STORY-072 | Solid fallback using `from` color (`<w:shd w:fill="..."/>`) + lint warning "DOCX gradient fill downgraded to solid" | [AC-004-docx.gif](AC-004-docx-solid-fallback-warning.gif) | [AC-004-docx.webm](AC-004-docx-solid-fallback-warning.webm) | 5 (slideforge-docx) | recorded |
| AC-005 | STORY-072 | Alt text required on gradient shapes; missing alt => `LayoutError::MissingAlt`; decorative: true => `AltText::Decorative` | [AC-005.gif](AC-005-alt-text-required-on-gradient-shapes.gif) | [AC-005.webm](AC-005-alt-text-required-on-gradient-shapes.webm) | 30 (slideforge-layout) | recorded |

---

## Coverage Mapping

| Acceptance Criterion | Edge Cases Demonstrated | Total Tests |
|----------------------|------------------------|-------------|
| AC-001: DSL gradient syntax accepted | EC-002 (lowercase hex), LOW-001 pinned string | 3 |
| AC-002: FillSpec::Gradient IR construction | EC-005 (same from=to), no-f64 invariant | 5 |
| AC-003: Layout passthrough verbatim | EC-001 (missing alt), EC-004 (multi-shape errors), EC-005, EC-006 (decorative) | 12 |
| AC-004 PPTX: a:gradFill emission | stop positions, ang=5400000, alt on cNvPr, decorative empty descr, EC-005 | 9 |
| AC-004 HTML: SVG linearGradient | from/to colors in style, accessible name, aria-hidden decorative, EC-005 | 8 |
| AC-004 DOCX: solid fallback + warning | EC-003 (fallback uses from), EC-005 (same-from-to valid) | 5 |
| AC-005: alt text contract unchanged | EC-001 (missing alt => MissingAlt), EC-006 (decorative => Decorative) | 30 |
| **Total** | | **72** |

---

## Path Coverage

Each acceptance criterion has both a success path and an error path demonstrated:

| AC | Success Path | Error Path |
|----|-------------|-----------|
| AC-001 | `fill gradient "#FF0000" to "#0000FF"` parses; AST string is `gradient #FF0000 to #0000FF` | (E-PAR-015 short/alpha hex rejection covered in AC-002 IR tests) |
| AC-002 | `FillSpec::Gradient { from: Rgb{255,0,0}, to: Rgb{0,0,255} }` constructable | Two `Gradient` with different from-colors are not equal |
| AC-003 | Gradient fill preserved verbatim in `ShapeFrame.fill` | Gradient shape without alt => `LayoutError::MissingAlt` |
| AC-004 PPTX | `a:gradFill` present in serialized XML with correct stops and angle | Same from=to is valid (EC-005, not an error) |
| AC-004 HTML | `<linearGradient>` def present; `fill="url(#...)"` on `<rect>` | Decorative gradient emits `aria-hidden="true"` |
| AC-004 DOCX | Export succeeds (no panic), lint warning emitted | Solid fallback from color verified in XML |
| AC-005 | Gradient with alt => `AltText::Provided` preserved | Gradient without alt => `LayoutError::MissingAlt` |

---

## Toolchain

| Tool | Version | Status |
|------|---------|--------|
| VHS | 0.10.0 | installed |
| cargo nextest | workspace | installed |
| FiraCode Nerd Font Mono | installed | used as VHS FontFamily |

---

## PR Embedding Snippet

```markdown
## Demo Evidence — STORY-072: shape: Gradient Fills

| AC | Demo |
|----|------|
| AC-001: DSL gradient syntax | ![AC-001](docs/demo-evidence/STORY-072/AC-001-dsl-gradient-syntax-accepted.gif) |
| AC-002: FillSpec::Gradient IR | ![AC-002](docs/demo-evidence/STORY-072/AC-002-fillspec-gradient-ir-construction.gif) |
| AC-003: Layout passthrough | ![AC-003](docs/demo-evidence/STORY-072/AC-003-layout-gradient-passthrough.gif) |
| AC-004 PPTX: a:gradFill | ![AC-004-pptx](docs/demo-evidence/STORY-072/AC-004-pptx-gradFill-emission.gif) |
| AC-004 HTML: SVG linearGradient | ![AC-004-html](docs/demo-evidence/STORY-072/AC-004-html-svg-linearGradient.gif) |
| AC-004 DOCX: solid fallback | ![AC-004-docx](docs/demo-evidence/STORY-072/AC-004-docx-solid-fallback-warning.gif) |
| AC-005: alt text contract | ![AC-005](docs/demo-evidence/STORY-072/AC-005-alt-text-required-on-gradient-shapes.gif) |
```

---

## Notes

- All 7 recordings run the actual implementation via `cargo nextest run` against the live crate.
- No example binaries were added to avoid Cargo.toml modifications; the test suite is the demo vehicle.
- `FillSpec::Gradient` was NOT in the v1.0 codebase before STORY-072; the recordings show the new variant passing all AC-specific tests.
- The `ShapeNode → ShapeSpec` decode (item b in the AC-002 pipeline-wiring note) is **not** demonstrated because it is deferred to FU-SHAPE-PIPELINE-WIRING. The IR path is exercised via directly constructed `ShapeSpec` fixtures.
- DOCX gradient rendering intentionally degrades to solid fill — this is a documented v1.0 limitation, not a bug. The lint warning test confirms the warning text is present.
