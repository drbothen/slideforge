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
behavioral_contracts: [BC-3.04.001, BC-3.05.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
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

2. **Rich inline formatting layout (BC-3.05.001)**: All 11 `InlineNode` variants
   are represented in `LaidOutSlide.frames` as typed spans with their format
   metadata. The layout stage records the inline type; exporters translate to format-
   specific markup (OMML, HTML tags, PDF text spans).

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-3.04.001 | shape: block declares custom shape with type/position/fill/text/alt | AC-001, AC-002, AC-003, AC-004 |
| BC-3.05.001 | All 11 inline format types render to correct output per format | AC-005, AC-006, AC-007 |

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
        alt: AltText::Explicit("Blue rectangle".into()),
    }),
}
```

### AC-003: Off-canvas shape position produces layout warning (not error)
(traces to BC-3.04.001 EC-002)

A shape with `position x -0.5in y 1.0in` (negative x) produces a
`LayoutWarning::OffCanvas { slide_index, shape_type, x_emu, y_emu }` warning.
This warning is accumulated via `DiagnosticSink` but does NOT halt layout. The
shape frame is still produced at the declared position.

### AC-004: Decorative shape emits empty AltText
(traces to BC-3.04.001 precondition 3)

A `shape:` with `decorative: true` produces `AltText::Decorative` in the
`ShapeFrame`. PPTX exporter maps this to `<p:cNvPr descr="" hidden="1"/>`.
PDF exporter marks the element as an Artifact. HTML exporter uses `alt=""`.

### AC-005: All 11 InlineNode variants represented in layout output
(traces to BC-3.05.001 postcondition — all 11 types produce correct IR)

All 11 `InlineNode` variants are preserved in `FrameContent::TextRun` records
within `LaidOutSlide.frames`. The layout stage does NOT produce format-specific
markup — it only carries the variant type and its attributes:

```rust
pub enum InlineNode {
    Plain(Arc<str>),
    Bold(Arc<str>),
    Italic(Arc<str>),
    Code(Arc<str>),
    Link { text: Arc<str>, url: Arc<str> },
    Math(MathAst),                    // rendered by slideforge-math
    Footnote(Arc<str>),
    Xref { text: Arc<str>, target: Arc<str> },
    Superscript(Arc<str>),
    Subscript(Arc<str>),
    Strikethrough(Arc<str>),
    Highlight(Arc<str>),
}
```

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

During layout, `InlineNode::Xref { target }` references are validated against the
slide titles present in the `Deck`. Unknown xref targets accumulate
`LayoutWarning::XrefTargetNotFound { target, slide_index }`. This is a warning,
not an error, to allow accumulation (DI-018).

## Tasks

- [ ] Add `FrameContent::Shape(ShapeFrame)` and `FrameContent::TextRun(Vec<InlineNode>)` to `Frame` enum in `src/types.rs`
- [ ] Implement `ShapeFrame` struct with `shape_type`, `fill`, `text`, `alt` fields
- [ ] Implement `ShapeType` enum: `Rect`, `Ellipse`, `Arrow`, `Line`, `Star`, `Custom`
- [ ] Implement `FillSpec` enum: `SolidColor(Rgb)`, `Gradient { from: Rgb, to: Rgb }`, `None`
- [ ] Implement `AltText` enum: `Explicit(Arc<str>)`, `Decorative`
- [ ] Implement shape layout pass in `layout::run()`: iterate slide shapes → produce `Frame`s
- [ ] Implement unit conversion: `from_inches`, `from_em` for shape positions
- [ ] Implement off-canvas detection (negative or > page dimensions) → `LayoutWarning`
- [ ] Verify `InlineNode` enum from `slideforge-types` has all 11 variants (if not, add them)
- [ ] Implement xref target validation pass at end of layout run
- [ ] Write unit tests:
  - `0.5in` → `Emu(457_200)`, `-0.5in` → off-canvas warning
  - `decorative: true` → `AltText::Decorative`
  - all 11 inline types survive layout pass unchanged
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
   violation. Layout returns `Err(LayoutError::MissingAlt { slide_index })`. The
   primary enforcement is in `slideforge-validate` (STORY-015).
3. **Integer EMU for all positions (DI-010, ADR-013)**: `em` units are resolved to
   EMU using brand font size at layout time. No `f64` in `BoundingBox`.
4. **InlineNode variants are exhaustive**: The layout stage must handle ALL 11
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
| This story spec | ~2,200 |
| STORY-026 layout types | ~1,500 |
| BC-3.04.001 + BC-3.05.001 files | ~3,500 |
| `slideforge-types` InlineNode definition | ~1,000 |
| Test files to write | ~2,000 |
| **Total** | **~10,200** |

## Test Strategy

- **Unit tests**: EMU conversion for all unit types (in, em); off-canvas warning for
  negative coordinates; decorative shape produces `AltText::Decorative`; all 11
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

Same constraints as STORY-026. No exporter crates.
