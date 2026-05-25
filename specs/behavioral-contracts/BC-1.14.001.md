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

# BC-1.14.001: notes Register Routes to Presenter Notes in PPTX/DOCX/HTML Only

## Description

The `notes` writing register is the presenter-facing register. Content declared in a
`notes` field or `notes:` block is routed exclusively to presenter notes sections in
output formats that have such a concept: PPTX speaker notes (`<p:notes>`), DOCX
presenter notes section, and HTML `<aside data-notes>`. The notes register is excluded
from the PDF main text flow and the web preview slide canvas.

## Preconditions

1. A .sf source contains at least one slide with a `notes "..."` field or `notes:` block.
2. The deck is built to one or more output formats.

## Postconditions

1. PPTX output: notes content appears in the `<p:notes>` element for that slide. It does NOT appear in the slide body shapes.
2. DOCX output: notes content appears in a designated presenter notes section. It does NOT appear in the DOCX report body (that is the `report` register's domain).
3. Static HTML output: notes content appears in `<aside data-notes>` or equivalent hidden/presenter element. It does NOT appear in the main slide canvas.
4. Web preview: notes content is NOT rendered on the slide canvas. It may be shown in a dedicated notes panel if the preview UI provides one.
5. PDF output: notes content does NOT appear in the PDF main text flow (detail-only formats per BC-1.14.004).
6. Build exits with code 0 for valid source.

## Invariants

1. Notes register content routing is determined at the Evaluate stage — the `LaidOutSlide` carries notes content tagged by register.
2. No notes content appears in PPTX slide shapes or DOCX report body paragraphs.
3. Notes register participates in DI-012: single .sf source → consistent content across formats.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only `notes` field (no visual content) | PPTX: empty slide body with speaker notes; DOCX: empty slide entry with notes section |
| EC-002 | `notes` field uses `{{ expr }}` interpolation | Interpolation evaluated normally; resolved value placed in notes register |
| EC-003 | `notes` field contains inline formatting (bold, links) | Inline formatting rendered in PPTX notes shape, DOCX notes section, HTML aside |
| EC-004 | `notes` content in a `section methodology:` block | Only appears in format sections that support presenter notes; excluded from body |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide title:` + `title "Q1 Review"` + `notes "Emphasize the growth story"` built to PPTX | PPTX: `<p:notes>` contains "Emphasize the growth story"; slide body does not | happy-path (TV-11.1) |
| Same deck built to HTML | HTML: `<aside data-notes>Emphasize the growth story</aside>` present; not on canvas | happy-path |
| Same deck built to PDF | PDF: "Emphasize the growth story" absent from PDF main text | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | PPTX notes text is in <p:notes> not in slide body shapes | integration test: parse PPTX XML, verify text location |
| VP-TBD | PDF does not contain notes text | integration test: PDF text extraction (pdftotext), verify notes text absent |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 ("Writing Register Support") per capabilities.md §CAP-029 |
| Capability Anchor Justification | CAP-029 ("Writing Register Support") per capabilities.md §CAP-029 — "notes (presenter-facing): routes to PPTX speaker notes, DOCX notes, HTML aside" is the definition of this register in CAP-029 |
| L2 Domain Invariants | DI-012 (single .sf source produces all formats consistently) |
| Architecture Module | slideforge-eval + all exporter crates (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.14.002 — related to (report register routing; complementary)
- BC-1.14.003 — related to (detail register routing; complementary)
- BC-1.14.004 — composes with (no register bleeds to wrong format)

## Architecture Anchors

- `architecture/pipeline.md#register-routing` — register tag routing in pipeline stages

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
