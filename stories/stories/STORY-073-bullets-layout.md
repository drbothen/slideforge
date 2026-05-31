---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-073
title: "Layout: ContentBlock::Bullets → FrameContent::TextRun frame generation"
epic: EPIC-07
wave: 4
points: 5
priority: P1
tdd_mode: strict
status: ready
crate: slideforge-layout
target_module: slideforge-layout
subsystems: [SS-05]
behavioral_contracts: [BC-3.05.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on: [STORY-028]
blocks: []
estimated_days: 2
---

# STORY-073: Layout — ContentBlock::Bullets → FrameContent::TextRun frame generation

## Subsystem Anchor Justification

SS-05 (Layout Engine) owns this story because `ContentBlock::Bullets` frame
generation is a layout-stage transformation. The layout engine converts semantic
bullet structures from the `Deck` IR into `FrameContent::TextRun` frames within
`LaidOutSlide.frames`, ready for exporter consumption. Validation of inline
content within bullets (xref scanning, depth bounds) is also a layout-stage
concern per BC-3.05.001.

## Dependency Anchor Justifications

- Depends on STORY-028: The `FrameContent::TextRun` variant, `InlineNode`
  handling infrastructure, `run_inline_validation` scaffolding, and the
  `LayoutError`/`LayoutWarning` accumulation pattern are all established in
  STORY-028. This story extends that infrastructure to cover
  `ContentBlock::Bullets`.
- Wave TBD: Wave assignment pending orchestrator dispatch. Likely Wave 3 or 4,
  after STORY-028 merges and the bullets path is needed by exporter stories
  (STORY-037, STORY-041, STORY-046).

## Summary

STORY-028 implements `run_inline_validation` for inline content within `shape:`
blocks and text frames. However, `ContentBlock::Bullets` items also carry
`Vec<InlineNode>` content (one `InlineNode` sequence per `BulletItem`). This
story extends the layout engine to:

1. **Generate `FrameContent::TextRun` frames for `ContentBlock::Bullets`
   blocks**: Each bullet item's inline content is preserved in the layout IR
   as a typed frame sequence. The layout stage does not produce format-specific
   markup — exporters translate to PPTX `<a:p>` runs, DOCX `<w:p>`, PDF text
   spans, or HTML `<li>` elements.

2. **Extend `run_inline_validation` to scan bullet inline content**: Currently
   (post-STORY-028), xref validation and inline depth bounds are checked only
   for shape text and explicit text frames. This story extends the validation
   pass to cover `BulletItem.content` fields across all slides.

3. **Add positive-coverage tests for bullets xref validation and depth bounds**:
   The STORY-028 tests cover the validation functions themselves but use shape
   and text frame fixtures. This story adds explicit bullet-oriented test
   fixtures that confirm the same invariants hold end-to-end through
   `layout::run()`.

## Behavioral Contracts

| BC | Title | Version | Covered ACs |
|----|-------|---------|-------------|
| BC-3.05.001 | All 12 inline format types render to correct output per format | v1.3.4 | AC-001, AC-002, AC-003, AC-INT-1 |

## Acceptance Criteria

### AC-001: ContentBlock::Bullets produces FrameContent::TextRun frames
(traces to BC-3.05.001 postcondition — all 12 inline types produce correct IR)

A slide with `ContentBlock::Bullets(vec![item1, item2, item3])` where each
`BulletItem` has non-empty `Vec<InlineNode>` content produces one
`Frame { content: FrameContent::TextRun(...) }` per bullet item in
`LaidOutSlide.frames`. Source order is preserved: item1 → first frame,
item2 → second frame, etc.

The `FrameContent::TextRun` carries the full `Vec<InlineNode>` sequence
verbatim — no inline processing occurs at layout time.

```rust
// Canonical fixture: 2-item bullet list → 2 frames
let deck = Deck { slides: vec![Slide {
    content_blocks: vec![ContentBlock::Bullets(vec![
        BulletItem { content: vec![InlineNode::Plain(Arc::from("first"))], depth: 0 },
        BulletItem { content: vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from("second"))])], depth: 0 },
    ])],
    ..Default::default()
}], ..Default::default() };
let laid_out = layout::run(&deck, &brand_config)?;
assert_eq!(laid_out.slides[0].frames.len(), 2);
// Both frames are TextRun variants
```

### AC-002: Xref validation scans bullet inline content
(traces to BC-3.05.001 EC-002 — xref validation scope)

When `run_inline_validation` runs (called from `layout::run()`), it traverses
ALL `InlineNode::Xref` occurrences within `ContentBlock::Bullets` items.
Unknown xref targets found within bullet content produce
`LayoutWarning::XrefTargetNotFound { target, source_slide_index }` exactly as
they do in shape text and text frame content (established in STORY-028).

Canonical test vector: a slide with one bullet containing
`InlineNode::Xref(Arc::from("missing-slide"))` where "missing-slide" is not
a title in the `Deck` produces exactly one `LayoutWarning::XrefTargetNotFound`
with `target == "missing-slide"` and the correct `source_slide_index`.

### AC-003: Inline depth bound enforced in bullet content
(traces to BC-3.05.001 invariant 4 — nesting depth bound)

`run_inline_validation` enforces the 64-level nesting depth limit on
`BulletItem.content` trees. A bullet item whose inline tree is nested 65 levels
deep produces `LayoutError::InlineDepthExceeded { source_slide_index, depth: 65 }`.
This is a hard error — the affected `LaidOutSlide` is NOT produced. The error
is accumulated via the standard multi-error accumulator (not bail-on-first).

Canonical test vector: a `BulletItem` with `content:` forming 65 nested
`Bold(vec![Bold(vec![...])])` nodes → `LayoutError::InlineDepthExceeded` with
`depth == 65`.

### AC-INT-1: layout::run end-to-end integration with bullets
(traces to BC-3.05.001 postcondition — full layout pipeline)

`layout::run()` produces correct `LaidOutDeck` output when the input `Deck`
contains `ContentBlock::Bullets` in addition to other content blocks. Verified
via an integration test in `crates/slideforge-layout/tests/` that:

1. Constructs a `Deck` with a slide containing `ContentBlock::Bullets` with at
   least one bullet per common `InlineNode` variant (Plain, Bold, Xref)
2. Calls `layout::run(&deck, &brand_config)`
3. Asserts the `LaidOutDeck` contains `FrameContent::TextRun` frames in
   source order
4. Asserts that a bullet `InlineNode::Xref` with an unknown target produces
   `LayoutWarning::XrefTargetNotFound`
5. Asserts that a well-formed bullet list (all known xref targets, depth ≤ 64)
   produces zero `LayoutError` entries

## Tasks

- [ ] Extend `layout::run()` to iterate `ContentBlock::Bullets` on each slide
      and produce `FrameContent::TextRun` frames (one per `BulletItem`)
- [ ] Extend `run_inline_validation` (from STORY-028) to traverse `BulletItem.content`
      sequences in addition to shape text and text frame content
- [ ] Add bullet fixture to the xref validation unit tests (`xref_unknown_in_bullet`)
- [ ] Add bullet fixture to the depth bound unit tests (`depth_exceeded_in_bullet`)
- [ ] Add `AC-INT-1` integration test in `crates/slideforge-layout/tests/`
- [ ] Verify no exporter crates appear in `slideforge-layout/Cargo.toml` (enforced
      by Forbidden Dependencies rule — same constraint as STORY-028)
- [ ] Update `slideforge-types/src/inline.rs` rustdoc: change "Exactly 11 variants"
      on line ~16-17 to "Exactly 12 variants" and cite this story (STORY-073) as
      the anchor for the bullets-layout scope

## Previous Story Intelligence

STORY-028 establishes:
- `FrameContent::TextRun(Vec<InlineNode>)` variant in the `Frame` enum
- `run_inline_validation` with xref validation and depth bound checking
- `LayoutError::InlineDepthExceeded` and `LayoutWarning::XrefTargetNotFound`
- The multi-error accumulation pattern (`LayoutError::Multiple`)

This story extends those foundations to cover `ContentBlock::Bullets`. The
implementer must ensure the `run_inline_validation` function accepts a unified
traversal over all inline content sources (shapes, text frames, AND bullets)
rather than separate per-source passes.

## Architecture Compliance Rules

1. **No exporter crates in slideforge-layout**: `slideforge-layout` must NOT
   depend on `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, or
   `slideforge-html`. Bullet frame generation is IR-level only.
2. **Integer EMU for bounding boxes (DI-010, ADR-013)**: Any bounding box
   associated with bullet frames uses `Emu(i64)`, not `f64`.
3. **12 InlineNode variants are exhaustive (BC-3.05.001 v1.3.4)**: The layout
   pass handling `BulletItem.content` must handle ALL 12 variants without a
   wildcard catch-all. Missing variants are compile errors.
4. **Xref validation scope includes bullets**: The validation pass must not
   silently skip bullet content (this was the process-gap noted across multiple
   adversarial passes).

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace | `InlineNode`, `ContentBlock::Bullets`, `BulletItem`, `Deck` IR types |
| `thiserror` | `=2.0.18` | `LayoutError` and `LayoutWarning` (already in STORY-028) |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-layout/src/inline.rs` | Modify | Extend `run_inline_validation` to include bullet content traversal |
| `crates/slideforge-layout/src/lib.rs` | Modify | Wire `ContentBlock::Bullets` → `FrameContent::TextRun` in `layout::run()` |
| `crates/slideforge-layout/tests/bullets_layout_integration.rs` | Create | AC-INT-1 integration test |
| `crates/slideforge-types/src/inline.rs` | Modify (rustdoc only) | Update "11 variants" → "12 variants"; cite STORY-073 |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,200 |
| STORY-028 context (inline validation infrastructure) | ~2,000 |
| BC-3.05.001 file | ~2,500 |
| `slideforge-types` InlineNode definition | ~1,000 |
| Test files to write | ~2,500 |
| **Total** | **~11,200** |

## Test Strategy

- **Unit tests**: Bullet list with 3 items → 3 `FrameContent::TextRun` frames
  in source order; bullet `InlineNode::Xref` with unknown target → warning
  accumulated; bullet inline tree at depth 65 → `LayoutError::InlineDepthExceeded`.
- **Integration test**: `layout::run()` end-to-end with a deck containing
  `ContentBlock::Bullets`; asserts frame count, frame types, and absence of
  spurious warnings on well-formed input.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty bullet list (`ContentBlock::Bullets(vec![])`) | Zero `FrameContent::TextRun` frames produced; no error or warning |
| EC-002 | Bullet item with empty content (`BulletItem { content: vec![], depth: 0 }`) | One frame produced with empty inline sequence; no depth error |
| EC-003 | Nested bullet (depth > 0) | Frame produced with depth preserved in `BulletItem.depth`; layout does not flatten |
| EC-004 | Xref inside nested bold inside bullet | Xref validation traverses into container nodes; `XrefTargetNotFound` still accumulated |
| EC-005 | Multiple bullets on same slide with depth violations | All `InlineDepthExceeded` errors accumulated before returning (not bail-on-first) |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-29 | product-owner | Initial story creation — bullets-layout anchor for recurring [process-gap] finding (STORY-028 pass-11 F-P11-MED-002 resolution) |
