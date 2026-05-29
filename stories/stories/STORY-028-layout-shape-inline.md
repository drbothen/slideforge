---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-028
title: "Layout: shape: Block + Rich Inline Formatting"
epic: EPIC-07
wave: 3
points: 5
priority: P1
tdd_mode: strict
status: draft
spec_version: "1.5"
behavioral_contracts: [BC-3.04.001, BC-3.05.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
crate: slideforge-layout
target_module: slideforge-layout
subsystems: [SS-05]
depends_on: [STORY-026]
blocks:
  - STORY-037
  - STORY-038
  - STORY-043
  - STORY-046
estimated_days: 2
---

# STORY-028: Layout — shape: Block + Rich Inline Formatting

## Subsystem Anchor Justification

SS-05 (Layout Engine) owns this story's scope because `shape:` block positioning and
`InlineNode` layout coordinates are computed during the `Deck → LaidOutDeck`
transformation. SS-06 (PPTX), SS-07 (PDF), and SS-09 (HTML) exporters receive
positioned shapes from the layout output — they do not compute positions themselves.

## Dependency Anchor Justifications

- Depends on STORY-026: `Frame`, `BoundingBox`, and `LaidOutSlide` types from
  STORY-026 are extended with `FrameContent::Shape` and `FrameContent::InlineRun`
  variants. The shape layout is appended to existing `frames` in each `LaidOutSlide`.
- Blocks STORY-037/038 (PPTX) and STORY-043 (PDF): Both need the positioned shape
  data to generate `<p:sp>` and PDF shape elements. STORY-046 (HTML) renders shapes
  as positioned SVG elements using the same BoundingBox data.

## Summary

Extend the layout engine with two capabilities:

1. **shape: block layout (BC-3.04.001)**: Process `Shape` nodes from the `Deck` IR
   and produce `Frame`s with `FrameContent::Shape`. Convert user-unit positions
   (inches, em) to integer EMU. Enforce that every shape has `alt` or
   `decorative: true` (defensive check; primary enforcement is in validation stage).

2. **Rich inline formatting layout (BC-3.05.001)**: All 12 `InlineNode` variants
   are represented in `LaidOutSlide.frames` as typed spans with their format
   metadata. The layout stage records the inline type; exporters translate to format-
   specific markup (OMML, HTML tags, PDF text spans).

## Behavioral Contracts

| BC | Title | Version | Covered ACs |
|----|-------|---------|-------------|
| BC-3.04.001 | shape: block declares custom shape with type/position/fill/text/alt | v1.4 | AC-001, AC-002, AC-003, AC-004, AC-BC-A1, AC-BC-A2, AC-BC-A3, AC-BC-A4, AC-BC-A5, AC-BC-A6, AC-INT-1 |
| BC-3.05.001 | All 12 inline format types render to correct output per format | v1.3.2 | AC-005, AC-006, AC-007, AC-BC-A7, AC-BC-A8 |
| LayoutError | Canonical field name source_slide_index across all variants | v1.0 | AC-BC-A9 |

## Acceptance Criteria

### AC-001: Shape EMU conversion from user-declared units
(traces to BC-3.04.001 postcondition 1 — position in integer EMU)

A `Shape` node from the `Deck` IR with `position x 0.5in y 1.0in width 2.0in
height 1.0in` is laid out as:
```rust
BoundingBox {
    x: Emu(457_200),    // 0.5 * 914400
    y: Emu(914_400),    // 1.0 * 914400
    width: Emu(1_828_800), // 2.0 * 914400
    height: Emu(914_400),  // 1.0 * 914400
}
```
All coordinates are `Emu(i64)`. No `f64` in the layout IR. `em` units use
`1em = brand.font_size_emu` (default: `Emu(457_200)` = 0.5 inch at 36pt).

### AC-002: Shape Frame added to LaidOutSlide
(traces to BC-3.04.001 postcondition 4 — reading order in PPTX spTree)

Each `Shape` from the source slide becomes a `Frame` appended after all
placeholder frames in `LaidOutSlide.frames`. Source order within a slide is
preserved (first shape in source → first shape frame appended).

```rust
Frame {
    bounding_box: BoundingBox { ... },
    content: FrameContent::Shape(ShapeFrame {
        shape_type: ShapeType::Rect,     // or Ellipse, Arrow, etc.
        fill: FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 }),
        text: Option<Vec<InlineNode>>,
        alt: AltText::Provided("Blue rectangle".into()),
    }),
}
```

### AC-003: Off-canvas shape position produces layout warning (not error)
(traces to BC-3.04.001 EC-002)

A shape with `position x -0.5in y 1.0in` (negative x) produces a
`LayoutWarning::OffCanvas { source_slide_index, shape_type, x_emu, y_emu }` warning.
This warning is accumulated via `DiagnosticSink` but does NOT halt layout. The
shape frame is still produced at the declared position.

### AC-004: Decorative shape emits empty AltText
(traces to BC-3.04.001 precondition 3)

A `shape:` with `decorative: true` produces `AltText::Decorative` in the
`ShapeFrame`. PPTX exporter maps this to `<p:cNvPr descr="" hidden="1"/>`.
PDF exporter marks the element as an Artifact. HTML exporter uses `alt=""`.

### AC-005: All 12 InlineNode variants represented in layout output
(traces to BC-3.05.001 postcondition — all 12 types produce correct IR)

All 12 `InlineNode` variants are preserved in `FrameContent::TextRun` records
within `LaidOutSlide.frames`. The layout stage does NOT produce format-specific
markup — it only carries the variant type and its attributes.

Production enum (authoritative — see `crates/slideforge-types/src/inline.rs`):

```rust
pub enum InlineNode {
    // Leaf nodes (terminal — no nested InlineNodes)
    Plain(Arc<str>),           // plain text
    Code(Arc<str>),            // monospace, no further inline processing
    Xref(Arc<str>),            // cross-ref target identifier
    Math(MathNode),            // LaTeX source; rendered by slideforge-math

    // Container nodes (nested content — supports inline composition)
    Bold(Vec<InlineNode>),
    Italic(Vec<InlineNode>),
    Footnote(Vec<InlineNode>),
    Superscript(Vec<InlineNode>),
    Subscript(Vec<InlineNode>),
    Strikethrough(Vec<InlineNode>),
    Highlight(Vec<InlineNode>),

    // Structured node (named fields)
    Link {
        text: Vec<InlineNode>,   // display content, can be styled
        url: Arc<str>,           // target URL
    },
}
```

Container variants (`Bold`, `Italic`, `Footnote`, `Superscript`, `Subscript`,
`Strikethrough`, `Highlight`, `Link.text`) take `Vec<InlineNode>` to enable
nesting. Leaf variants (`Plain`, `Code`, `Xref`) take `Arc<str>`. `Math` takes
`MathNode`. The 12-variant count is authoritative per BC-3.05.001 v1.3.2.

Each exporter translates its relevant variants to format output. The layout stage
preserves the `InlineNode` sequence verbatim in the frame's text content.

### AC-006: Nested inline formatting preserved
(traces to BC-3.05.001 EC-001 — nested bold inside italic)

When the DSL produces nested inline formatting (e.g., bold inside italic), the
`Deck` IR contains nested `InlineNode` sequences. The layout stage preserves
nesting depth. Exporters receive the full nesting and produce:
- PPTX: `<a:rPr b="1" i="1">` (both attributes on one run)
- HTML: `<em><strong>text</strong></em>`

### AC-007: Xref target validation at layout time
(traces to BC-3.05.001 EC-002)

During layout, `InlineNode::Xref(target)` references are validated against the
slide titles present in the `Deck`. Unknown xref targets accumulate
`LayoutWarning::XrefTargetNotFound { target, source_slide_index }`. This is a warning,
not an error, to allow accumulation (DI-018).

### AC-BC-A1: ShapeSpec.position EMU conversion end-to-end
(traces to BC-3.04.001 postcondition 2 — exact conversion constants)

The canonical conversion test vector must pass: `shape: type rect position x 0.5in
y 1.0in width 2.0in height 1.0in` produces the following `BoundingBox` in
`LaidOutDeck`:
```
BoundingBox { x: Emu(457_200), y: Emu(914_400), width: Emu(1_828_800), height: Emu(914_400) }
```
Computation: `Inches(500) * 914_400 / 1_000 = 457_200`. All arithmetic is `i64`
integer division; no `f64`. Verified via an integration test in
`crates/slideforge-layout/tests/` that calls `layout::run()` and asserts
`frames[0].bounding_box` equals the expected EMU values. (BC-3.04.001 v1.4,
postcondition 2 canonical test vector)

### AC-BC-A2: Hex color case-insensitive; short/alpha forms rejected
(traces to BC-3.04.001 invariant 5 — hex color contract)

Hex fill colors are case-insensitive. Both `#FF6F00` (uppercase) and `#ff6f00`
(lowercase) parse to the same `Rgb { r: 255, g: 111, b: 0 }`. Short-form `#RGB`
(3 digits) is rejected with E-PAR-015. Alpha form `#RRGGBBAA` (8 digits) is
rejected with E-PAR-015. Error message includes the file:line:col span. (BC-3.04.001
v1.4, invariant 5 and EC-006/EC-007/EC-008/EC-009)

### AC-BC-A3: shape_type closed vocabulary; unknown keyword produces E-PAR-012
(traces to BC-3.04.001 invariant 4 — closed type vocabulary)

The `shape_type` field accepts exactly 6 keywords: `rect`, `ellipse`, `arrow`,
`line`, `star`, `roundRect`. Any other keyword (e.g., `frobnicator`) produces
E-PAR-012 with a human-readable message listing the known types:
```
Unknown shape type 'frobnicator' at <file>:<line>:<col>.
Known types: [rect, ellipse, arrow, line, star, roundRect]
```
There is NO silent `ShapeType::Custom(...)` fallback. (BC-3.04.001 v1.4, invariant 4
and EC-005; canonical rule: no silent fallbacks per CLAUDE.md)

### AC-BC-A4: Off-canvas inclusive boundary semantics
(traces to BC-3.04.001 invariant 6 — boundary semantics)

A shape exactly touching the page edge is ON-canvas (not a warning). Specifically:
`x + width == page_width` is NOT off-canvas; `x + width > page_width` by even 1 EMU
IS off-canvas. Canonical test vector: `x=8.0in, width=2.0in` on a 10-inch canvas
(`page_width = Emu(9_144_000)`) produces NO `LayoutWarning::OffCanvas`. A shape with
`x=8.0in, width=2.0in + 1 EMU` DOES produce the warning. (BC-3.04.001 v1.4,
invariant 6 and EC-002/EC-003)

### AC-BC-A5: MissingAlt error carries SourceSpan
(traces to BC-3.04.001 invariant 7 — error span requirement)

`LayoutError::MissingAlt` carries a `span: SourceSpan` field pointing to the
offending `shape:` block in the source file. This satisfies the CLAUDE.md rule that
all errors must carry source spans. The span must be non-default (file + line + col
set) when a `shape:` block without `alt` or `decorative: true` reaches the layout
stage. (BC-3.04.001 v1.4, invariant 7)

### AC-BC-A6: Multi-shape error accumulation in LayoutError::Multiple
(traces to BC-3.04.001 postcondition 6 — multi-error accumulation)

When a slide contains two or more shapes that are both missing `alt` and
`decorative: true`, ALL `LayoutError::MissingAlt` errors are accumulated before
returning. The layout function returns `Err(LayoutError::Multiple(vec![err1, err2,
...]))` (or equivalent accumulator variant). It does NOT bail on the first error.
Canonical test vector: slide with 2 shapes both missing alt → `Vec<LayoutError>`
with 2 `MissingAlt` entries. (BC-3.04.001 v1.4, postcondition 6 and EC-010)

### AC-BC-A7: InlineDepthExceeded at depth 65
(traces to BC-3.05.001 invariant 4 — nesting depth bound)

Inline trees nested deeper than 64 levels produce
`LayoutError::InlineDepthExceeded { source_slide_index, depth: 65 }` (or the actual
exceeded depth). This is a HARD error — output is NOT produced for the affected
slide. Canonical test vector: a tree of 65 nested `Bold(vec![Bold(vec![...])])` nodes
triggers the error. Depth 64 is the maximum permitted (no error). (BC-3.05.001 v1.3.2,
invariant 4 and EC-006, with canonical error code E-LAY-005)

### AC-BC-A8: Xref inside MathNode NOT validated
(traces to BC-3.05.001 invariant 5 — math xref validation boundary)

`InlineNode::Xref` references that appear inside a `MathNode`'s content are NOT
validated by the xref validation pass at layout time. The layout engine does not
traverse into `MathNode` for xref resolution. This is an explicit v1.0 scope
boundary (not an oversight). A unit test must confirm: a `Math(MathNode { latex:
"\\xref{missing-slide}", ... })` variant does NOT produce
`LayoutWarning::XrefTargetNotFound`. (BC-3.05.001 v1.3.2, invariant 5 and EC-007)

### AC-BC-A9: Canonical field name source_slide_index across all LayoutError variants
(traces to BC-3.04.001 and BC-3.05.001 — structural consistency across error variants)

All `LayoutError` variants that carry a slide identifier MUST use the field name
`source_slide_index: usize` (not `slide_index`, not `idx`). This is a structural
consistency invariant across `MissingAlt`, `InlineDepthExceeded`, `OffCanvas`, and
any future variants. Verified via compilation (field name mismatch → compile error in
match arms).

### AC-INT-1: layout::run end-to-end integration
(traces to BC-3.04.001 postcondition 3 — shape frames in LaidOutDeck; F-CRIT-001
integration anchor)

`layout::run()` populates `LaidOutDeck` with shape frames from
`ContentBlock::Shape` blocks AND runs inline xref validation across all slides in
one pass. Verified via an integration test in
`crates/slideforge-layout/tests/layout_integration.rs` (or equivalent test module)
that:
1. Constructs a `Deck` with at least one slide containing a `ContentBlock::Shape`
2. Calls `layout::run(&deck, &brand_config)`
3. Asserts the resulting `LaidOutDeck` contains a `Frame` with `FrameContent::Shape`
4. Asserts that an `InlineNode::Xref` with an unknown target in the same deck
   produces `LayoutWarning::XrefTargetNotFound`

This closes F-CRIT-001 (the integration wire between parser output and layout output).

## ShapeSpec Position Schema

Per BC-3.04.001 v1.4 postcondition 1 and the data-engineer schema commit (9ea373a8),
the canonical `ShapeSpec` and supporting types in `slideforge-types/src/specs.rs` are:

```rust
pub struct ShapeSpec {
    pub shape_type: ShapeType,
    pub position: ShapePosition,
    pub fill: FillSpec,
    pub text: Option<Vec<InlineNode>>,
    pub alt: Option<AltText>,
    pub decorative: bool,
    pub span: SourceSpan,
}

pub struct ShapePosition {
    pub x: ShapeUnit,
    pub y: ShapeUnit,
    pub width: ShapeUnit,
    pub height: ShapeUnit,
}

pub enum ShapeUnit {
    /// inches × 1000 stored as i64 (avoids f64)
    /// e.g., 0.5in → Inches(500)
    Inches(i64),
    /// em × 1000 stored as i64
    /// e.g., 2em → Em(2000)
    Em(i64),
}

pub enum FillSpec {
    SolidColor(Rgb),
    None,
    // Gradient: DEFERRED to STORY-072 (shape-gradient-fills)
}
```

Note: `ShapeType::RoundRect` is the correct v1.4 variant (previously `Custom` was
listed — that was incorrect; unknown keywords are parse errors, not `Custom` fallbacks).

## Tasks

- [ ] Add `FrameContent::Shape(ShapeFrame)` and `FrameContent::TextRun(Vec<InlineNode>)` to `Frame` enum in `src/types.rs`
- [ ] Implement `ShapeFrame` struct with `shape_type`, `fill`, `text`, `alt` fields
- [ ] Implement `ShapeType` enum: `Rect`, `Ellipse`, `Arrow`, `Line`, `Star`, `RoundRect` (no `Custom` — unknown keyword is a parse error per BC-3.04.001 invariant 4)
- [ ] Implement `FillSpec` enum: `SolidColor(Rgb)`, `None` (Gradient deferred to STORY-072 per BC-3.04.001 v1.4.2 Deferred Surfaces)
- [ ] Implement `AltText` enum: `Provided(Arc<str>)`, `Decorative`
- [ ] Implement shape layout pass in `layout::run()`: iterate slide shapes → produce `Frame`s
- [ ] Implement unit conversion: `from_inches`, `from_em` for shape positions
- [ ] Implement off-canvas detection (negative or > page dimensions) → `LayoutWarning`
- [ ] Verify `InlineNode` enum from `slideforge-types` has all 12 variants (confirmed in `crates/slideforge-types/src/inline.rs` lines 22-64)
- [ ] Implement xref target validation pass at end of layout run
- [ ] Write unit tests:
  - `0.5in` → `Emu(457_200)`, `-0.5in` → off-canvas warning
  - `decorative: true` → `AltText::Decorative`
  - all 12 inline types survive layout pass unchanged
  - xref to missing target → warning accumulated

## Previous Story Intelligence

STORY-026 established that `LaidOutSlide.frames` is a `Vec<Frame>`. This story
adds new `FrameContent` variants to that enum. The implementer must extend the
exhaustive match arms in any existing code that matches on `FrameContent`. There
are no existing exporters yet (they are Wave 4), so there are no match arm breakage
risks — but proptest and snapshot tests from STORY-026 must still pass.

## Architecture Compliance Rules

1. **Shape DSL is the only user path for custom shapes (DI-021, BC-3.04.002)**: The
   layout stage handles `shape:` DSL blocks. The `raw` keyword is rejected at parse
   time (STORY-009). No code path here processes raw XML.
2. **Alt text required (DI-001)**: Defensive check: if a `Shape` node reaches the
   layout stage without `alt` or `decorative: true`, it is an internal invariant
   violation. Layout returns `Err(LayoutError::MissingAlt { source_slide_index })`. The
   primary enforcement is in `slideforge-validate` (STORY-015).
3. **Integer EMU for all positions (DI-010, ADR-013)**: `em` units are resolved to
   EMU using brand font size at layout time. No `f64` in `BoundingBox`.
4. **InlineNode variants are exhaustive**: The layout stage must handle ALL 12
   `InlineNode` variants without a wildcard catch-all. Missing variants are a
   compile error.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace | `InlineNode`, `Shape`, `Rgb`, `Deck` IR types |
| `thiserror` | `=2.0.18` | `LayoutError` and `LayoutWarning` |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-layout/src/types.rs` | Modify | Add `ShapeFrame`, `FillSpec`, `AltText`, `ShapeType` |
| `crates/slideforge-layout/src/shapes.rs` | Create | Shape layout pass logic |
| `crates/slideforge-layout/src/inline.rs` | Create | Inline node validation + xref check |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec (v1.2 — expanded ACs) | ~4,800 |
| STORY-026 layout types | ~1,500 |
| BC files (2 BCs: BC-3.04.001 v1.4 + BC-3.05.001 v1.3.2) | ~5,000 |
| `slideforge-types` InlineNode definition | ~1,000 |
| `slideforge-types/src/specs.rs` ShapeSpec schema | ~800 |
| Test files to write | ~3,000 |
| **Total** | **~16,100** |

## Test Strategy

- **Unit tests**: EMU conversion for all unit types (in, em); off-canvas warning for
  negative coordinates; decorative shape produces `AltText::Decorative`; all 12
  `InlineNode` variants survive the layout pass; xref to unknown target accumulates
  `LayoutWarning::XrefTargetNotFound`.
- **Snapshot tests**: `LaidOutSlide` frames for a reference slide with one shape of
  each supported type.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | shape without alt and without decorative | `LayoutError::MissingAlt` (defensive; primary check in validation) |
| EC-002 | Shape with negative position | `LayoutWarning::OffCanvas`; shape frame still produced |
| EC-003 | Shape fill color contrast warning | Delegated to BC-5.01.003 (validation stage, STORY-017) |
| EC-004 | shape in DOCX export | Shape frame carries `target_formats: All`; DOCX exporter renders as `<w:drawing>` |
| EC-005 | Nested bold inside italic | Both `Bold` and `Italic` variants preserved; exporter merges |

## Forbidden Dependencies

Same constraints as STORY-026. No exporter crates. `slideforge-layout` MUST NOT
depend on `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, or `slideforge-html`.
Build MUST fail if those crates appear in `slideforge-layout/Cargo.toml` dependencies.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-25 | story-writer | Initial decomposition |
| 1.1 | 2026-05-28 | story-writer | BC-3.04.001/BC-3.05.001 v1.3 adjudication — DONE_WITH_CONCERN resolved; 11→12 variant count; AC-005 enum corrected (Arc<str>→Vec<InlineNode> for container variants); AC-BC-A1 through AC-BC-A9 + AC-INT-1 added; ShapeSpec position schema section added; roundRect added to ShapeType; NFR-025 added |
| 1.2 | 2026-05-28 | story-writer | Minor: spec_version field added to frontmatter; Forbidden Dependencies made explicit |
| 1.3 | 2026-05-29 | architect | Pass-3 adjudication: BC version refs updated to BC-3.04.001 v1.4 and BC-3.05.001 v1.3.2 (F-MED-001); OffCanvas example updated to source_slide_index canonical name (F-MED-002) |
| 1.4 | 2026-05-29 | story-writer | Pass-5 drift fix (F-P5-LOW-002): AltText::Explicit → AltText::Provided (lines 100, 339) to match canonical slideforge-types/src/specs.rs:143 |
| 1.5 | 2026-05-29 | product-owner | Pass-8 sweep changes (commit 3e892d07): BC-3.04.001 version references updated to v1.4.2; task list FillSpec description corrected to reflect Gradient is deferred (not present); E-PAR-013/E-PAR-014 references updated to E-PAR-015/E-PAR-016 per pass-9 F-P9-HIGH-002 namespace collision resolution |
