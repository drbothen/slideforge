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
capability: CAP-029
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

# BC-1.14.002: report Register Routes to DOCX Body; Excluded from PPTX Slide Content

## Description

The `report` writing register is the reader-facing formal prose register. Content declared
in a `report` field or `report:` block is routed to the DOCX report body paragraphs and
to the PDF main text flow. The report register content does NOT appear as visual content
on PPTX slides, and is NOT rendered on the web preview slide canvas. This is the register
that enables "one source, two audiences" — the same .sf file produces a visual presentation
and a formal written report.

## Preconditions

1. A .sf source contains at least one slide with a `report "..."` field or `report:` block.
2. The deck is built to one or more output formats.

## Postconditions

1. DOCX output: report content appears as body paragraphs in the report document, associated with the corresponding slide position.
2. PDF output: report content appears in the PDF main text body flow (as running prose, not as slide content).
3. PPTX output: report content does NOT appear in any slide shape or placeholder. The PPTX slide body contains only the slide's visual content fields.
4. HTML static output: report content appears in a designated section of the HTML document (not in the slide canvas).
5. Web preview: report content is NOT rendered on the slide canvas.
6. Build exits with code 0 for valid source.

## Invariants

1. Report register routing is determined at the Evaluate stage.
2. No report content appears in PPTX slide shapes or the web preview canvas.
3. Report register participates in DI-012.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with ONLY `report` register content and no visual fields | PPTX: slide may be empty/placeholder (no visual body); DOCX: report body paragraphs present |
| EC-002 | `report` block with multiple paragraphs | Each paragraph rendered as a DOCX body paragraph; correct paragraph styles applied |
| EC-003 | `report` content inside an `@for` generated slide | Each iteration's report content produces separate DOCX body paragraphs in order |
| EC-004 | `report` content with inline links `[text](url)` | Rendered as hyperlink in DOCX; rendered as `<a href>` in HTML |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide content:` + `title "Q1"` + `report "Detailed narrative..."` built to DOCX | DOCX: body paragraph with "Detailed narrative..."; exit 0 | happy-path |
| Same deck built to PPTX | PPTX: slide has no report text in any shape; exit 0 | happy-path |
| Same deck built to HTML | HTML: report text in dedicated section, not slide canvas | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | DOCX body contains report text; PPTX contains no report text | integration test: build to both formats; scan XML for report content location |
| VP-TBD | PPTX slide XML contains no report register text nodes | integration test: parse PPTX slide XML; grep for report content |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 ("Writing Register Support") per capabilities.md §CAP-029 |
| Capability Anchor Justification | CAP-029 ("Writing Register Support") per capabilities.md §CAP-029 — "report (reader-facing formal prose): routes to DOCX body paragraphs, excluded from PPTX visual content" is CAP-029's specification of this register |
| L2 Domain Invariants | DI-012 (single .sf source produces all formats consistently) |
| Architecture Module | slideforge-eval + slideforge-docx crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.14.001 — related to (notes register routing; complementary)
- BC-1.14.003 — related to (detail register routing; complementary)
- BC-1.14.004 — composes with (no register bleeds to wrong format)
- BC-4.02.001 — depends on (DOCX exporter consumes report register content)

## Architecture Anchors

- `architecture/system-overview.md` — register tag routing in pipeline stages

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
