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

# BC-4.01.006: PPTX Contains notesMaster1.xml and handoutMaster1.xml Even If Empty

## Description

OOXML schema compliance requires `notesMaster1.xml` and `handoutMaster1.xml` to be
present in every PPTX file, even when no presenter notes or handout content is used.
Omitting these parts causes "repair required" dialogs in some renderers (validated in
Spike S6 BUG-002). Both files must also be registered in `[Content_Types].xml` and
referenced via relationships.

## Preconditions

1. A PPTX file is being produced by the PPTX exporter (BC-4.01.001).
2. The Brand struct provides notes master and handout master templates (or default
   empty templates are generated).

## Postconditions

1. The PPTX ZIP contains `ppt/notesMasters/notesMaster1.xml`.
2. The PPTX ZIP contains `ppt/handoutMasters/handoutMaster1.xml`.
3. Both parts are registered in `[Content_Types].xml` with the correct OOXML content types:
   - `application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml`
   - `application/vnd.openxmlformats-officedocument.presentationml.handoutMaster+xml`
4. `ppt/_rels/presentation.xml.rels` contains relationship entries for both masters.
5. The PPTX opens in PowerPoint 365, LibreOffice Impress, and Keynote without "repair
   required" or "missing parts" dialogs.

## Invariants

1. notesMaster1.xml and handoutMaster1.xml are ALWAYS emitted — no conditional logic
   omits them even when notes content is absent.
2. The content types are exact OOXML strings (no variants or abbreviations).
3. If the brand template includes custom notesMaster/handoutMaster styling, that styling
   is preserved; if absent, minimal valid empty masters are generated.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Brand template (.pptx) has no notesMaster | Slideforge generates a minimal valid notesMaster; no error |
| EC-002 | Deck with zero notes content | notesMaster1.xml still present; 0 notesSlide parts (per BC-4.01.003) |
| EC-003 | PPTX opened in Google Slides | Google Slides ignores handout master; no corruption |
| EC-004 | ZIP entry path casing (notesMasters vs notesMaster) | Exact path `ppt/notesMasters/notesMaster1.xml` used; case-sensitive ZIP |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Any deck → PPTX | `ppt/notesMasters/notesMaster1.xml` present in ZIP | happy-path |
| Any deck → PPTX | `ppt/handoutMasters/handoutMaster1.xml` present in ZIP | happy-path |
| Any deck → PPTX | `[Content_Types].xml` has both content type entries | happy-path |
| Deck with no notes content | notesMaster present; no notesSlide parts; PPTX opens without repair dialog | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Both master parts present in every produced PPTX | unit test: enumerate ZIP; assert path existence |
| VP-TBD | Both content types registered in [Content_Types].xml | unit test: parse content types XML |
| VP-TBD | PPTX opens without repair in LibreOffice | CI: LibreOffice --headless convert; check exit code |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — OOXML structural completeness (notesMaster/handoutMaster required even if empty) is part of the cross-renderer fidelity requirement in CAP-015 |
| L2 Domain Invariants | DI-013 (PPTX must pass multi-renderer fidelity check — missing required parts causes repair dialogs) |
| Architecture Module | slideforge-pptx crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.01.001 — composes with (required parts are part of the valid PPTX produced by BC-4.01.001)
- BC-4.01.003 — composes with (speaker notes master is referenced by notesSlide parts)
- BC-4.01.002 — depends on (visual parity gate also checks for repair-free opening)

## Architecture Anchors

- `architecture/export-architecture.md` — required OOXML parts list (Spike S6 BUG-002)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
