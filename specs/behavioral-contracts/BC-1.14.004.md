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

# BC-1.14.004: No Register Content Bleeds to Wrong Format

## Description

The three writing registers (notes, report, detail) each route content to specific
output formats and must never appear in formats where they are not intended. Speaker
notes must not appear as PPTX slide body content. Report-register prose must not
appear as PPTX visual slide content. Detail-register content must not appear in the
web preview. This no-bleed invariant is the core of the "one source, every format"
value proposition.

## Preconditions

1. A .sf source uses all three writing registers: notes, report, and detail.
2. The deck is built to all 5 output formats: `--format pptx,docx,pdf,html,preview`.

## Postconditions

1. `notes` content appears ONLY in: PPTX speaker notes (`<p:notes>`), DOCX presenter notes section, HTML `<aside data-notes>` or equivalent.
2. `notes` content does NOT appear in: PPTX slide body, DOCX report body, PDF main text, web preview slide canvas.
3. `report` content appears ONLY in: DOCX body paragraphs, PDF main body text.
4. `report` content does NOT appear in: PPTX slide body, HTML web preview canvas.
5. `detail` content appears ONLY in: DOCX extended sections, PDF extended appendix.
6. `detail` content does NOT appear in: PPTX, HTML preview.

## Invariants

1. No register content crosses format boundaries. (DI-012)
2. The routing rules are determined at the Evaluate stage, not the Export stage.
3. A `LaidOutSlide` carries register-tagged content; each exporter knows which registers to include.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A slide has only `report` register content (no slide body) | Slide body is empty/placeholder in PPTX; report content appears only in DOCX |
| EC-002 | A slide has all three registers | PPTX: slide body (visual) + notes; DOCX: slide body + report + detail; PDF: slide body + report + detail |
| EC-003 | `detail` content in a standalone `section detail:` block | Only appears in DOCX/PDF; excluded from PPTX and web preview |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Slide with `notes "Presenter: emphasize this point"` | PPTX: notes present; DOCX body: notes absent | happy-path (TV-11.1) |
| Slide with `report "Detailed narrative for readers"` | DOCX body: narrative present; PPTX slide: narrative absent | happy-path |
| Slide with both body content and all 3 registers | Each format gets exactly the right registers | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | For each output format, only allowed registers are present | integration test: build to all formats; grep XML for cross-bled content |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 ("Writing Register Support") per capabilities.md §CAP-029 |
| Capability Anchor Justification | CAP-029 ("Writing Register Support") per capabilities.md §CAP-029 — "no register content bleeds across formats" is explicitly stated in CAP-029 as a requirement |
| L2 Domain Invariants | DI-012 (single .sf source produces all formats consistently) |
| Architecture Module | slideforge-eval + all exporter crates (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.14.001 — composes with (notes register routing)
- BC-1.14.002 — composes with (report register routing)
- BC-1.14.003 — composes with (detail register routing)

## Architecture Anchors

- `architecture/system-overview.md#register-routing` — register routing through pipeline stages

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
