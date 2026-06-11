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
behavioral_contracts: [BC-3.07.001]
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

## Behavioral Contracts

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-3.07.001 | takeaway Field Renders as Visible Takeaway Bar on Presentation Slides (PPTX, HTML, PDF) | AC-001 through AC-006 |
| BC-3.06.003 | All Positioned Elements Valid EMU Coords | AC-002 |
| BC-4.03.001 | PDF/UA-1 compliant | AC-005 |
| BC-4.03.003 | HTML WCAG AA | AC-004 |

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

- `takeaway` must be threaded through as a new `ContentBlock::Takeaway(Arc<str>)` variant.
  The IR design (ADR-019 / Two-IR model) requires semantic info in `Deck` and geometric
  info in `LaidOutDeck`.
- The takeaway bar is a presentation-format concern only (PPTX, HTML, PDF). DOCX behavior
  (executive_summary aggregation via BC-3.02.001) is unchanged.
- Per BC-3.07.001 postcondition 7 and CLAUDE.md DI-001: the takeaway bar text IS the
  accessible alt/description. `decorative: true` is not applicable — the bar always
  conveys information. The PPTX shape carries `<p:cNvPr name="takeaway">`.
- **Canonical takeaway bar geometry (BC-3.07.001 postcondition 2, gene-transfusion-assessment §1.5):**
  - x: `0.67"` → `Emu(613_440)`, y: `6.1"` → `Emu(5_562_840)`
  - width: `8.66"` → `Emu(7_924_320)`, height: `0.5"` → `Emu(457_200)`
  - fill: `#E8F0F8` (light blue, `<a:srgbClr val="E8F0F8"/>`)
- **Body region compression (BC-3.07.001 postcondition 2 + Invariant 4):** when a
  `ContentBlock::Takeaway` is present on a slide, `BodyHeight::Full (5.0")` is replaced
  by `BodyHeight::Compressed (4.2")`. Slides WITHOUT a takeaway continue to use
  `BodyHeight::Full`. This is a per-slide decision, not a per-deck global setting.
- Per BC-3.06.003: the takeaway bar bbox must be non-negative and within page bounds.
  The canonical geometry fits within the default 16:9 dimensions (`9_144_000 × 5_143_500` EMU).
- Per BC-3.07.001 Invariant 1: takeaway is supported on 20 of 31 slide types (exceptions:
  `table`, `end`, `key_metrics`). `takeaway:` on an exception type produces W-VAL-103.
- Per BC-3.07.001 postcondition 8: `takeaway_align` (default `center`) and
  `takeaway_size` (default: brand body font size) are optional first-class fields.
- Per BC-3.07.001 Invariant 6: empty `takeaway: ""` emits W-VAL-103 cosmetic warning;
  no bar is rendered.

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
(traces to BC-3.07.001 postcondition 1 — evaluator produces ContentBlock::Takeaway for non-empty takeaway field)

When a slide declaration contains `takeaway: "Key insight here"`, the evaluator produces
a `ContentBlock::Takeaway("Key insight here")` in `Slide.blocks`. The eval pass routes
takeaway into BOTH paths: `ContentBlock::Takeaway` in `Slide.blocks` (for presentation
layout) and the existing `section_data.executive_summary` aggregation (for DOCX, BC-3.02.001).
The DOCX executive_summary aggregation (STORY-027/042) is not regressed.

Verified by: unit test in `slideforge-eval` asserting `ContentBlock::Takeaway` is present
in evaluated slide blocks when `takeaway:` field is set.

### AC-002: Takeaway bar frame produced by layout pass with canonical geometry
(traces to BC-3.07.001 postcondition 2 + BC-3.06.003 invariant 1 — takeaway bar Frame.bbox at canonical position)

`layout::run()` produces a `FrameContent::Takeaway(text)` frame for each slide containing
a takeaway. The `Frame.bbox` matches canonical geometry: x=`Emu(613_440)`, y=`Emu(5_562_840)`,
width=`Emu(7_924_320)`, height=`Emu(457_200)`. The body region for the same slide uses
`BodyHeight::Compressed (4.2")` instead of `BodyHeight::Full (5.0")`. Slides without a
takeaway continue to use `BodyHeight::Full`. The takeaway bar bbox satisfies BC-3.06.003
(non-negative, within page bounds).

Verified by: unit test building a content slide with takeaway; assert `FrameContent::Takeaway`
frame bbox matches canonical geometry exactly; assert `BodyHeight::Compressed` on that slide
and `BodyHeight::Full` on a slide without takeaway.

### AC-003: PPTX emits takeaway bar as visible shape on each slide
(traces to BC-3.07.001 postcondition 3 — PPTX sp shape with canonical fill, name, and text)

A deck built with `slideforge build --format pptx` and one or more slides with `takeaway:`
produces a .pptx where each such slide contains a `<p:sp>` shape with:
- `<p:cNvPr name="takeaway">` (semantic accessibility name)
- `<a:solidFill><a:srgbClr val="E8F0F8"/>` (canonical fill color)
- The takeaway string as text content
The shape does NOT appear on slides without a `takeaway:` field (BC-3.07.001 Invariant 2).

Verified by: integration test building a mixed deck (with and without takeaway); parse
PPTX XML; assert shape with `name="takeaway"` present on takeaway slides, absent on others.

### AC-004: HTML emits takeaway bar as accessible element
(traces to BC-3.07.001 postcondition 4 + BC-4.03.003 — HTML WCAG AA)

In static HTML output, each slide with a `takeaway:` value has a
`<div class="takeaway-bar" role="note">` element containing the takeaway text. The element
is positioned at the bottom of the slide canvas consistent with canonical geometry.
WCAG axe-core scan passes (no new violations introduced by the bar).

Verified by: unit test in `slideforge-html` asserting `class="takeaway-bar" role="note"`
element in rendered HTML for takeaway slides.

### AC-005: PDF emits takeaway bar with correct structure tag
(traces to BC-3.07.001 postcondition 5 + BC-4.03.001 — PDF/UA-1)

In PDF output, each slide with a `takeaway:` value has a tagged `/P` structure element
for the takeaway bar with the takeaway text as accessible text content. The veraPDF
CI gate remains clean.

Verified by: unit test asserting `/P` tagged structure element for takeaway bar in PDF
output; veraPDF gate unbroken.

### AC-006: DOCX executive_summary aggregation not regressed
(traces to BC-3.07.001 postcondition 6 + BC-3.02.001 — DOCX aggregate path independent)

The DOCX output continues to produce the executive_summary section aggregating all
`takeaway` values. This behavior is unchanged (BC-3.07.001 Invariant 5). The on-slide
bar and the DOCX aggregate path are independent pipelines; a regression in either does
not excuse failure in the other.

Verified by: run existing STORY-027/042 regression tests; confirm all pass.

### AC-007: Empty takeaway and exception slide types handled correctly
(traces to BC-3.07.001 Invariants 1 and 6 — empty-string W-VAL-103; exception types W-VAL-103)

`takeaway: ""` emits W-VAL-103 cosmetic warning (no bar rendered, body uses Full height).
`takeaway:` on a `table`, `end`, or `key_metrics` slide type emits W-VAL-103 (unknown
field for that type, no bar rendered).

Verified by: unit tests for both cases; assert W-VAL-103 emitted, no takeaway bar frame
in LaidOutDeck for either case.

## Tasks

- [ ] **T-001:** Read STORY-027, STORY-042, BC-3.02.001, and BC-3.07.001 before touching
  any code.
- [ ] **T-002 (RED):** Write `test_takeaway_in_content_blocks()` in `slideforge-eval`;
  assert `ContentBlock::Takeaway` present and DOCX path not regressed.
- [ ] **T-003 (RED):** Write `test_takeaway_frame_canonical_geometry()` in `slideforge-layout`;
  assert bbox = `(613_440, 5_562_840, 7_924_320, 457_200)` EMU and `BodyHeight::Compressed`
  on the takeaway slide; `BodyHeight::Full` on a non-takeaway slide.
- [ ] **T-004 (RED):** Write `test_pptx_has_takeaway_bar_shape()` in `slideforge-pptx`;
  assert `<p:cNvPr name="takeaway">` and `srgbClr val="E8F0F8"` present; absent on
  non-takeaway slides.
- [ ] **T-005 (RED):** Write `test_html_has_takeaway_bar()` in `slideforge-html`; assert
  `class="takeaway-bar" role="note"` element present.
- [ ] **T-006 (RED):** Write `test_pdf_has_takeaway_structure_element()` in `slideforge-pdf`.
- [ ] **T-007 (RED):** Write `test_empty_takeaway_emits_w_val_103_no_bar()` and
  `test_takeaway_on_exception_type_emits_w_val_103()`.
- [ ] **T-008 (GREEN):** Add `ContentBlock::Takeaway` and `FrameContent::Takeaway` variants.
- [ ] **T-009 (GREEN):** Route `takeaway` field in eval pass (both paths: ContentBlock +
  executive_summary).
- [ ] **T-010 (GREEN):** Add `TAKEAWAY_BAR_REGION` (canonical geometry) to region maps;
  select `BodyHeight::Compressed` when `ContentBlock::Takeaway` present.
- [ ] **T-011 (GREEN):** Emit takeaway bar `<p:sp>` shape in PPTX serializer with canonical
  fill, semantic name, and text.
- [ ] **T-012 (GREEN):** Emit `<div class="takeaway-bar" role="note">` in HTML renderer.
- [ ] **T-013 (GREEN):** Emit tagged `/P` structure element in PDF exporter.
- [ ] **T-014:** Run `cargo nextest run -p slideforge-eval -p slideforge-layout -p slideforge-pptx -p slideforge-html -p slideforge-pdf --no-fail-fast`.
- [ ] **T-015:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no takeaway field | No takeaway bar shape emitted; no error |
| EC-002 | All slides in deck have takeaway | All slides get bar; DOCX executive_summary has all entries |
| EC-003 | takeaway field is empty string | Empty bar not emitted (same as absent field); W-VAL-103 cosmetic warning |
| EC-004 | Deck with only takeaway slides | Each slide has bar; no crashes |

## Test Strategy

TDD strict mode. Seven failing tests across five crates (AC-001 through AC-007). This is
the largest story in the rendering-fix wave (5 crates, 5 days estimated). If context is
tight during implementation, split as described in Token Budget section above.

Key geometry invariant for T-003: `bbox = Emu { x: 613_440, y: 5_562_840, w: 7_924_320, h: 457_200 }`.
The layout test must assert these exact values from BC-3.07.001 postcondition 2, not
approximate or derived values. Tests for AC-007 (empty string + exception types) must
assert that no `FrameContent::Takeaway` appears in LaidOutDeck and W-VAL-103 is emitted.
