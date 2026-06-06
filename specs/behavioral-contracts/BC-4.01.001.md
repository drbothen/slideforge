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
capability: CAP-015
lifecycle_status: active
introduced: v1.0.0
modified: ["1.2: Added TextTag-based title-placeholder routing contract (STORY-086 — ADR-019 Decision 3.1)"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-4.01.001: Serialize LaidOutDeck to Valid .pptx with Correct Placeholder Inheritance

## Description

The PPTX exporter accepts a `LaidOutDeck` IR and serializes it to a valid OOXML .pptx
file using ooxmlsdk. The output conforms to ECMA-376 schema constraints including
correct placeholder inheritance (slide → layout → master), proper element ordering,
and complete relationship references. The output must be openable in PowerPoint,
Keynote, Google Slides, and LibreOffice without schema errors or missing content.

Title text is routed to the PPTX title placeholder (`<p:ph type="title"/>` or `idx="0"`)
via the `TextTag::Title` field on `ContentBlock::Text` — NOT via a positional fallback
heuristic. This tag-based routing is the load-bearing contract for LESSON-13 positive
content vectors and the Stage 2b threading pass (ADR-019).

## Preconditions

1. A valid `LaidOutDeck` IR exists (produced by the layout stage with no errors).
2. A `Brand` struct is available (from synthesis or loading).
3. The target output directory exists and is writable.
4. `LaidOutDeck` IR types implement `Hash + Eq + Clone` (required by DI-011).
5. All coordinates in `LaidOutDeck` are integer EMUs (required by DI-010).
6. `Slide.blocks` has been populated by the Stage 2b threading pass
   (`thread_fields_to_blocks`) per BC-1.16.001. Specifically, title text arrives as
   `ContentBlock::Text(TextBlock { tag: TextTag::Title, .. })` in the semantic IR,
   and the layout stage has converted it to the appropriate `FrameContent` frame with
   the title placeholder designation (`<p:ph type="title"/>` or `<p:ph idx="0">`).

## Postconditions

1. A valid .pptx file is written to the output path.
2. The .pptx passes OOXML schema validation (structural validity, no schema errors).
3. Placeholder inheritance is correct: slide placeholders inherit from layout by `idx`,
   layout inherits from master by `type`.
4. The .pptx is openable in PowerPoint 365, LibreOffice Impress, and Keynote without
   error dialogs.
5. Slide IDs in the .pptx start at ≥ 256 (ECMA-376 minimum, per Spike S6 BUG-006).
6. Master slide IDs start at ≥ 2^31.
7. `[Content_Types].xml` registers every part type present in the ZIP.
8. All semantic information required by other exporters (DOCX, PDF) is preserved in
   `LaidOutDeck` (DI-009).

### TextTag Routing (v1.2 — ADR-019 Decision 3.1)

9. **Title routing:** A slide frame derived from a `ContentBlock::Text` with
   `tag: TextTag::Title` is serialized as a PPTX title placeholder shape — a `<p:sp>`
   element whose `<p:ph>` child has `type="title"` (or `idx="0"` for body-first layouts).
   The text content appears inside `<p:txBody><a:p><a:r><a:t>...</a:t></a:r></a:p></p:txBody>`
   within that placeholder shape. This routing is determined by the `TextTag::Title` field
   on the originating `ContentBlock::Text`; it is NOT inferred from position (e.g., "first
   text run on the slide").

10. **Subtitle routing:** A slide frame derived from a `ContentBlock::Text` with
    `tag: TextTag::Subtitle` is serialized as the subtitle placeholder shape (`<p:ph type="subTitle"/>`)
    when the layout has a subtitle placeholder, or as a body placeholder otherwise.

11. **Body routing:** A slide frame derived from a `ContentBlock::Text` with
    `tag: TextTag::Body` is serialized as a body placeholder shape (`<p:ph type="body"/>` or
    `idx="1"`) — distinct from the title placeholder. Body text MUST NOT appear in the title
    placeholder shape.

12. **Tag-over-position invariant:** The PPTX exporter MUST NOT use the position of a
    `ContentBlock::Text` within `Slide.blocks` as the primary routing signal. Routing is
    exclusively driven by the `tag` field. A title block that appears second in `Slide.blocks`
    (due to any future reordering) still routes to the title placeholder.

## Invariants

1. Two-IR model integrity: the PPTX exporter reads from `LaidOutDeck` only — it does not modify the `Deck` IR. (DI-009)
2. Integer EMU coordinates only — no f64 in the OOXML output path. (DI-010)
3. All `LaidOutDeck` IR types used in the export path implement `Hash + Eq + Clone`. (DI-011)
4. Same `LaidOutDeck` + same `Brand` produces byte-identical output (determinism, excluding timestamps).
5. **TextTag routing is tag-driven, not position-driven.** Title text always routes to the title
   placeholder shape regardless of where `ContentBlock::Text(Title)` appears in `Slide.blocks`.
   The exporter MUST NOT use list-index position as a routing signal. (ADR-019 Decision 3.1)
6. **Title and body placeholders are distinct.** A title-tagged frame and a body-tagged frame
   MUST NOT share the same `<p:sp>` placeholder shape. The title placeholder carries only title
   text; the body placeholder carries only body/subtitle text. Mixed-content placeholder shapes
   are a schema error.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no visual elements (only notes/metadata) | Valid slide produced; empty content area; speaker notes present |
| EC-002 | Slide with maximum content (31 bullet points) | Valid PPTX; potential E-LAY-001 canvas overflow warning (not a PPTX error) |
| EC-003 | Deck with only 1 slide | Valid 1-slide PPTX; all standard ZIP parts present |
| EC-004 | Chart SVG embedded as media part | Chart present in `/ppt/media/chart1.svg`; relationship reference correct |
| EC-005 | Diagram SVG embedded as media part | SVG has no `foreignObject` (per BC-1.12.003); embedded correctly |
| EC-006 | Output directory does not exist | Directory created; output written. |
| EC-007 | `ContentBlock::Text` with `tag: TextTag::Title` appears second in `Slide.blocks` (hypothetical reorder) | Title text still routes to title placeholder (`<p:ph type="title"/>`); position within blocks list is not consulted |
| EC-008 | Slide has both `TextTag::Title` and `TextTag::Body` frames | Title runs appear in `<p:ph type="title"/>` shape; body runs appear in `<p:ph type="body"/>` or `idx="1"` shape; no content overlap between the two shapes |
| EC-009 | Slide has `TextTag::Subtitle` with no subtitle layout placeholder | Subtitle text routed to the body placeholder instead; no PPTX schema error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| LaidOutDeck with 1 title slide, title text "My Title" | Valid .pptx; slide XML has `<p:ph type="title"/>` shape containing `<a:t>My Title</a:t>`; text run non-empty | happy-path (TextTag title routing) |
| LaidOutDeck with content slide: title "Heading", body "Body text" | PPTX slide has title placeholder with "Heading" and body placeholder with "Body text"; they are in separate `<p:sp>` elements | happy-path (title+body separation) |
| LaidOutDeck with all 31 slide types (fixture) | Valid .pptx; 31 slides; all layouts present; no schema errors | happy-path |
| LaidOutDeck with severity_cards slide | .pptx includes card color fills with correct brand colors | happy-path |
| Same LaidOutDeck built twice | Byte-identical .pptx output (excluding ZIP timestamps) | edge-case (determinism) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All slide IDs ≥ 256 in output .pptx | unit test: parse slide XML, check spPr/sldId attrs |
| VP-TBD | All master IDs ≥ 2^31 in output .pptx | unit test |
| VP-TBD | [Content_Types].xml covers all parts in the ZIP | snapshot test |
| VP-TBD | OOXML schema validation passes (via ooxmlsdk validation) | integration test |
| VP-TBD | Title-tagged ContentBlock serialized to `<p:ph type="title"/>` shape (not body placeholder) | integration test: parse slide XML, assert ph type attribute (LESSON-13 positive content vector) |
| VP-TBD | Body-tagged ContentBlock serialized to a distinct `<p:ph>` shape from title | integration test: parse slide XML, confirm two separate `<p:sp>` elements |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — this BC is the primary serialization contract for CAP-015, covering the core serialize-to-.pptx requirement including TextTag-based placeholder routing |
| L2 Domain Invariants | DI-009 (Two-IR model integrity), DI-010 (integer EMU), DI-011 (Hash+Eq+Clone), DI-012 (single source → all formats consistent) |
| Architecture Module | slideforge-pptx crate (filled by architect) |
| Stories | STORY-086 (Stage 2b threading — provides TextTag-tagged ContentBlocks that feed this routing); exporter story TBD |

## Related BCs

- BC-1.16.001 — depends on (Stage 2b threading produces TextTag::Title/Body/Subtitle on ContentBlocks; this BC's TextTag routing contract requires those tagged blocks as upstream input)
- BC-4.01.002 — depends on (visual parity test for the output of this BC)
- BC-4.01.003 — composes with (speaker notes and master system are part of this export)
- BC-4.01.005 — composes with (slide/master IDs are enforced by this BC)
- BC-4.03.002 — related to (PDF export uses different backend but same LaidOutDeck IR)

## Architecture Anchors

- `architecture/export-architecture.md` — PPTX serialization design
- `architecture/ir-design.md` — Two-IR model (Deck + LaidOutDeck)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
