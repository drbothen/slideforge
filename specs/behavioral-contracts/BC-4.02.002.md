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

# BC-4.02.002: Auto-Generated Document Sections Present in DOCX from Slide Data

## Description

The DOCX exporter produces auto-generated document sections (executive_summary,
risk_register, methodology, scope, approval, etc.) from slide data, in addition
to the per-slide narrative body paragraphs from BC-4.02.001. Auto-generated
sections derive their content from specific slide types: executive_summary from
`takeaway` fields, risk_register from `severity_cards` slides, etc. This satisfies
CAP-011 for DOCX output.

## Preconditions

1. A valid `LaidOutDeck` IR exists with slide type information preserved.
2. The deck contains at least one slide type that maps to an auto-generated section
   (e.g., `slide severity_cards:`, `slide key_insights:`, etc.).
3. The DOCX exporter has access to the section-generation rules from the `SectionType`
   plugin registry.

## Postconditions

1. For each configured auto-generation rule, a corresponding document section appears
   in the .docx output after the main narrative body.
2. executive_summary section contains aggregated `takeaway` field content from all
   slides in the deck.
3. risk_register section contains risk items from all `slide severity_cards:` blocks
   in tabular format.
4. Manually-authored `section <name>:` blocks appear at their declared position in
   the document.
5. Section ordering follows the document structure: intro → per-slide narrative →
   auto-generated sections → manually-authored sections → approval section (if present).
6. The .docx opens and renders correctly in Word 365 and LibreOffice Writer.

## Invariants

1. Auto-generation rules are declarative and managed by the `SectionType` plugin —
   the DOCX exporter does not hardcode section logic. (DI-008 — plugin architecture)
2. An auto-generated section is only emitted when there is at least one source slide
   of the required type; empty sections are not emitted.
3. Manually-authored sections appear at their explicit position — they are NOT merged
   with auto-generated sections of the same logical name.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No `severity_cards` slides in deck | No risk_register section emitted in .docx |
| EC-002 | Multiple `severity_cards` slides | All risk items aggregated into single risk_register section |
| EC-003 | Manually-authored `section scope:` AND auto-generated scope section | Manual section wins; auto-generation does not duplicate |
| EC-004 | `takeaway` field empty on all slides | executive_summary section not emitted (empty source data) |
| EC-005 | Deck with 20 severity_cards slides | Risk register table has 20 rows; no row limit |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with 3 `severity_cards` slides (HIGH, MED, LOW) | .docx risk_register section with 3-row table; severity labels present | happy-path |
| Deck with `takeaway "Key finding: X"` on 2 slides | .docx executive_summary section with 2 takeaway paragraphs | happy-path |
| Deck with no auto-generatable slides | .docx has only per-slide narrative body; no auto sections | edge-case |
| Deck with `section methodology:` block | .docx has methodology section at declared position | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | risk_register row count = severity_cards slide count | integration test: parse .docx XML, count table rows in risk_register section |
| VP-TBD | executive_summary paragraph count = count of non-empty takeaway fields | integration test |
| VP-TBD | Auto-sections absent when source slide type absent from deck | unit test with minimal deck (no severity_cards) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 ("DOCX Export") per capabilities.md §CAP-016 |
| Capability Anchor Justification | CAP-016 ("DOCX Export") per capabilities.md §CAP-016 — "auto-generating document sections from slide data" is verbatim from CAP-016; this BC specifies the DOCX-specific behavior of CAP-011 |
| L2 Domain Invariants | DI-008 (all bundled plugins use plugin trait APIs — SectionType plugin manages section generation logic), DI-012 (single source → all formats consistent) |
| Architecture Module | slideforge-docx crate + SectionType plugin (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.02.001 — composes with (auto-sections appear after the per-slide narrative body)
- BC-3.02.001 — depends on (auto-generated DOCX sections are the output of the SectionType rules defined there)
- BC-5.02.001 — depends on (SectionType is one of the 10 plugin surfaces; must use plugin API)

## Architecture Anchors

- `architecture/export-architecture.md#docx` — auto-generated section design
- `architecture/plugin-architecture.md#section-type` — SectionType plugin trait

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
