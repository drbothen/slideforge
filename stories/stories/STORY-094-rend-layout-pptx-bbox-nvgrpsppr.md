---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-094
title: "REND-001/003: Finalize placeholder bbox in layout pass + add nvGrpSpPr to slide spTree"
epic: EPIC-08
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.0"
created: "2026-06-11"
source_findings: [REND-001, REND-003]
behavioral_contracts: [BC-3.06.003, BC-4.01.001]
# BC status: BC-3.06.003 (layout coordinates non-negative and within bounds — violated by
# 0,0 stacking), BC-4.01.001 (serialize LaidOutDeck to valid .pptx — CT_GroupShape schema
# requires nvGrpSpPr as first spTree child). Both BCs are authored; both findings are
# contract violations in merged code.
verification_properties: []
nfr_refs: []
closes_findings: [REND-001, REND-003]
depends_on:
  - STORY-073
  - STORY-037
  - STORY-038
  - STORY-086
blocks: []
target_module: slideforge-layout, slideforge-pptx
subsystems: [SS-05, SS-06]
estimated_days: 3
---

# STORY-094: REND-001/003 — Finalize Placeholder BBox + nvGrpSpPr

## Subsystem Anchor Justification

SS-05 (Layout Engine) owns REND-001's bbox finalization: the bug is in
`slideforge-layout/src/layout.rs:1039-1064` where placeholder bounding-boxes are
never finalized at the STORY-073/STORY-088 seam. SS-06 (PPTX Export) owns REND-003's
`nvGrpSpPr` insertion: the schema violation is in the PPTX slide and notes serializers.
EPIC-08 anchors this story because the most user-visible symptom (bullets stacked at
0,0 in PPTX output) is an export-observable defect gated by EPIC-08.

## Dependency Anchor Justifications

- `depends_on: [STORY-073]` — STORY-073 introduced ContentBlock::Bullets frame
  generation; the bbox finalization bug lives at the STORY-073/STORY-088 seam in the
  layout pass.
- `depends_on: [STORY-037]` — nvGrpSpPr fix is in the PPTX serializer crate built in
  STORY-037.
- `depends_on: [STORY-038]` — layout compliance and placeholder inheritance, same crate.
- `depends_on: [STORY-086]` — Stage 2b threading populates Slide.blocks used by layout.

## Narrative

As a slideforge user, I want bullet content to appear at the correct position on each
slide in PPTX, HTML, and PDF output, so that my presentation is legible and the PPTX
file opens without repair prompts in PowerPoint.

## Previous Story Intelligence

STORY-073 delivered ContentBlock::Bullets → FrameContent::TextRun frame generation.
STORY-088 delivered the bullets list-literal DSL syntax. The seam between these two
stories left the placeholder bbox un-finalized in the layout pass, resulting in all
bullet shapes emitting at position (0,0). STORY-037 established the PPTX serializer
but did not add the mandatory `<p:nvGrpSpPr>` child to `<p:spTree>` in slide and notes
serializers (only master/layout serializers are correct).

## Architecture Compliance Rules

- Per BC-3.06.003: every Frame in every LaidOutSlide must have bbox.x ≥ 0, bbox.y ≥ 0,
  bbox.width > 0, bbox.height > 0, all within page_size bounds. A Frame at (0,0) is an
  InvalidBoundingBox per postcondition 2.
- Per BC-4.01.001: the .pptx must pass OOXML schema validation and open in PowerPoint
  without error dialogs (postcondition 4). CT_GroupShape requires `nvGrpSpPr` as its
  mandatory first child element.
- Per ADR-001 (no raw XML): use ooxmlsdk types for all OOXML element construction.
- Per DI-010: all coordinates are integer EMUs (Emu(i64)); no f64.
- `slideforge-layout` MUST NOT depend on `slideforge-pptx`.

## Library & Framework Requirements

- `ooxmlsdk` 0.6.1 — use `NonVisualGroupShapeProperties`, `NonVisualGroupShapeDrawingProperties`,
  `GroupShapeNonVisual` types (confirm exact type names by reading ooxmlsdk 0.6.1 source).
- `slideforge-layout` — region map constants in `regions.rs`; `BoundingBox` in `types.rs`.

## File Structure Requirements

Files to modify:
- `crates/slideforge-layout/src/layout.rs` — lines 1039-1064: finalize placeholder bbox
  before pushing frame into LaidOutSlide.frames
- `crates/slideforge-pptx/src/slide_serializer.rs` — `serialize_slide()` and
  `serialize_notes_slide()` functions: prepend `NonVisualGroupShapeProperties` as first
  child of each `<p:spTree>`
- `crates/slideforge-layout/src/tests/` — add failing test before fix
- `crates/slideforge-pptx/src/tests/core_tests.rs` — add failing test before fix

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,500 |
| `crates/slideforge-layout/src/layout.rs` (1039-1064 + context) | ~3,000 |
| `crates/slideforge-pptx/src/slide_serializer.rs` (relevant fns) | ~4,000 |
| `crates/slideforge-layout/src/regions.rs` | ~2,000 |
| `crates/slideforge-layout/src/types.rs` | ~1,500 |
| Test files | ~2,000 |
| **Total** | **~15,000** |

Well within 20-30% of agent context window.

## Acceptance Criteria

### AC-001: Bullet shapes have correct bbox in LaidOutDeck
(traces to BC-3.06.003 postcondition 1)

After `layout::run()` completes for any slide containing `FrameContent::TextRun` frames
produced from `ContentBlock::Bullets`, every such frame's `bbox.x > Emu(0)`,
`bbox.y > Emu(0)`, `bbox.width > Emu(0)`, `bbox.height > Emu(0)`, and
`bbox.y + bbox.height <= page_size.height`. The 0,0 stacking defect is eliminated.

Verified by: unit test `test_bullets_bbox_nonzero()` in `slideforge-layout` that builds a
`content` slide with bullets and asserts every frame bbox is non-zero and in bounds.

### AC-002: Duplicate ph idx eliminated from PPTX bullet shapes
(traces to BC-4.01.001 postcondition 2)

A PPTX slide containing a body placeholder does NOT emit two `<p:ph type="body"
idx="1"/>` descriptors on the same slide. The de-duplication is enforced by the layout
or PPTX exporter such that only one body placeholder shape is emitted per slide.

Verified by: unit test that serializes a content slide and asserts `<p:ph type="body"
idx="1"/>` appears exactly once in `slide1.xml`.

### AC-003: nvGrpSpPr present as first child of spTree in slideN.xml
(traces to BC-4.01.001 postcondition 2)

Every `<p:spTree>` in every `slideN.xml` and `notesSlideN.xml` produced by the PPTX
exporter has `<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>`
as its first child element, matching the CT_GroupShape mandatory first-child schema rule.

Verified by: unit test that serializes a minimal deck, unzips the .pptx, parses
`slide1.xml`, and asserts the first child of `<p:spTree>` is `<p:nvGrpSpPr>`.

### AC-004: nvGrpSpPr present in notesSlideN.xml
(traces to BC-4.01.001 postcondition 2)

Every `<p:spTree>` in every `notesSlideN.xml` also has `<p:nvGrpSpPr>` as its first
child. Slides with no notes content still emit the mandatory element.

Verified by: unit test asserting `notesSlide1.xml` spTree first child is nvGrpSpPr.

### AC-005: Regression test — layout produces InvalidBoundingBox error not 0,0 frame
(traces to BC-3.06.003 postcondition 2)

If a region map lookup returns `(x: Emu(-1), ...)` due to a hypothetical bug,
`layout::run()` returns `Err(LayoutError::InvalidBoundingBox { ... })` rather than
silently emitting a frame at negative coordinates.

Verified by: unit test injecting an invalid region map stub.

## Tasks

- [ ] **T-001 (RED):** Write `test_bullets_bbox_nonzero()` in `slideforge-layout` — asserts bullet frames have positive bbox.
- [ ] **T-002 (RED):** Write `test_nvgrpsppr_slide_serializer()` in `slideforge-pptx` — asserts first spTree child is nvGrpSpPr.
- [ ] **T-003 (RED):** Write `test_no_duplicate_ph_body_idx()` in `slideforge-pptx` — asserts body ph appears once.
- [ ] **T-004 (GREEN):** Identify the bbox finalization gap in `layout.rs:1039-1064` for bullet frames; fix by calling the region-map bbox finalizer before pushing frames.
- [ ] **T-005 (GREEN):** Add `nvGrpSpPr` as the first child of `<p:spTree>` in `serialize_slide()` in `slide_serializer.rs`.
- [ ] **T-006 (GREEN):** Add `nvGrpSpPr` as the first child of `<p:spTree>` in `serialize_notes_slide()` in `slide_serializer.rs`.
- [ ] **T-007 (GREEN):** Deduplicate body placeholder emission to fix dup ph idx.
- [ ] **T-008:** Run `cargo nextest run -p slideforge-layout -p slideforge-pptx --no-fail-fast` and confirm all tests pass.
- [ ] **T-009:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no bullets (title-only) | Unaffected; no regression in existing layout behavior |
| EC-002 | Slide with both title and bullets | Title frame and bullets frames all have distinct, non-zero bboxes |
| EC-003 | Notes slide with no content | notesSlide1.xml still has nvGrpSpPr as first spTree child |
| EC-004 | Deck with 31 slide types | All types still produce valid spTree structure |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-3.06.003 | All Positioned Elements Have Valid Non-Negative EMU Coordinates | AC-001, AC-005 |
| BC-4.01.001 | Serialize LaidOutDeck to Valid .pptx | AC-002, AC-003, AC-004 |

## Test Strategy

TDD strict mode. Write failing tests first (T-001 through T-003), then minimal fixes.
No snapshot test update needed for this story — the fix adds structural correctness
without changing the visual content of existing snapshots.
