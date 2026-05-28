---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-030
title: "Math: MathML (HTML) + Path-Based (PDF) Output"
epic: EPIC-10
wave: 3
points: 5
priority: P1
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.10.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-math
target_module: slideforge-math
subsystems: [SS-13]
depends_on: [STORY-029]
blocks:
  - STORY-043
  - STORY-044
  - STORY-046
  - STORY-047
estimated_days: 2
---

# STORY-030: Math — MathML (HTML) + Path-Based (PDF) Output

## Subsystem Anchor Justification

SS-13 (Math) owns this story because `render_mathml` and `render_pdf_paths` are
methods on the `MathRenderer` trait that lives in `slideforge-math`. These are two
additional rendering targets from the same `MathAst` IR produced by STORY-029.
The HTML and PDF exporters call into this crate; they do not implement math rendering
themselves.

## Dependency Anchor Justifications

- Depends on STORY-029: The `MathAst` IR and `MathRendererImpl` struct are defined
  there. This story implements the two methods stubbed as `NotYetImplemented`.
- Blocks STORY-043/044 (PDF): PDF math rendering requires `render_pdf_paths`.
- Blocks STORY-046/047 (HTML/preview): HTML math requires `render_mathml`.

## Summary

Complete the `MathRenderer` plugin by implementing the remaining two format renderers:

1. **MathML output** for static HTML and web preview (BC-1.10.003 postconditions 3–4):
   `render_mathml(ast: &MathAst) -> Result<String, MathError>` produces a `<math>`
   element compliant with MathML Core Level 1 and accessible to screen readers.

2. **PDF path output** (BC-1.10.003 postcondition 5): `render_pdf_paths(ast: &MathAst)
   -> Result<SvgPaths, MathError>` produces a vector-path SVG string for PDF embedding
   via usvg+resvg. No text characters in the output — full rasterization avoidance for
   PDF/UA-1 compliance.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.10.003 | Math renders to OMML for PPTX/DOCX, MathML for HTML, paths for PDF | AC-001, AC-002, AC-003, AC-004, AC-005 |

## Acceptance Criteria

### AC-001: MathML output for HTML export
(traces to BC-1.10.003 postcondition 3)

`MathRendererImpl::render_mathml(ast: &MathAst) -> Result<String, MathError>` produces
a `<math>` element. For `$x^2 + y^2 = r^2$`:
```xml
<math xmlns="http://www.w3.org/1998/Math/MathML" display="inline">
  <mrow>
    <msup><mi>x</mi><mn>2</mn></msup>
    <mo>+</mo>
    <msup><mi>y</mi><mn>2</mn></msup>
    <mo>=</mo>
    <msup><mi>r</mi><mn>2</mn></msup>
  </mrow>
</math>
```
Display math (`MathMode::Display`) uses `display="block"` on the root `<math>` element.
The MathML namespace `xmlns="http://www.w3.org/1998/Math/MathML"` is present.

### AC-002: MathML is accessible to screen readers
(traces to BC-1.10.003 postcondition 3 — "accessible to screen readers")

The `<math>` element produced by `render_mathml` includes `aria-label` containing a
plain-text description of the expression derived from the `MathAst`. For
`$E = mc^2$`, the `aria-label` is `"E equals m times c squared"` (generated from
the AST node types). This plain-text generation is a best-effort heuristic for the
v1.0 supported subset.

### AC-003: PDF path output — vector SVG, no text characters
(traces to BC-1.10.003 postcondition 5)

`MathRendererImpl::render_pdf_paths(ast: &MathAst) -> Result<SvgPaths, MathError>`
produces an SVG string where all text has been converted to path data using a font
glyph outline engine. No `<text>` elements remain in the output. No `<image>`
elements. All geometry is `<path d="...">` elements with absolute coordinates.

`SvgPaths` is a newtype wrapper `SvgPaths(String)` containing the SVG string.

Implementation approach: render the MathAst to SVG via a KaTeX/font-based path
renderer, then normalize with usvg to remove any residual text elements. If the
rendering produces any `<text>` elements after normalization, return
`Err(MathError::TextRemainsInPathOutput)`.

### AC-004: Single MathAst source for all formats
(traces to BC-1.10.003 invariant 1)

`render_omml`, `render_mathml`, and `render_pdf_paths` each take `&MathAst` as
input. No re-parsing of the LaTeX source occurs in these methods. The AST is parsed
once (in `MathRendererImpl::parse`) and the same `MathAst` instance is passed to
all three format renderers.

### AC-005: Web preview uses KaTeX HTML+CSS
(traces to BC-1.10.003 postcondition 4)

For the web preview case, `render_mathml` returns MathML. The web preview client
(axum-served HTML) loads KaTeX CSS from a bundled asset and uses the MathML
output. No separate `render_katex` method is needed — MathML is the web preview
representation. The KaTeX stylesheet is bundled with the preview server
(STORY-047 responsibility).

## Tasks

- [ ] Implement `render_mathml` in `crates/slideforge-math/src/mathml.rs`
  - `MathAst → <math>` XML string
  - `display="inline"` or `display="block"` from `MathMode`
  - `aria-label` generation from AST node types (plain-text heuristic for v1.0 subset)
  - MathML namespace declaration
- [ ] Implement `render_pdf_paths` in `crates/slideforge-math/src/pdf_paths.rs`
  - `MathAst → SVG paths` using font glyph outlines
  - Normalize via usvg to ensure no `<text>` elements remain
  - Return `Err(TextRemainsInPathOutput)` if normalization cannot remove text
- [ ] Define `SvgPaths(String)` newtype in `src/types.rs`
- [ ] Remove `Err(NotYetImplemented)` stubs from STORY-029's implementations
- [ ] Add `MathError::TextRemainsInPathOutput` variant
- [ ] Write unit tests:
  - `$x^2$` → MathML contains `<msup>`, `display="inline"`, namespace present
  - `$$\sum$$` → `display="block"` on root `<math>`
  - `render_mathml` output has `aria-label`
  - `render_pdf_paths` output contains no `<text>` elements
  - Same `MathAst` can be passed to all three renderers without re-parsing
- [ ] Write `insta` snapshot tests for MathML output of representative expressions

## Previous Story Intelligence

STORY-029 stubbed `render_mathml` and `render_pdf_paths` with `Err(NotYetImplemented)`.
This story replaces those stubs. The `MathRendererImpl` struct already exists — this
story adds methods to it. Ensure the unit tests from STORY-029 still pass after this
story is delivered.

## Architecture Compliance Rules

1. **Pure core (SS-13)**: `render_mathml` and `render_pdf_paths` are pure functions.
   No I/O. No spawned processes.
2. **Single AST source (BC-1.10.003 invariant 1)**: No re-parsing. Both methods take
   `&MathAst` and produce format output.
3. **PDF/UA-1 compatibility**: PDF math paths must be vector (DI-014 via
   BC-1.10.003 invariant 3). The `render_pdf_paths` output must not contain
   `<text>` elements that would bypass PDF/UA-1 text extraction.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-math` (self) | workspace | Extends STORY-029 implementation |
| `usvg` | `=0.47.0` | SVG normalization to remove text in PDF path output |
| `quick-xml` | `=0.36.0` | MathML XML generation |
| `insta` | `=1.39.0` | snapshot tests for MathML output |

Note: The font glyph outline engine for PDF path rendering must be a pure Rust
library. Evaluate `ab_glyph` 0.2 or `ttf-parser` 0.21 for glyph outline extraction.
If no suitable library is available for the full supported LaTeX subset, render
math to rasterized PNG as fallback for PDF ONLY (not for PPTX/HTML) and document
this as a known limitation requiring v2 resolution.

Note: pulldown-latex provides built-in `push_mathml` for MathML generation. Use this
directly rather than hand-rolling MathML serialization from the internal AST.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-math/src/mathml.rs` | Create | MathML renderer |
| `crates/slideforge-math/src/pdf_paths.rs` | Create | PDF vector path renderer |
| `crates/slideforge-math/src/types.rs` | Modify | Add `SvgPaths` newtype |
| `crates/slideforge-math/src/error.rs` | Modify | Add `TextRemainsInPathOutput` variant |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~1,800 |
| BC-1.10.003 | ~1,500 |
| STORY-029 MathAst types | ~1,500 |
| MathML spec reference (subset) | ~1,000 |
| Test files to write | ~2,000 |
| **Total** | **~7,800** |

## Test Strategy

- **Unit tests**: MathML output for `^`, `_`, `\frac`, `\sum`, `\sqrt`; `display="block"`
  for `MathMode::Display`; `aria-label` present; no `<text>` in PDF path output;
  `render_pdf_paths` does not panic on any `MathAst` producible by STORY-029's parser.
- **Integration test stub**: `$x^2$` → `render_mathml` → `render_pdf_paths` → both
  succeed with no error. Full integration (inside an exported HTML/PDF) tested in
  STORY-046/043.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Display math (`$$...$$`) in MathML | `display="block"` on `<math>` root |
| EC-002 | Inline math inside bullets list item | `display="inline"` MathML embedded within paragraph |
| EC-003 | Math expression with `@{var}` substitution | Variable already substituted by STORY-029; `render_mathml` receives resolved `MathAst` |
| EC-004 | Math in `notes` register | MathML rendered in HTML preview speaker notes; PDF paths embedded in note content |
| EC-005 | usvg fails to normalize SVG paths | `Err(MathError::TextRemainsInPathOutput)` returned; caller emits E-EXP-004 |

## Forbidden Dependencies

Same as STORY-029. No exporter crates, no Node.js, no subprocess invocation.
