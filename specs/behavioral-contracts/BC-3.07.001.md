---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: product-owner
timestamp: 2026-06-11T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-05
capability: CAP-010
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.07.001: takeaway Field Renders as Visible Takeaway Bar on Presentation Slides (PPTX, HTML, PDF)

## Description

When a slide declaration includes a `takeaway: "..."` field, the build pipeline must
produce a visually distinct takeaway bar anchored to the bottom of the slide in all
presentation-format outputs (PPTX, HTML, PDF). The bar displays the takeaway text in
a styled, accessible element. DOCX output continues to aggregate `takeaway` values into
the auto-generated `executive_summary` section (per BC-3.02.001) and does NOT render a
per-slide bar — the on-slide bar is a presentation-format concern only.

This contract closes REND-004 (demo-deep-review-2026-06-11): the `takeaway` field was
previously parsed and stored but never threaded through the layout pass as a visual
shape.

## Preconditions

1. The deck is being built to at least one presentation format: `.pptx`, `.pdf`, or
   `.html` (static HTML).
2. At least one slide declaration in the evaluated `Deck` IR contains a
   `ContentBlock::Takeaway(text)` in `slide.blocks` where `text` is non-empty.
3. The layout stage has access to the active `Brand` and page dimensions.
4. The two-IR model (BC-3.06.001) applies: semantic content is in `Deck`; geometric
   placement of the takeaway bar is determined in `LaidOutDeck`.

## Postconditions

1. **Evaluator (SS-02):** Every slide with a `takeaway: "..."` DSL field that evaluates
   to a non-empty string produces a `ContentBlock::Takeaway(Arc<str>)` in
   `Slide.blocks`. The DOCX `executive_summary` aggregation path (BC-3.02.001) remains
   unchanged and parallel — the eval pass routes the takeaway into BOTH paths:
   `ContentBlock::Takeaway` in `Slide.blocks` (for presentation-format layout) and the
   existing `section_data.executive_summary` aggregation (for DOCX).

2. **Layout (SS-05):** `layout::run` maps each `ContentBlock::Takeaway(text)` to a
   `Frame` with `FrameContent::Takeaway(Arc<str>)`. The `Frame.bbox` is placed in the
   **takeaway bar region**: the bottom strip of the slide.

   Canonical geometry (reference: gene-transfusion-assessment §1.5):
   - x: `0.67"` from left edge → `Emu(613_440)`
   - y: `6.1"` from top → `Emu(5_562_840)`
   - width: `8.66"` (slides inset on both sides) → `Emu(7_924_320)`
   - height: `0.5"` → `Emu(457_200)`
   - fill: `#E8F0F8` (light blue per gene-transfusion-assessment §1.5)

   When a takeaway bar is present, the slide's **body region height is compressed**
   from the full body height to the compressed variant, per gene-transfusion-assessment
   §1.5: `BodyHeight::Full (5.0")` → `BodyHeight::Compressed (4.2")`. The layout
   engine encodes two named height constants; the region map selects the compressed
   variant when a `ContentBlock::Takeaway` is present on that slide. This prevents the
   takeaway bar from overlapping the body content area.

3. **PPTX (SS-06):** Each slide with a takeaway frame produces a `<p:sp>` shape with:
   - Position and size derived from the takeaway bar `Frame.bbox` in EMU
   - Fill: `<a:solidFill><a:srgbClr val="E8F0F8"/>` (hex from canonical geometry)
   - Text: the takeaway string, rendered at a legible font size
   - The `<p:cNvPr name="takeaway">` semantic name (accessibility name)
   - The shape does NOT appear on slides without a takeaway frame

4. **HTML (SS-09):** Each slide with a takeaway frame produces a
   `<div class="takeaway-bar" role="note">` element containing the takeaway text.
   The element is positioned at the bottom of the slide canvas consistent with the
   canonical geometry. The element is NOT omitted from slides without a takeaway.
   WCAG axe-core scan passes (no new violations introduced by the bar).

5. **PDF (SS-07):** Each slide with a takeaway frame produces a tagged structure
   element for the takeaway bar. The element is tagged `/P` with the takeaway text as
   accessible text content. The PDF/UA-1 veraPDF CI gate remains clean.

6. **DOCX (SS-08) — unchanged:** DOCX output does NOT render per-slide takeaway bars.
   The existing `executive_summary` aggregation (BC-3.02.001) is the correct DOCX
   treatment. No regression of the DOCX path is introduced by this contract.

7. **Accessibility (all formats):** The takeaway bar text is real text content, NOT a
   bitmap or decoration. The `alt` of the takeaway bar shape MUST be the takeaway text
   itself (because the text IS the accessible description). `decorative: true` is NOT
   applicable — a takeaway bar always conveys information.

8. **Alignment and font-size overrides (gene-transfusion-assessment §1.5, TK-02/TK-03):**
   - `takeaway_align` (promoted to first-class `align:` field on the `takeaway:` block):
     `"left"`, `"center"`, `"right"`. Default: `center`.
   - `takeaway_size` (promoted to first-class `font_size:` field on the `takeaway:` block):
     point-size override. Default: derived from brand body font size.
   Both fields are optional; their absence uses the defaults above.

## Invariants

1. The takeaway bar is produced for ALL slide types that support the `takeaway:` field
   (per gene-transfusion-assessment §1.5: 20 of 23 types; exceptions are `table`, `end`,
   and `key_metrics`). Attempting `takeaway:` on an exception type produces W-VAL-103
   (unknown field). Slide types that support `takeaway:` list it in their `optional_fields()`.

2. No takeaway bar shape is emitted for slides that do NOT have a `ContentBlock::Takeaway`
   in their blocks. The absence of the field produces no visual artifact.

3. The takeaway bar region coordinates satisfy BC-3.06.003 (non-negative EMU, within page
   bounds). The canonical geometry (see postcondition 2) fits within the default 16:9 slide
   dimensions (`9_144_000 × 5_143_500` EMU).

4. The body region compression (`5.0"` → `4.2"`) is ONLY applied when a
   `ContentBlock::Takeaway` is present on a given slide. Slides without a takeaway use
   the full `BodyHeight::Full` region. This is a per-slide layout decision, not a
   per-deck global setting.

5. The DOCX executive_summary aggregation (BC-3.02.001) and the on-slide bar (this BC)
   are independent pipeline paths. A regression in either path does not excuse a failure
   in the other.

6. Empty takeaway string (`takeaway: ""`): W-VAL-103 is emitted (empty string is an
   unknown-valued optional field); no bar is rendered; the field is treated as absent.
   This is identical to the EC-003 behavior in STORY-097.

7. Per CLAUDE.md conventions: `#![forbid(unsafe_code)]`, `Arc<str>` for string content,
   integer EMU for all coordinates, no `.unwrap()` outside tests.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no takeaway field | No takeaway bar emitted in any format; body region uses BodyHeight::Full |
| EC-002 | All slides in deck have takeaway | All slides get bar in PPTX/HTML/PDF; DOCX executive_summary has all entries |
| EC-003 | takeaway: "" (empty string) | W-VAL-103 cosmetic warning; no bar emitted; treated as absent |
| EC-004 | takeaway with align: "left" override | Bar text left-aligned in PPTX/HTML/PDF |
| EC-005 | takeaway on `table` slide type | W-VAL-103 warning (unknown field for table type); no bar emitted |
| EC-006 | takeaway on `end` slide type | W-VAL-103 warning; no bar emitted |
| EC-007 | DOCX-only build (--format docx) | No takeaway bar shape emitted (DOCX is not a bar format); executive_summary section still produced from takeaway data |
| EC-008 | Slide in warn-only mode with takeaway | Bar rendered; build continues |
| EC-009 | Deck with @for loop generating slides all having takeaway | Each generated slide has a bar; executive_summary has all N entries |
| EC-010 | Brand page size other than default 16:9 | Canonical geometry scales proportionally per BC-3.06.003 invariant 3 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide content: title "T" body "B" takeaway "Key insight"` → pptx | `<p:sp>` with `<p:cNvPr name="takeaway">` present at bottom of slide; `<a:srgbClr val="E8F0F8"/>` fill; text "Key insight" | happy-path PPTX |
| Same slide → html | `<div class="takeaway-bar" role="note">Key insight</div>` present in slide HTML | happy-path HTML |
| Same slide → pdf | Tagged `/P` structure element with "Key insight" text in PDF output | happy-path PDF |
| Same slide → docx | No takeaway bar element; executive_summary section has "Key insight" entry | DOCX aggregate path |
| `slide content: title "T" body "B"` (no takeaway) → pptx | No `<p:sp>` with name="takeaway"; body frame uses BodyHeight::Full | absent takeaway |
| `slide content: title "T" body "B" takeaway "K"` + `slide title: title "Title only"` | content slide has bar + compressed body; title slide has no bar; both export cleanly | mixed deck |
| `slide content: title "T" body "B" takeaway "" ` | W-VAL-103 warning; no bar; body uses BodyHeight::Full | empty-string |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Takeaway bar frame in LaidOutDeck has bbox within slide bounds per BC-3.06.003 | unit test: layout a slide with takeaway; assert FrameContent::Takeaway bbox matches canonical geometry |
| VP-TBD | Body region height is compressed when takeaway is present; full when absent | unit test: compare BodyHeight variant for takeaway vs. no-takeaway slide |
| VP-TBD | PPTX sp shape present on takeaway slides; absent on non-takeaway slides | integration test: build mixed deck; parse pptx XML; assert shape presence/absence |
| VP-TBD | HTML takeaway-bar div present on takeaway slides; absent on non-takeaway slides | unit test: parse rendered HTML |
| VP-TBD | DOCX executive_summary path not regressed by takeaway bar change | regression test: run STORY-027/042 test suite; all pass |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — "Each type enforces its own required fields and layout rules via the SlideType plugin trait." The takeaway bar is an optional layout feature declared in 20 of 31 slide types' optional_fields(); this BC defines the precise rendering rule the SlideType plugin produces when the takeaway field is present. |
| L2 Domain Invariants | DI-001 (alt text required on visual elements — takeaway bar text is its own alt), DI-010 (integer EMU for all coordinates), DI-012 (single .sf source produces all formats consistently) |
| Source | gene-transfusion-assessment.md §1.5 "Takeaway Bar" — geometry, color, body-height compression rule, and align/size override semantics |
| Architecture Modules | SS-02 (Evaluator — ContentBlock::Takeaway), SS-05 (Layout Engine — takeaway bar region), SS-06 (PPTX — sp shape), SS-07 (PDF — tagged structure), SS-09 (HTML — div element) |
| Stories | STORY-097 |

## Related BCs

- BC-3.06.003 — composes with (takeaway bar frame must satisfy coordinate validity invariants)
- BC-3.02.001 — parallel to (DOCX executive_summary aggregation is the DOCX equivalent; independent path)
- BC-4.03.001 — downstream of (PDF/UA-1 compliance applies to the tagged takeaway bar structure element)
- BC-4.03.003 — downstream of (HTML WCAG AA compliance applies to the takeaway-bar div)
- BC-5.01.001 — depends on (alt text requirement applies to the takeaway bar shape; text = alt)

## Architecture Anchors

- `architecture/module-decomposition.md` — SS-05 Layout Engine takeaway bar region
- gene-transfusion-assessment.md §1.5 — canonical geometry and color

## Story Anchor

STORY-097

## VP Anchors

(filled after VP creation)
