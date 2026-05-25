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
capability: CAP-011
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

# BC-3.02.002: Manually Authored Section Blocks (section methodology:) Appear in DOCX/PDF

## Description

A `section <type>:` block in a .sf file (e.g., `section methodology:`, `section scope:`,
`section approval:`) declares manually authored narrative content that appears as a
structured section in DOCX and PDF output. Section blocks are not rendered in PPTX or
web preview. They can contain rich inline formatting, `report` register sub-blocks, and
`{{ }}` interpolation.

## Preconditions

1. The .sf file contains one or more `section <type>:` blocks.
2. The section type is a recognized `SectionType` (methodology, scope, approval, appendix,
   glossary, and any type registered via the SectionType plugin trait).
3. The section appears at the top level of the deck (not inside a slide block).

## Postconditions

1. In DOCX output: the section appears as a Word section with a heading styled to match
   the declared section type, followed by the section's content as body paragraphs.
2. In PDF output: the section appears as a tagged section with appropriate heading level.
3. In PPTX output: the section block is IGNORED (not rendered, no placeholder slide
   generated).
4. In web preview: the section block is IGNORED.
5. Inline `{{ }}` interpolations within section content are resolved before rendering.
6. The section preserves its source order relative to other sections and slides'
   `report` register content.

## Invariants

1. Section blocks are output-format conditional: DOCX/PDF only. (DI-012)
2. The SectionType trait handles all section rendering — no per-type hard-coding. (DI-008)
3. An unrecognized `section <type>:` produces a compile error naming the unknown type
   and listing registered types.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Unrecognized section type `section foobar:` | E-PAR-007-class error: "Unknown section type 'foobar'. Known types: [methodology, scope, approval, appendix, glossary, ...]" |
| EC-002 | section block inside a slide block | Parse error: section blocks must be top-level, not nested inside slide blocks |
| EC-003 | section block with only @if content that evaluates to false | Section produces no body content; section heading is still emitted in DOCX (empty section); lint warning |
| EC-004 | section block with report: sub-block | Report sub-block content is included in the section's DOCX output |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `section methodology: body "We applied..."` | DOCX has "Methodology" heading + paragraph; PPTX has no corresponding element | happy-path |
| `section scope: body "{{ client_name }} engagement"` | DOCX has "Scope" heading + interpolated paragraph | happy-path |
| Deck exported to .pptx only | Section blocks not rendered; no error | edge-case |
| `section xyz:` with unrecognized type | E-PAR-007-class error; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Section blocks absent from PPTX zip package (no corresponding slide XML) | integration test (inspect .pptx zip; assert section content absent) |
| VP-TBD | Section heading styles match SectionType taxonomy in DOCX | snapshot test (Word XML heading style assertion) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 ("Document Section Generation") per capabilities.md §CAP-011 |
| Capability Anchor Justification | CAP-011 ("Document Section Generation") per capabilities.md §CAP-011 — "manually authored (section methodology:, section scope:, section approval:)" is explicitly enumerated in CAP-011 |
| L2 Domain Invariants | DI-008 (SectionType trait API), DI-012 (single source produces all formats) |
| Architecture Module | slideforge-eval crate — section block parsing; slideforge-docx crate — SectionType rendering (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.02.001 — related to (auto-generated sections use the same SectionType trait rendering path)
- BC-1.14.002 — related to (report register and section blocks are two distinct DOCX content sources)

## Architecture Anchors

- `architecture/layout-subsystem.md#section-types` — SectionType trait and manual section blocks

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
