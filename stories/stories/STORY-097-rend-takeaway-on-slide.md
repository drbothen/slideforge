---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-097
title: "REND-004: takeaway field renders as visible bar on PPTX/HTML/PDF slides"
epic: EPIC-07
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.0"
created: "2026-06-11"
source_findings: [REND-004]
behavioral_contracts: []
# BC status: pending PO authorship — flag raised.
# The takeaway field currently only aggregates into DOCX executive_summary via STORY-027/042.
# The spec mandates a visible takeaway bar per slide in all presentation formats (PPTX,
# HTML, PDF). No existing BC covers this on-slide rendering path. Product-owner must author
# a new BC in the Authoring or Layout bounded context (suggest BC-3.07.001 or extend
# BC-3.06 / BC-3.01).
# Routing crates: slideforge-eval, slideforge-layout, slideforge-pptx, slideforge-html,
# slideforge-pdf.
verification_properties: []
nfr_refs: []
closes_findings: [REND-004]
depends_on:
  - STORY-073
  - STORY-037
  - STORY-046
  - STORY-043
  - STORY-086
blocks: []
target_module: slideforge-eval, slideforge-layout, slideforge-pptx, slideforge-html, slideforge-pdf
subsystems: [SS-02, SS-05, SS-06, SS-07, SS-09]
estimated_days: 5
---

# STORY-097: REND-004 — takeaway Field Renders as Visible Bar on All Presentation Formats

## BC Gap Notice

No existing behavioral contract covers on-slide takeaway rendering. The PO must author
a new BC (suggest BC-3.07.001 in a new §3.07 "Slide Takeaway Bar" section, or extend
BC-3.01 or BC-3.04). Until that BC is authored and this story is updated with the BC ID,
status remains `draft` and this story MUST NOT be dispatched to implementation.

PO action required: author BC specifying:
- `takeaway` field in a slide declaration produces a visible horizontal bar anchored
  to the bottom of the slide in PPTX, HTML, and PDF output.
- The bar displays the takeaway text in a visually distinguished style.
- DOCX: takeaway continues to aggregate into executive_summary (existing behavior, no change).

## Subsystem Anchor Justification

This story touches 5 subsystems because the `takeaway` field must be threaded through
the full pipeline: SS-02 (eval: recognize `takeaway` as a slide-level FieldValue),
SS-05 (layout: allocate a takeaway bar region in slide layouts that accept it), SS-06
(PPTX exporter: emit takeaway bar shape), SS-07 (PDF exporter: emit takeaway bar),
SS-09 (HTML exporter: emit takeaway bar). EPIC-07 anchors as the layout epic.

## Dependency Anchor Justifications

- `depends_on: [STORY-073]` — layout frame generation; takeaway bar is a new frame type
  added to the layout pass.
- `depends_on: [STORY-037]` — PPTX core serializer for emitting the bar shape.
- `depends_on: [STORY-046]` — HTML exporter (static HTML); takeaway bar emitted here.
- `depends_on: [STORY-043]` — PDF core; takeaway bar tagged correctly.
- `depends_on: [STORY-086]` — Stage 2b threading must route `takeaway` FieldValue to the
  appropriate ContentBlock so layout can process it.

## Narrative

As a presenter, I want each slide that has a `takeaway: "..."` field to show a visible
takeaway bar at the bottom of the slide in PPTX, HTML, and PDF output, so that the key
point of every slide is immediately visible to my audience.

## Previous Story Intelligence

STORY-027 and STORY-042 implemented takeaway aggregation into the DOCX
`executive_summary` section — this is correct and must not be regressed. The on-slide
rendering was never implemented: the `takeaway` field is parsed and stored but never
threaded into the layout pass as a visual shape.

## Architecture Compliance Rules

- `takeaway` must be threaded through as a new `ContentBlock::Takeaway(Arc<str>)` variant
  or via an existing mechanism such as a custom frame. The IR design (ADR-019 / Two-IR
  model) requires semantic info in `Deck` and geometric info in `LaidOutDeck`.
- The takeaway bar is a presentation-format concern only (PPTX, HTML, PDF). DOCX behavior
  (executive_summary aggregation) is unchanged.
- Per CLAUDE.md: `alt` required on the takeaway bar shape; if it is a visual element with
  no explicit alt, use the takeaway text itself as the accessible alt/description.
- Per BC-3.06.003: the takeaway bar's bbox must be within slide bounds (non-negative,
  within page_size). The canonical position is a bar at the bottom of the slide (e.g.,
  occupying the bottom 8-10% of slide height).
- The bar must NOT overlap the slide content area — region maps must reserve space.

## Library & Framework Requirements

- `slideforge-types` — add `ContentBlock::Takeaway(Arc<str>)` variant (or confirm it
  already exists under a different name).
- `ooxmlsdk` 0.6.1 for PPTX shape emission.
- All crate versions from `Cargo.lock`.

## File Structure Requirements

Files to create / modify:
- `crates/slideforge-types/src/ir.rs` — add `ContentBlock::Takeaway` variant (or
  verify correct existing variant).
- `crates/slideforge-eval/src/eval.rs` — route `takeaway` field into `ContentBlock::Takeaway`.
- `crates/slideforge-layout/src/layout.rs` — map `ContentBlock::Takeaway` to
  `FrameContent::Takeaway(Arc<str>)` with bbox from takeaway-bar region.
- `crates/slideforge-layout/src/regions.rs` — define `TAKEAWAY_BAR_REGION` for slides
  that support the `takeaway` field (all 31 slide types should reserve bottom-bar space).
- `crates/slideforge-pptx/src/slide_serializer.rs` — emit takeaway bar as `<p:sp>` shape.
- `crates/slideforge-html/src/render.rs` — emit `<div class="takeaway-bar">` element.
- `crates/slideforge-pdf/src/exporter.rs` — emit tagged takeaway bar shape.
- Tests for each crate.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,500 |
| `slideforge-types/src/ir.rs` | ~2,000 |
| `slideforge-eval/src/eval.rs` | ~3,000 |
| `slideforge-layout/src/layout.rs` + `regions.rs` | ~4,000 |
| `slideforge-pptx/src/slide_serializer.rs` (new fn) | ~2,000 |
| `slideforge-html/src/render.rs` | ~1,500 |
| `slideforge-pdf/src/exporter.rs` | ~2,000 |
| Test files | ~3,000 |
| **Total** | **~20,000** |

Approaching the 20-30% window; dispatching agent should confirm context budget before
starting. If context is tight, split into two sub-stories: (A) layout + PPTX, (B) HTML + PDF.

## Acceptance Criteria

### AC-001: takeaway FieldValue routed to ContentBlock in eval pass
(pending BC — traces to takeaway field threading contract TBD)

When a slide declaration contains `takeaway: "Key insight here"`, the evaluator produces
a `ContentBlock::Takeaway("Key insight here")` in `Slide.blocks`. The existing DOCX
executive_summary aggregation (STORY-027/042) is not regressed.

Verified by: unit test in `slideforge-eval` asserting `ContentBlock::Takeaway` is present
in evaluated slide blocks when `takeaway:` field is set.

### AC-002: Takeaway bar frame produced by layout pass with valid bbox
(pending BC — traces to layout bbox contract; partially covered by BC-3.06.003)

`layout::run()` produces a `FrameContent::Takeaway(text)` frame for each slide containing
a takeaway, positioned in the bottom bar region of the slide. The frame's bbox is valid
per BC-3.06.003 (non-negative, within page_size bounds), and does not overlap the slide's
main content region.

Verified by: unit test building a content slide with takeaway; assert `FrameContent::Takeaway`
frame has `bbox.y >= page_size.height * 0.85` (bottom 15% of slide) and bbox within bounds.

### AC-003: PPTX emits takeaway bar as visible shape on each slide
(pending BC — traces to PPTX serialization contract)

A deck built with `slideforge build --format pptx` and one or more slides with `takeaway:`
produces a .pptx where each such slide contains a `<p:sp>` shape at the bottom of the
slide with the takeaway text. The shape does not appear on slides without a `takeaway:`
field.

Verified by: integration test building a mixed deck (with and without takeaway); assert
the takeaway-bar shape present on expected slides, absent on others.

### AC-004: HTML emits takeaway bar as accessible element
(pending BC — traces to HTML export + WCAG AA contract BC-4.03.003)

In static HTML output, each slide with a `takeaway:` value has a
`<div class="takeaway-bar" role="note">` element containing the takeaway text. The element
is positioned at the bottom of the slide canvas. WCAG axe-core scan passes (no new violations).

Verified by: unit test in `slideforge-html` asserting `class="takeaway-bar"` element in
rendered HTML for takeaway slides.

### AC-005: PDF emits takeaway bar with correct structure tag
(pending BC — traces to PDF/UA-1 contract BC-4.03.001)

In PDF output, each slide with a `takeaway:` value has a tagged structure element for
the takeaway bar. The element is tagged `/P` or `/Figure` (as appropriate per PDF/UA-1)
with accessible text. veraPDF CI gate remains clean.

Verified by: unit test asserting takeaway bar structure element in PDF output.

### AC-006: DOCX executive_summary aggregation not regressed
(pending BC — existing behavior from STORY-027/042)

The DOCX output continues to produce the executive_summary section aggregating all
`takeaway` values. This behavior is unchanged.

Verified by: run existing STORY-027/042 regression tests; confirm all pass.

## Tasks

- [ ] **T-001:** Read the STORY-027, STORY-042, and BC-3.02.001 specs to understand the
  existing takeaway DOCX path before touching any code.
- [ ] **T-002 (RED):** Write `test_takeaway_in_content_blocks()` in `slideforge-eval`.
- [ ] **T-003 (RED):** Write `test_takeaway_frame_bbox_in_bottom_region()` in `slideforge-layout`.
- [ ] **T-004 (RED):** Write `test_pptx_has_takeaway_bar_shape()` in `slideforge-pptx`.
- [ ] **T-005 (RED):** Write `test_html_has_takeaway_bar()` in `slideforge-html`.
- [ ] **T-006 (RED):** Write `test_pdf_has_takeaway_structure_element()` in `slideforge-pdf`.
- [ ] **T-007 (GREEN):** Add `ContentBlock::Takeaway` and `FrameContent::Takeaway` variants.
- [ ] **T-008 (GREEN):** Route `takeaway` field in eval pass.
- [ ] **T-009 (GREEN):** Add `TAKEAWAY_BAR_REGION` to region maps; map in layout pass.
- [ ] **T-010 (GREEN):** Emit takeaway bar shape in PPTX serializer.
- [ ] **T-011 (GREEN):** Emit takeaway bar element in HTML renderer.
- [ ] **T-012 (GREEN):** Emit takeaway bar in PDF exporter.
- [ ] **T-013:** Run `cargo nextest run -p slideforge-eval -p slideforge-layout -p slideforge-pptx -p slideforge-html -p slideforge-pdf --no-fail-fast`.
- [ ] **T-014:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no takeaway field | No takeaway bar shape emitted; no error |
| EC-002 | All slides in deck have takeaway | All slides get bar; DOCX executive_summary has all entries |
| EC-003 | takeaway field is empty string | Empty bar not emitted (same as absent field); W-VAL-103 cosmetic warning |
| EC-004 | Deck with only takeaway slides | Each slide has bar; no crashes |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| (pending PO authorship) | On-slide takeaway bar rendering | AC-001 through AC-005 |
| BC-3.06.003 | All Positioned Elements Valid EMU Coords | AC-002 |
| BC-4.03.001 | PDF/UA-1 compliant | AC-005 |
| BC-4.03.003 | HTML WCAG AA | AC-004 |

## Test Strategy

TDD strict mode. Five failing tests across five crates. This is the largest story in the
rendering-fix wave (5 crates, 5 days estimated). If context is tight during implementation,
split as described in Token Budget section above.
