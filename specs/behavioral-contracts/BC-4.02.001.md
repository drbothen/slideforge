---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
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

## Preconditions

1. A valid `LaidOutDeck` IR exists with report/detail register content resolved
   (BC-1.14.002, BC-1.14.003).
2. A Brand struct is available for DOCX styling (fonts, colors, heading styles).
3. The target output directory exists and is writable.

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

## Invariants

1. The `report` register is the primary DOCX body content; `notes` register content
   NEVER appears in the DOCX body. (DI-012)
2. The `detail` register content is included in DOCX and PDF, excluded from PPTX
   (enforced by BC-1.14.003).
3. Two-IR model: the DOCX exporter reads from `LaidOutDeck` only, preserving all
   semantic content. (DI-009)
4. Same .sf source + same brand → same .docx content (determinism, excluding timestamps).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only `notes` content (no report/detail) | DOCX: slide heading + empty body paragraph; no notes text in body |
| EC-002 | Slide with `report` multi-paragraph prose | Multiple `<w:p>` elements; paragraph breaks preserved |
| EC-003 | report content contains a hyperlink | DOCX hyperlink with correct `w:hyperlink r:id` relationship |
| EC-004 | Deck with 50 slides | .docx has 50 heading sections; all content present; file opens without error |
| EC-005 | report content contains bold inside italic | Correct nested `<w:rPr>` with `<w:b>` and `<w:i>` run properties |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 2-slide deck with `report "Analysis follows."` on each slide | .docx has 2 headings + 2 body paragraphs with "Analysis follows." | happy-path |
| Slide with `notes "Presenter only"` and no report | .docx heading present; body paragraph empty; "Presenter only" not in .docx | edge-case |
| Slide with `report "**Bold** and *italic*"` | .docx: paragraph with run `<w:b/>` and run `<w:i/>` | happy-path |
| Same deck built twice | Byte-identical .docx (excluding ZIP timestamps) | edge-case (determinism) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | notes register content absent from .docx body text | integration test: unzip .docx, grep body XML for notes content |
| VP-TBD | report register content present in .docx body, one section per slide | integration test: parse .docx body XML, count headings |
| VP-TBD | .docx passes OOXML schema validation | integration test: validate with docx validator library |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 ("DOCX Export") per capabilities.md §CAP-016 |
| Capability Anchor Justification | CAP-016 ("DOCX Export") per capabilities.md §CAP-016 — "consuming the report and detail writing registers as narrative body content" is verbatim from CAP-016 |
| L2 Domain Invariants | DI-009 (Two-IR model integrity), DI-012 (single source → all formats consistent; register routing) |
| Architecture Module | slideforge-docx crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.14.002 — depends on (report register routing enforced upstream)
- BC-1.14.003 — depends on (detail register routing enforced upstream)
- BC-4.02.002 — composes with (auto-generated document sections are part of DOCX structure)
- BC-3.05.001 — depends on (inline formatting must be correctly mapped to DOCX run properties)

## Architecture Anchors

- `architecture/export-architecture.md#docx` — DOCX serialization design
- `architecture/ir-design.md` — Two-IR model (writing register routing)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
