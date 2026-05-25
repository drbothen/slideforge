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

# BC-4.01.003: PPTX Contains Speaker Notes, Master/Layout/Theme System, and Slide Sections

## Description

The PPTX exporter must embed speaker notes from the `notes` writing register into
`notesSlide` parts, generate a complete master/layout/theme hierarchy from the brand,
and produce slide section groupings when `section` blocks are declared. These structural
components are required for professional presentation use and are tested in all four
target renderers.

## Preconditions

1. A valid `LaidOutDeck` IR exists with at least one slide.
2. A `Brand` struct is available with a complete master/layout hierarchy (BC-2.01.005).
3. Speaker notes content from the `notes` register has been resolved (BC-1.14.001).
4. `section` groupings are present in the deck metadata if applicable.

## Postconditions

1. Every slide that has non-empty `notes` content has a corresponding `notesSlide` part
   in the PPTX ZIP at `ppt/notesSlides/notesSlide<N>.xml`.
2. Slides without notes content have no `notesSlide` part (or an empty one — either is valid).
3. The PPTX contains a slide master (`ppt/slideMasters/slideMaster1.xml`) and at minimum
   11 standard + 20 custom slide layouts (per BC-2.01.005).
4. A `theme1.xml` is present in `ppt/theme/` and references all 12 OOXML color slots (DI-015).
5. When `section` blocks are declared, the PPTX `<p:sectionLst>` element groups slides
   accordingly.
6. The PPTX opens in PowerPoint 365 with notes visible in the Notes pane.

## Invariants

1. Speaker notes are NEVER placed in slide body content — they route exclusively to
   `notesSlide` parts. (DI-012 — register routing fidelity)
2. The master/layout hierarchy is generated from the brand, not hardcoded.
3. Slide section memberships are derived from the `section` DSL blocks; no section data
   is fabricated.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with empty notes register | No `notesSlide` part generated for that slide (or empty notesSlide) |
| EC-002 | Deck with no section blocks | No `<p:sectionLst>` element in presentationPr; silently omitted |
| EC-003 | Notes content contains {{ }} interpolation | Interpolation evaluated before embedding in notesSlide XML |
| EC-004 | Section name contains XML special characters (<, >, &) | Section name XML-escaped in `<p:sectionLst>` |
| EC-005 | Two sections containing a single slide each | Both sections present; each slide assigned to its section |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with 3 slides, each having `notes "Speaker text"` | PPTX has 3 notesSlide parts; PowerPoint notes pane shows text | happy-path |
| Deck with `section "Background":` grouping 2 slides | PPTX `<p:sectionLst>` has 1 section entry with 2 slide refs | happy-path |
| Deck with no notes fields | PPTX has 0 notesSlide parts (or empty); no error | edge-case |
| Deck with notes `alt "< & >"` | notesSlide XML: `&lt; &amp; &gt;` — no malformed XML | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | notesSlide count = count of slides with non-empty notes | unit test: unzip PPTX, count notesSlide parts |
| VP-TBD | theme1.xml contains all 12 OOXML color slot elements | snapshot test |
| VP-TBD | Section slide refs are correct and non-overlapping | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — "Embed speaker notes, master/layout/theme system, placeholder inheritance, slide sections" is verbatim from CAP-015 |
| L2 Domain Invariants | DI-012 (single source → all formats consistent; register routing), DI-015 (brand palette covers all 12 color slots) |
| Architecture Module | slideforge-pptx crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.01.001 — composes with (this BC specifies structural components of the PPTX produced by BC-4.01.001)
- BC-1.14.001 — depends on (notes register routing is a prerequisite)
- BC-2.01.005 — depends on (31 layouts must exist in the master for this BC to pass)
- BC-4.01.006 — composes with (notesMaster is a related structural requirement)

## Architecture Anchors

- `architecture/export-subsystem.md#pptx` — PPTX structural requirements

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
