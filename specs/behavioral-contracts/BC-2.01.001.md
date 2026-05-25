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
capability: CAP-018
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

# BC-2.01.001: Load Brand from Existing .pptx or .docx Template File

## Description

When a deck declares `brand "path/to/template.pptx"` (or `.docx`), the brand loader
reads the OOXML package, extracts theme colors (all 12 slots), font configuration,
logo reference, and footer text into an in-memory `BrandTemplate` struct. This struct
is then consumed by the PPTX exporter to apply the brand to synthesized slides.

## Preconditions

1. The template path resolves to a readable file.
2. The file has a `.pptx` or `.docx` extension.
3. The file is a valid OOXML ZIP package with `ppt/theme/theme1.xml` (PPTX) or
   `word/theme/theme1.xml` (DOCX) present.

## Postconditions

1. A `BrandTemplate` struct is returned containing all 12 OOXML scheme color slots
   (dk1, lt1, dk2, lt2, acc1-acc6, hlink, folHlink), heading/body font names,
   logo image bytes (if detected in slideMaster1.xml.rels), and footer text.
2. All 31 slide layouts are discoverable from the loaded template (slideLayout1.xml
   through slideLayoutN.xml are mapped by name to the taxonomy in Spike S5).
3. Build exits with code 0.

## Invariants

1. The loaded `BrandTemplate` always exposes all 12 OOXML color slots — any missing
   slot in the source template triggers E-BRD-003 with an inferred value. (DI-015)
2. The loading is read-only; the source template file is never modified.
3. Loading a PPTX brand template does not affect the `Deck` IR; it only affects
   the export stage.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Template file not found at resolved path | E-BRD-001 (broken, exit 4): brand file not found |
| EC-002 | File is not a valid ZIP/OOXML package (corrupt) | E-BRD-002 (broken, exit 4): cannot parse brand template |
| EC-003 | Template has only 8 of 12 color slots in theme1.xml | 4 E-BRD-003 warnings for missing slots; build continues with inferred values |
| EC-004 | Template has multiple slide masters | Only slideMaster1.xml is extracted in v1.0; a lint warning notes multi-master templates |
| EC-005 | Template has no logo (no image relationship from slideMaster) | BrandTemplate.logo = None; no error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid corporate .pptx with all 12 colors | BrandTemplate with exact colors; exit 0 | happy-path |
| Valid .docx brand template | BrandTemplate loaded from word/theme/theme1.xml; exit 0 | happy-path |
| Non-existent path "missing.pptx" | E-BRD-001; exit 4 | error |
| Corrupt file (not a ZIP) | E-BRD-002; exit 4 | error |
| Template with 8 colors in theme | BrandTemplate with 12 colors (4 inferred); 4 E-BRD-003 warnings | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Loaded template always has exactly 12 color entries | unit test (count fields) |
| VP-TBD | Source template file is unmodified after loading | integration test (file hash before/after) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 |
| Capability Anchor Justification | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 — loading from an existing .pptx/.docx is the first loading path explicitly described in CAP-018 |
| L2 Domain Invariants | DI-015 (brand palette must cover all 12 OOXML slots), DI-016 (single master architecture) |
| Architecture Module | slideforge-brand crate — BrandLoader (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.01.002 — related to (synthesis from brand.toml is the complementary loading path)
- BC-2.01.003 — related to (extract-brand produces brand.toml from the same OOXML this BC loads)
- BC-4.01.001 — depends on (PPTX exporter consumes the BrandTemplate produced here)

## Architecture Anchors

- `architecture/branding-subsystem.md#loading` — brand loading from existing .pptx/.docx
- `architecture/pipeline.md` — brand resolution as a pre-export stage

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
