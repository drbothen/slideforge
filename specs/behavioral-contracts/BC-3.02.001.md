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

# BC-3.02.001: Auto-Generated DOCX Sections Derived from Slide Data (executive_summary, risk_register)

## Description

The DOCX exporter auto-generates structured document sections from semantic slide data
without any explicit `section` block in the .sf source. For example, `takeaway` fields
across all slides are collected into an `executive_summary` section; `slide severity_cards:`
slides are aggregated into a `risk_register` section. This transforms presentation data
into a connected formal document automatically.

## Preconditions

1. The deck is being exported to `.docx` (or `.pdf`).
2. The deck contains slide types that contribute to auto-generated section types
   (e.g., `takeaway` fields, `slide severity_cards:` blocks).
3. No conflicting explicit `section executive_summary:` block is declared in the .sf
   file (if one is, it supersedes the auto-generated content for that section).

## Postconditions

1. The DOCX output contains auto-generated sections as top-level Word sections with
   appropriate heading styles.
2. `executive_summary` section: populated with `takeaway` field text from all slides
   that declare a `takeaway:` field, in slide order.
3. `risk_register` section: populated with structured rows from all `slide severity_cards:`
   blocks (one row per card entry: title, severity, description, owner).
4. Auto-generated sections appear at the end of the DOCX by default (after slide-mapped
   body content) unless `section_order:` is declared in deck metadata.
5. If no slides contribute data to an auto-generated section type, that section is omitted
   from the DOCX (no empty section headings).

## Invariants

1. Auto-generation is deterministic: same deck always produces the same sections in the
   same order. (DI-012)
2. Auto-generated section content is derived from slide data already in the `Deck` IR;
   it is not re-parsed from the source file.
3. `report` register content in slides also flows into the DOCX body (per BC-1.14.002),
   separate from auto-generated sections.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck has no takeaway fields and no severity_cards slides | No executive_summary or risk_register sections generated; no error |
| EC-002 | Both auto-generated and manual executive_summary: block present | Manual `section executive_summary:` block supersedes auto-generated; lint warning: "Auto-generated executive_summary overridden by explicit section block" |
| EC-003 | severity_cards slide in a @for loop (multiple instances) | All loop iterations contribute rows to the risk_register; de-duplication is NOT applied |
| EC-004 | @if block suppresses all severity_cards slides | No risk_register section generated (no contributing slides) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with 5 slides each having `takeaway: "..."` | DOCX has executive_summary section with 5 takeaway bullet points | happy-path |
| Deck with 2 severity_cards slides | DOCX has risk_register section with 2N rows (N per slide, based on card count) | happy-path |
| Deck with no takeaway fields | No executive_summary section in DOCX | edge-case |
| Deck exported to .pptx only (no .docx) | No section generation attempted | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Auto-generated sections contain exactly the data from contributing slides in order | integration test (verify section content matches takeaway field values) |
| VP-TBD | No section generated when no slides contribute data | integration test (assert section heading absent) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 ("Document Section Generation") per capabilities.md §CAP-011 |
| Capability Anchor Justification | CAP-011 ("Document Section Generation") per capabilities.md §CAP-011 — auto-generated sections from slide data (executive_summary from takeaway fields, risk_register from severity_cards) is explicitly named in CAP-011 |
| L2 Domain Invariants | DI-012 (single .sf source produces all formats consistently) |
| Architecture Module | slideforge-docx crate — SectionType plugins for auto-generated sections (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.02.002 — related to (manual section blocks use the same SectionType trait as auto-generated sections)
- BC-4.02.001 — depends on (DOCX serializer renders the auto-generated sections into Word XML)
- BC-1.14.002 — related to (report register is a different content routing mechanism from auto-generated sections)

## Architecture Anchors

- `architecture/layout-subsystem.md#section-types` — SectionType trait and auto-generation logic

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
