---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-016
lifecycle_status: active
introduced: v1.0.0
modified: ["1.2: Added TextTag::Title → Heading1 routing contract; replaced fragile positional fallback with tag-based routing (STORY-086 — ADR-019 Decision 3.1)"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-4.02.001: Serialize Deck to .docx with Report Register as Body Paragraphs

## Description

The DOCX exporter consumes the `LaidOutDeck` IR and produces a Word-compatible .docx
document. The `report` writing register content becomes narrative body paragraphs in
the document. The `detail` register becomes supplementary sections. PPTX-only content
(visual-only slides with no report/detail register content) produces appropriate
structural markers (slide heading) but no synthetic narrative. This is the primary
mechanism for the "one source, PPTX + DOCX report" value proposition.

Slide headings in the DOCX are derived from `FrameContent::Title(text)` frames in the
`LaidOutDeck`. The pipeline is: Stage 2b sets `TextBlock.tag = TextTag::Title` on the
semantic `ContentBlock::Text`; `layout::run` translates `TextTag::Title` →
`FrameContent::Title(Arc<str>)` in the `LaidOutDeck` frames (populating the frame that
`document_body.rs` already finds at line 146 to emit a `Heading1` paragraph); the DOCX
exporter reads `FrameContent::Title(t)` from `LaidOutDeck.slides[*].frames` (already
implemented in `document_body.rs`; styles.xml already defines `Heading1/2/Normal`) and
does NOT consult `TextBlock.tag` or `ContentBlock` list position directly. This
`FrameContent`-driven routing is the contract the implementer must satisfy; the adversary
will verify that `Heading1` generation is driven by `FrameContent::Title` and not by
`ContentBlock` position.

## Preconditions

1. A valid `LaidOutDeck` IR exists with report/detail register content resolved
   (BC-1.14.002, BC-1.14.003).
2. A Brand struct is available for DOCX styling (fonts, colors, heading styles).
3. The target output directory exists and is writable.
4. `Slide.blocks` has been populated by the Stage 2b threading pass
   (`thread_fields_to_blocks`) per BC-1.16.001, and `layout::run` has translated the
   `TextTag::Title`-tagged `ContentBlock::Text` blocks into `FrameContent::Title(text)`
   frames in the `LaidOutDeck`. The DOCX exporter reads `FrameContent::Title(t)` from
   `LaidOutDeck.slides[*].frames` (via `document_body.rs`) — it does NOT read
   `ContentBlock.tag` directly from the semantic IR.

## Postconditions

1. A valid .docx file is written at the output path.
2. The .docx passes OOXML schema validation (openable in Word 365, LibreOffice Writer).
3. `report` register content from each slide appears as body paragraphs under a
   heading derived from the slide title.
4. `detail` register content appears in supplementary sections after the main body.
5. `notes` register content does NOT appear in the DOCX body (DI-012 register routing).
6. Slides with no report/detail content produce a heading and a blank paragraph
   (no content omitted silently).
7. Inline formatting (bold, italic, code, hyperlinks, etc.) from BC-3.05.001 is
   preserved in DOCX paragraph runs using the correct Word XML run properties.

### TextTag Routing (v1.2 — ADR-019 Decision 3.1)

Layer clarification: `TextTag::Title/Subtitle/Body` on `TextBlock` is the **semantic source**
(lives in the `Deck` / inline IR). The `TextTag→FrameContent` translation is performed by
`layout::run` (in `crates/slideforge-layout/src/layout.rs`): it maps `TextTag::Title` →
`FrameContent::Title(Arc<str>)`, `TextTag::Subtitle` → `FrameContent::Subtitle(Arc<str>)`,
and `TextTag::Body` → `FrameContent::Body(Vec<ContentBlock>)`. The DOCX exporter reads
`FrameContent::Title(t)` from `LaidOutDeck.slides[*].frames` (already implemented in
`document_body.rs`; `styles.xml` already defines `Heading1/2/Normal`) and does NOT consult
`TextBlock.tag` directly. `FrameContent::Title` is the **geometric carrier** that the exporter
consumes.

8. **Title → Heading1:** A slide frame carrying `FrameContent::Title(text)` (populated by
   `layout::run` from a `ContentBlock::Text` with `tag: TextTag::Title`) is serialized as a
   DOCX `Heading1` paragraph. The paragraph element carries
   `<w:pPr><w:pStyle w:val="Heading1"/></w:pPr>` and the title text appears in
   `<w:r><w:t>...</w:t></w:r>` within that paragraph. This routing is determined by the
   `FrameContent::Title` variant on the `LaidOutDeck` frame; the exporter does NOT inspect
   `TextBlock.tag` and does NOT infer routing from frame list position.

9. **Subtitle → Heading2 (when present):** A slide frame carrying
   `FrameContent::Subtitle(text)` (populated by `layout::run` from a `ContentBlock::Text`
   with `tag: TextTag::Subtitle`) is serialized as a DOCX `Heading2` paragraph
   (`<w:pStyle w:val="Heading2"/>`), appearing immediately after the Heading1 paragraph
   for its slide.

10. **Body → Normal paragraph:** A slide frame carrying `FrameContent::Body(blocks)`
    (populated by `layout::run` from a `ContentBlock::Text` with `tag: TextTag::Body`) is
    serialized as a standard body paragraph (`Normal` style or no explicit style), NOT as a
    heading. Body text MUST NOT receive Heading1 styling.

11. **FrameContent-over-position invariant:** The DOCX exporter MUST NOT use the position of a
    frame within `LaidOutSlide.frames` as the primary routing signal for heading style. Routing
    is exclusively driven by the `FrameContent` variant. A `FrameContent::Title` frame that
    appears at list index 1 instead of 0 still routes to `Heading1`. The semantic
    `TextBlock.tag` is consumed exclusively by `layout::run`; by the time the exporter runs,
    the tag has been translated to the appropriate `FrameContent` variant.

## Invariants

1. The `report` register is the primary DOCX body content; `notes` register content
   NEVER appears in the DOCX body. (DI-012)
2. The `detail` register content is included in DOCX and PDF, excluded from PPTX
   (enforced by BC-1.14.003).
3. Two-IR model: the DOCX exporter reads from `LaidOutDeck` only, preserving all
   semantic content. (DI-009)
4. Same .sf source + same brand → same .docx content (determinism, excluding timestamps).
5. **Routing is FrameContent-driven, not position-driven.** `layout::run` translates
   `TextTag::Title` → `FrameContent::Title` and `TextTag::Body` → `FrameContent::Body`;
   the DOCX exporter routes by `FrameContent` variant and MUST NOT use list-index position
   as a routing signal. By the time the exporter runs, the semantic `TextBlock.tag` has
   already been consumed by layout and is not consulted again. (ADR-019 Decision 3.1)
6. **Heading1 and body paragraphs are distinct.** A `TextTag::Title` frame produces a
   Heading1 paragraph. A `TextTag::Body` frame produces a Normal paragraph. These must
   not be confused or merged.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only `notes` content (no report/detail) | DOCX: Heading1 paragraph with slide title (if title tagged frame present) + empty body paragraph; no notes text in body |
| EC-002 | Slide with `report` multi-paragraph prose | Multiple `<w:p>` elements; paragraph breaks preserved |
| EC-003 | report content contains a hyperlink | DOCX hyperlink with correct `w:hyperlink r:id` relationship |
| EC-004 | Deck with 50 slides | .docx has 50 Heading1 sections; all content present; file opens without error |
| EC-005 | report content contains bold inside italic | Correct nested `<w:rPr>` with `<w:b>` and `<w:i>` run properties |
| EC-006 | `ContentBlock::Text` with `tag: TextTag::Title` appears second in frame list | Title text still serialized as Heading1; position within frame list is not consulted |
| EC-007 | Slide has both `TextTag::Title` and `TextTag::Body` frames | Heading1 paragraph with title text, followed by Normal paragraph with body text; they are separate `<w:p>` elements |
| EC-008 | Slide has no `TextTag::Title` frame (no title field in .sf source) | No Heading1 paragraph for that slide; report/detail content begins directly; no crash |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Slide `title "Report Title"` + `report "Analysis."` | DOCX `word/document.xml` has `<w:pStyle w:val="Heading1"/>` paragraph whose `<w:t>` run contains "Report Title"; followed by Normal paragraph containing "Analysis." | happy-path (TextTag→Heading1 routing) |
| 2-slide deck with `report "Analysis follows."` on each slide | .docx has 2 Heading1 sections + 2 Normal body paragraphs with "Analysis follows." | happy-path |
| Slide with `notes "Presenter only"` and no report | .docx heading present (if title frame exists); body paragraph empty; "Presenter only" not in .docx | edge-case |
| Slide with `report "**Bold** and *italic*"` | .docx: paragraph with run `<w:b/>` and run `<w:i/>` | happy-path |
| Same deck built twice | Byte-identical .docx (excluding ZIP timestamps) | edge-case (determinism) |
| Slide with `tag: TextTag::Body` frame only (no title) | No `<w:pStyle w:val="Heading1"/>` element for that slide; body text in Normal paragraph | edge-case (body-not-heading) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | notes register content absent from .docx body text | integration test: unzip .docx, grep body XML for notes content |
| VP-TBD | report register content present in .docx body, one Heading1 section per slide | integration test: parse .docx body XML, count `<w:pStyle w:val="Heading1"/>` elements |
| VP-TBD | .docx passes OOXML schema validation | integration test: validate with docx validator library |
| VP-TBD | Title-tagged ContentBlock serialized to `<w:pStyle w:val="Heading1"/>` paragraph (not Normal style) | integration test: parse document.xml, assert pStyle val (LESSON-13 positive content vector) |
| VP-TBD | Body-tagged ContentBlock does NOT receive Heading1 style | integration test: parse document.xml, assert no Heading1 on body paragraphs |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 ("DOCX Export") per capabilities.md §CAP-016 |
| Capability Anchor Justification | CAP-016 ("DOCX Export") per capabilities.md §CAP-016 — "consuming the report and detail writing registers as narrative body content" is verbatim from CAP-016; TextTag-based Heading1 routing is the mechanism for producing structured document headings from slide titles |
| L2 Domain Invariants | DI-009 (Two-IR model integrity), DI-012 (single source → all formats consistent; register routing) |
| Architecture Module | slideforge-docx crate (filled by architect) |
| Stories | STORY-086 (Stage 2b threading — provides TextTag-tagged ContentBlocks that feed this heading routing); exporter story TBD |

## Related BCs

- BC-1.16.001 — depends on (Stage 2b threading produces TextTag::Title/Body/Subtitle on ContentBlocks; this BC's Heading1 routing requires those tagged frames as upstream input)
- BC-1.14.002 — depends on (report register routing enforced upstream)
- BC-1.14.003 — depends on (detail register routing enforced upstream)
- BC-4.02.002 — composes with (auto-generated document sections are part of DOCX structure)
- BC-3.05.001 — depends on (inline formatting must be correctly mapped to DOCX run properties)

## Architecture Anchors

- `architecture/export-architecture.md` — DOCX serialization design
- `architecture/ir-design.md` — Two-IR model (writing register routing)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
