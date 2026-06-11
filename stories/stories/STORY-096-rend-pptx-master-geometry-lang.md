---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-096
title: "REND-007: PPTX slideMaster 16:9 geometry + progress_bar layout + lang on runs"
epic: EPIC-08
wave: 5
points: 5
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.0"
created: "2026-06-11"
source_findings: [REND-007]
behavioral_contracts: [BC-4.01.001, BC-4.01.005, BC-5.01.005]
# BC status: BC-4.01.001 (valid .pptx — master 4:3 geometry on a 16:9 deck causes footer
# and date placeholders off-slide; PowerPoint opens with layout warnings). BC-4.01.005
# (all 31 layouts present — progress_bar has no named layout, falls back to generic).
# BC-5.01.005 (lang propagates to PPTX rPr lang attribute — currently dropped).
# All three BCs are authored.
verification_properties: []
nfr_refs: []
closes_findings: [REND-007]
depends_on:
  - STORY-037
  - STORY-038
  - STORY-023
blocks: []
target_module: slideforge-pptx
subsystems: [SS-06]
estimated_days: 2
---

# STORY-096: REND-007 — PPTX Master 16:9 Geometry + progress_bar Layout + Run lang

## Subsystem Anchor Justification

SS-06 (PPTX Export) owns all three defects: slide master geometry, layout presence, and
run-level lang attribute. All fixes are in `slideforge-pptx`. EPIC-08 is the owning epic.

## Dependency Anchor Justifications

- `depends_on: [STORY-037]` — PPTX core serializer; master/layout XML is built here.
- `depends_on: [STORY-038]` — Layout compliance and the `find_layout_index` map that
  drives layout lookup for each slide type.
- `depends_on: [STORY-023]` — Brand synthesis produces the 31 layout hierarchy including
  page size; the master sldSz must match `brand.page_size`.

## Narrative

As a slideforge user, I want the PPTX slide master to use the correct 16:9 geometry so
that footer and date placeholders appear within the slide area, a `progress_bar` slide has
its intended named layout, and all text runs carry the correct language attribute for
spell-check and accessibility.

## Previous Story Intelligence

STORY-037 built the PPTX core serializer. STORY-038 added layout compliance. The master
geometry was hardcoded to 4:3 (`cx="6858000" cy="5143500"`) independently of the deck's
configured page size. The `progress_bar` slide type was added in STORY-087 (Wave 4) but
no named layout was added to the PPTX layout hierarchy for it. Run-level lang has never
been propagated to `<a:rPr lang="..."/>`.

## Architecture Compliance Rules

- Per BC-4.01.001 postcondition 4: .pptx openable in PowerPoint 365 without error
  dialogs. A 4:3 master on a 16:9 deck causes PowerPoint layout warnings.
- Per BC-4.01.005: all 31 slide types must have a corresponding named layout.
- Per BC-5.01.005: lang declaration propagates to `<a:rPr lang="en-US"/>` (or the deck's
  declared lang value) on all PPTX text runs.
- Per ADR-001: no raw XML strings; use ooxmlsdk types.
- The `sldSz` element in `slideMaster1.xml` must match the `sldSz` in `presentation.xml`
  (derived from `brand.page_size`).

## Library & Framework Requirements

- `ooxmlsdk` 0.6.1 — `RunProperties` type carries a `lang` field (confirm field name from
  ooxmlsdk source). `PresentationSlideSize` or equivalent for master sldSz.

## File Structure Requirements

Files to modify:
- `crates/slideforge-pptx/src/master_serializer.rs` (or equivalent) — fix `sldSz` to use
  `brand.page_size` instead of hardcoded 4:3 values.
- `crates/slideforge-pptx/src/slide_serializer.rs` — propagate `lang` from deck metadata
  to `<a:rPr lang="..."/>` on every text run.
- `crates/slideforge-pptx/src/layout_serializer.rs` (or brand-to-layouts module) — add
  named layout for `progress_bar` slide type.
- Test files in `crates/slideforge-pptx/src/tests/`.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,000 |
| `crates/slideforge-pptx/src/master_serializer.rs` | ~2,500 |
| `crates/slideforge-pptx/src/slide_serializer.rs` (run props section) | ~3,000 |
| `crates/slideforge-pptx/src/layout_serializer.rs` | ~2,000 |
| Test files | ~2,000 |
| **Total** | **~11,500** |

## Acceptance Criteria

### AC-001: Slide master sldSz matches deck page size
(traces to BC-4.01.001 postcondition 2)

When the deck uses default 16:9 page size (`cx="9144000" cy="5143500"`), `slideMaster1.xml`
declares `<p:sldSz cx="9144000" cy="5143500"/>` — not the 4:3 values. For custom brand page
sizes, the master sldSz uses the brand's configured width and height.

Verified by: unit test building a 16:9 deck; unzip .pptx; parse `slideMaster1.xml`;
assert sldSz attributes match `presentation.xml` sldSz.

### AC-002: progress_bar slide type has a named layout in the layout hierarchy
(traces to BC-4.01.005 postcondition — all 31 slide types have named layouts)

A deck containing a `progress_bar` slide renders using a named `progress_bar` layout (not
the generic layout 1 fallback). The PPTX layout hierarchy includes a layout named
`progress_bar` that captures the intended visual structure.

Verified by: unit test building a deck with a `progress_bar` slide; unzip .pptx; assert
a layout XML file with `<p:cSld name="progress_bar">` (or equivalent naming convention)
is present and the slide's `r:id` relationship points to it.

### AC-003: All text runs carry lang attribute from deck metadata
(traces to BC-5.01.005 postcondition — lang propagates to PPTX rPr)

Every `<a:rPr>` element in every `slideN.xml` carries a `lang` attribute whose value
matches the deck's declared `lang` field (default `"en-US"` when not specified). Runs with
no explicit lang currently emit no attribute — after this fix they emit `lang="en-US"`.

Verified by: unit test serializing a slide with body text; parse the resulting slide XML;
assert every `<a:rPr>` has `lang` attribute. Second test with an explicit `lang "fr-FR"`
deck declaration; assert `lang="fr-FR"` on runs.

### AC-004: Footer and date placeholders stay within slide bounds in 16:9
(traces to BC-4.01.001 postcondition 4 — openable without error dialogs)

With the master sldSz corrected to 16:9, footer and date placeholder positions inherited
from the master are within the 9144000 × 5143500 bounds. No placeholder is positioned
off-slide.

Verified by: snapshot test of `slideMaster1.xml` comparing placeholder positions against
known-good 16:9 master reference.

## Tasks

- [ ] **T-001 (RED):** Write `test_master_sldSz_matches_deck()` in `slideforge-pptx`.
- [ ] **T-002 (RED):** Write `test_progress_bar_has_named_layout()`.
- [ ] **T-003 (RED):** Write `test_run_has_lang_attribute()`.
- [ ] **T-004 (GREEN):** Replace hardcoded 4:3 sldSz in master serializer with
  `brand.page_size.width_emu` / `brand.page_size.height_emu`.
- [ ] **T-005 (GREEN):** Add `progress_bar` named layout to the layout synthesis path.
- [ ] **T-006 (GREEN):** Propagate deck `lang` to `RunProperties.lang` on all text runs
  in `slide_serializer.rs`.
- [ ] **T-007:** Run `cargo nextest run -p slideforge-pptx --no-fail-fast`.
- [ ] **T-008:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Custom brand page size (4:3 brand) | Master sldSz = brand's configured dimensions |
| EC-002 | Deck with no explicit lang declaration | Default `lang="en-US"` on all runs |
| EC-003 | Slide with no text content | No rPr emitted; no error |
| EC-004 | progress_bar slide with label field | Named layout renders correctly |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-4.01.001 | Serialize LaidOutDeck to Valid .pptx | AC-001, AC-004 |
| BC-4.01.005 | All 31 layouts present | AC-002 |
| BC-5.01.005 | lang propagates to PPTX rPr | AC-003 |

## Test Strategy

TDD strict mode. Three failing tests first. These are mechanical fixes in the PPTX
serializer; no architecture changes required.
