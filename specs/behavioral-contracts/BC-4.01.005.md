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

# BC-4.01.005: Slide IDs Start at 256; Master IDs at 2^31; All 31 Layouts Present

## Description

OOXML schema and renderer compatibility require that slide IDs start at ≥ 256 and
slide master IDs start at ≥ 2^31 (2,147,483,648). In addition, all 11 standard layout
types plus 20 custom layouts (31 total, matching the 31 slideforge slide types per
BC-2.01.005) must be present in the PPTX file. These constraints were validated
empirically during Spike S6 and are required to avoid corrupt PPTX in some renderers.

## Preconditions

1. A valid `LaidOutDeck` IR is being serialized to PPTX.
2. The `Brand` struct includes a complete 31-layout hierarchy (BC-2.01.005).
3. The PPTX exporter is generating `<p:sldId>` and `<p:sldMasterId>` elements.

## Postconditions

1. Every `<p:sldId id="...">` attribute in `presentation.xml` has a value ≥ 256.
2. Slide IDs are unique within the presentation (no two slides share an ID).
3. The `<p:sldMasterId id="...">` attribute in `presentation.xml` has a value ≥ 2,147,483,648.
4. The PPTX contains exactly 31 slide layout parts in `ppt/slideLayouts/`.
5. Each of the 31 slideforge slide types has a corresponding layout referenced from
   the slide master.
6. The `[Content_Types].xml` registers all 31 slideLayouts and the slide master.

## Invariants

1. The slide ID sequence starts at exactly 256 (for deterministic output) and increments
   by 1 per slide.
2. The master ID is exactly 2,147,483,648 (2^31) — not a random value.
3. All 31 layouts are present even if not all slide types are used in the current deck.
   (Required for template reuse and for renderer compatibility.)
4. Two-IR model: ID assignment happens in the PPTX exporter, not in the layout engine.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with 1 slide | slide ID = 256; one `<p:sldId>` entry; all 31 layouts still present |
| EC-002 | Deck with 256 slides | slide IDs 256..511; no collision; all valid |
| EC-003 | Deck with 257 slides | slide IDs 256..512; still ≥ 256 for all; no overflow into invalid range |
| EC-004 | PPTX opened in LibreOffice after round-trip | IDs preserved on load/save cycle (LibreOffice does not renumber) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3-slide deck serialized to PPTX | `<p:sldId id="256">`, `<p:sldId id="257">`, `<p:sldId id="258">` | happy-path |
| Any deck | `<p:sldMasterId id="2147483648">` | happy-path |
| Any deck | 31 slideLayout parts in `ppt/slideLayouts/` | happy-path |
| Parse the generated PPTX with ooxmlsdk validation | Zero schema errors related to ID ranges | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All `<p:sldId id>` values in presentation.xml are ≥ 256 | unit test: parse presentation.xml, extract IDs, assert min ≥ 256 |
| VP-TBD | `<p:sldMasterId id>` value = 2147483648 | unit test |
| VP-TBD | Exactly 31 slideLayout XML parts present in PPTX ZIP | unit test: enumerate ZIP entries with path `ppt/slideLayouts/` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — "Slide IDs in the .pptx start at 256; master IDs start at 2^31" is an explicit OOXML correctness requirement for CAP-015 cross-renderer fidelity |
| L2 Domain Invariants | DI-013 (PPTX must pass multi-renderer fidelity — ID constraints are required for renderer compatibility) |
| Architecture Module | slideforge-pptx crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.01.001 — composes with (ID constraints are part of BC-4.01.001's Postcondition 5 and 6)
- BC-4.01.002 — depends on (visual fidelity check validates output including correct ID assignment)
- BC-2.01.005 — depends on (31 layouts in the brand is the prerequisite for 31 layouts in the PPTX)

## Architecture Anchors

- `architecture/export-architecture.md#pptx` — slide ID and master ID assignment (Spike S6 BUG-006)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
