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
capability: CAP-015
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

# BC-4.01.001: Serialize LaidOutDeck to Valid .pptx with Correct Placeholder Inheritance

## Description

The PPTX exporter accepts a `LaidOutDeck` IR and serializes it to a valid OOXML .pptx
file using ooxmlsdk. The output conforms to ECMA-376 schema constraints including
correct placeholder inheritance (slide → layout → master), proper element ordering,
and complete relationship references. The output must be openable in PowerPoint,
Keynote, Google Slides, and LibreOffice without schema errors or missing content.

## Preconditions

1. A valid `LaidOutDeck` IR exists (produced by the layout stage with no errors).
2. A `Brand` struct is available (from synthesis or loading).
3. The target output directory exists and is writable.
4. `LaidOutDeck` IR types implement `Hash + Eq + Clone` (required by DI-011).
5. All coordinates in `LaidOutDeck` are integer EMUs (required by DI-010).

## Postconditions

1. A valid .pptx file is written to the output path.
2. The .pptx passes OOXML schema validation (structural validity, no schema errors).
3. Placeholder inheritance is correct: slide placeholders inherit from layout by `idx`, layout inherits from master by `type`.
4. The .pptx is openable in PowerPoint 365, LibreOffice Impress, and Keynote without error dialogs.
5. Slide IDs in the .pptx start at ≥ 256 (ECMA-376 minimum, per Spike S6 BUG-006).
6. Master slide IDs start at ≥ 2^31.
7. `[Content_Types].xml` registers every part type present in the ZIP.
8. All semantic information required by other exporters (DOCX, PDF) is preserved in `LaidOutDeck` (DI-009).

## Invariants

1. Two-IR model integrity: the PPTX exporter reads from `LaidOutDeck` only — it does not modify the `Deck` IR. (DI-009)
2. Integer EMU coordinates only — no f64 in the OOXML output path. (DI-010)
3. All `LaidOutDeck` IR types used in the export path implement `Hash + Eq + Clone`. (DI-011)
4. Same `LaidOutDeck` + same `Brand` produces byte-identical output (determinism, excluding timestamps).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no visual elements (only notes/metadata) | Valid slide produced; empty content area; speaker notes present |
| EC-002 | Slide with maximum content (31 bullet points) | Valid PPTX; potential E-LAY-001 canvas overflow warning (not a PPTX error) |
| EC-003 | Deck with only 1 slide | Valid 1-slide PPTX; all standard ZIP parts present |
| EC-004 | Chart SVG embedded as media part | Chart present in `/ppt/media/chart1.svg`; relationship reference correct |
| EC-005 | Diagram SVG embedded as media part | SVG has no `foreignObject` (per BC-1.12.003); embedded correctly |
| EC-006 | Output directory does not exist | Directory created; output written. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| LaidOutDeck with 1 title slide | Valid .pptx; 1 slide; title placeholder populated | happy-path |
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

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — this BC is the primary serialization contract for CAP-015, covering the core serialize-to-.pptx requirement |
| L2 Domain Invariants | DI-009 (Two-IR model integrity), DI-010 (integer EMU), DI-011 (Hash+Eq+Clone), DI-012 (single source → all formats consistent) |
| Architecture Module | slideforge-pptx crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

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
