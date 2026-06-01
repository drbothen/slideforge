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
capability: CAP-011
lifecycle_status: active
introduced: v1.0.0
modified:
  - "2026-06-01: v1.2 — Added postcondition 7 (detail: sub-blocks), postcondition 8 (inline-structure preservation via FieldValue::Inlines), EC-005 (unrecognized sub-block key → non-fatal warning), EC-006 (reserved-name collision); clarified EC-004; added BC-1.14.003 cross-reference. Closes STORY-077 BC-status flag."
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
web preview. They can contain rich inline formatting, `report` and `detail` register
sub-blocks, and `{{ }}` interpolation. The detail routing semantics (exclusion from PPTX
and web preview) are owned by BC-1.14.003; this contract governs the section block's
structural role, its appearance in DOCX/PDF, and the evaluation-stage tagging of
sub-block content as `RegisteredContent`.

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
7. A `section <type>:` block may contain a `detail:` sub-block. At the Evaluate stage,
   that sub-block's content is extracted and attached to the section's output node as
   `RegisteredContent { register: Register::Detail, content: Vec<InlineNode> }`. DOCX
   and PDF exporters read this entry from the section node and render it in the appropriate
   extended section. PPTX and web preview exporters do NOT read section node
   `register_content`. (Detail routing exclusion rules are governed by BC-1.14.003; this
   postcondition covers the tagging obligation.)
8. The body sub-block content of a `section <type>:` block (whether `report:`, `detail:`,
   or plain narrative body) is stored in `SectionBlock.body` as `FieldValue::Inlines`
   (not as a flat `Value::Str`). This preserves rich inline structure — bold, xref/links,
   `{{ }}` interpolation nodes — from parse time through to the Evaluate stage without
   any loss of structural information. Observable consequence: bold text within a section
   sub-block renders as bold in DOCX/PDF output (not as literal asterisks).

## Invariants

1. Section blocks are output-format conditional: DOCX/PDF only. (DI-012)
2. The SectionType trait handles all section rendering — no per-type hard-coding. (DI-008)
3. An unrecognized `section <type>:` (unrecognized section TYPE at the block level) is a
   FATAL compile error naming the unknown type and listing registered types. This is
   distinct from invariant 4 below, which governs sub-block keys inside a recognized section.
4. An unrecognized sub-block key inside a RECOGNIZED `section <type>:` block (e.g., `foo:`
   inside `section methodology:`) is a NON-FATAL lint warning naming the key. The warning
   is emitted at parse time. Build continues; the unrecognized key is silently ignored.
   Exception: if the unknown key coincidentally matches a reserved register name
   (`notes`, `report`, `detail`) but is not being used as a register declaration, the
   reserved-name collision policy applies (surfaced as a parse error with a corrective hint).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Unrecognized section type `section foobar:` | E-PAR-007-class error: "Unknown section type 'foobar'. Known types: [methodology, scope, approval, appendix, glossary, ...]"; build exits 1 |
| EC-002 | section block inside a slide block | Parse error: section blocks must be top-level, not nested inside slide blocks |
| EC-003 | section block with only @if content that evaluates to false | Section produces no body content; section heading is still emitted in DOCX (empty section); lint warning |
| EC-004 | section block with `report:` sub-block | At Evaluate stage: `RegisteredContent { register: Register::Report, content: Vec<InlineNode> }` is attached to the section's output node. DOCX exporter reads this entry and renders it as section body content. PPTX/web preview do not render it. (BC-1.14.002 governs report routing exclusion rules.) |
| EC-005 | Unrecognized sub-block key inside recognized section (e.g., `foo:` inside `section methodology:`) | Non-fatal lint warning at parse time: "Unrecognized section sub-block key 'foo' — ignored". Build continues. This is governed by invariant 4, not invariant 3. |
| EC-006 | Unrecognized sub-block key that collides with a reserved register name | Parse error with corrective hint: "Key '<name>' is a reserved register name — use `<name>:` register syntax or choose a different key." Build exits 1. |

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
| Stories | STORY-077 |

## Related BCs

- BC-3.02.001 — related to (auto-generated sections use the same SectionType trait rendering path)
- BC-1.14.002 — related to (report register routing exclusion rules; applies when section `report:` sub-block is present)
- BC-1.14.003 — authority for (detail register routing exclusion rules; postcondition 7 of this BC defers to BC-1.14.003 for PPTX/web exclusion semantics; BC-1.14.003 EC-001 explicitly covers the `section detail:` standalone case)

## Architecture Anchors

- `architecture/crate-architecture.md` — SectionType trait and manual section blocks

## Story Anchor

STORY-077 — SectionBlock IR Extension: FieldValue body + section-level register routing

## VP Anchors

(filled after VP creation)
