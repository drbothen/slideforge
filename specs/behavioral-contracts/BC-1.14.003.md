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

# BC-1.14.003: detail Register Routes to DOCX/PDF Only; Excluded from PPTX and Web Preview

## Description

The `detail` writing register is the document-only extended analysis register. Content
declared in a `detail` field or `detail:` block is routed exclusively to DOCX extended
sections and PDF appendix/extended body content. The detail register is excluded from
PPTX (both slide body and speaker notes), HTML static output canvas, and the web preview.
This register enables long-form technical appendices that have no place in a presentation
context.

## Preconditions

1. A .sf source contains at least one slide or section with a `detail "..."` field or `detail:` block.
2. The deck is built to one or more output formats.

## Postconditions

1. DOCX output: detail content appears in extended sections of the DOCX document (after the main slide-derived report body).
2. PDF output: detail content appears in the PDF extended body or appendix section.
3. PPTX output: detail content does NOT appear anywhere in the PPTX file — not in slide shapes, not in speaker notes.
4. Static HTML output: detail content does NOT appear in the slide canvas. If included in HTML output at all, it appears in a designated `<section data-detail>` area.
5. Web preview: detail content is NOT rendered.
6. Build exits with code 0 for valid source.

## Invariants

1. Detail register routing is determined at the Evaluate stage.
2. No detail content appears in PPTX or web preview under any circumstances.
3. Detail register participates in DI-012.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `section detail:` standalone block with no parent slide | Valid — detail sections are independent of slide content; appear in DOCX/PDF as standalone sections |
| EC-002 | `detail` field on every slide + built to PPTX only | No detail content in PPTX; exit 0 (no error — it is expected that detail is excluded from PPTX) |
| EC-003 | `detail` block with markdown-style headings | Headings rendered as DOCX styled headings (Heading 2/3) in the extended section |
| EC-004 | `detail` and `report` content both present on same slide | Both render in DOCX (report as body, detail as extended section); both absent from PPTX |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Slide with `detail "Technical analysis..."` built to PPTX | PPTX: no detail text in any XML element; exit 0 | happy-path |
| Same slide built to DOCX | DOCX: "Technical analysis..." in extended section; exit 0 | happy-path |
| Same slide built to web preview | Preview: no detail text on canvas; exit 0 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | PPTX contains no detail register text nodes | integration test: parse PPTX XML; grep for detail content |
| VP-TBD | DOCX contains detail content in extended section | integration test: parse DOCX XML; verify detail text in extended section structure |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 ("Writing Register Support") per capabilities.md §CAP-029 |
| Capability Anchor Justification | CAP-029 ("Writing Register Support") per capabilities.md §CAP-029 — "detail (document-only extended analysis): routes to DOCX/PDF only; excluded from PPTX and web preview" is the definition in CAP-029 |
| L2 Domain Invariants | DI-012 (single .sf source produces all formats consistently) |
| Architecture Module | slideforge-eval + slideforge-docx + slideforge-pdf crates (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.14.001 — related to (notes register routing)
- BC-1.14.002 — related to (report register routing)
- BC-1.14.004 — composes with (no register bleeds to wrong format)

## Architecture Anchors

- `architecture/system-overview.md` — register tag routing in pipeline stages

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
