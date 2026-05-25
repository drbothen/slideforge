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
capability: CAP-017
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

# BC-4.03.001: PDF Output Passes veraPDF --flavour ua1 (PDF/UA-1 Compliant, Tagged)

## Description

The slideforge PDF exporter produces tagged PDF with a `/StructTreeRoot`, logical
reading order, alt text on all non-decorative figures, font ToUnicode CMaps,
document language metadata, and `/MarkInfo`. The output must pass
`verapdf --flavour ua1` validation in CI with zero violations. Rasterized slide
images are not acceptable. The implementation uses pdf-writer + krilla + a custom
`SlideTagEngine` (Spike S2 resolution — Chrome headless is explicitly rejected).

## Preconditions

1. A valid `LaidOutDeck` IR exists with all non-decorative visual elements having non-empty alt text.
2. `lang "..."` is declared in deck metadata (DI-003).
3. The `slideforge-pdf` crate is built with pdf-writer + krilla.
4. Font files are available for embedding (required for ToUnicode CMaps).
5. veraPDF CLI is installed on CI (`brew install verapdf` or Docker image).

## Postconditions

1. A .pdf file is written containing:
   - `/StructTreeRoot` with a complete structure tree.
   - One `/Part` element per slide in the structure tree.
   - All text elements tagged with correct PDF standard structure types (H1-H6, P, LI, etc.).
   - All figure elements tagged as `/Figure` with `/Alt` text from DSL `alt "..."` field.
   - Decorative elements marked as PDF Artifacts (excluded from structure tree).
   - `/MarkInfo << /Marked true >>`.
   - `/Lang` set to the deck's declared language.
   - All fonts subsetted with valid `/ToUnicode` CMaps.
2. `verapdf --flavour ua1` exits with `isCompliant: true` and zero violations.
3. Text in the PDF is searchable, copy-pasteable, and screen-reader accessible.
4. Charts (SVG from plotters) are embedded as vector paths via usvg normalization, not rasterized.

## Invariants

1. Chrome headless is NEVER used for PDF production. (Spike S2 explicit rejection — binary size incompatible with single-binary distribution, PDF/UA-1 unreliable for absolute-positioned layouts)
2. The structure tree reading order matches the `reading_order: Vec<ElementId>` in `LaidOutSlide`.
3. Every `Figure` element in the structure tree has a non-empty `/Alt` entry.
4. Font subsetting is performed using the `subsetter` crate (same as Typst).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with decorative images only on one slide | Decorative elements marked as Artifacts; slide still has valid structure tree (at minimum a /Part element) |
| EC-002 | Chart SVG with complex paths (> 10,000 path commands) | usvg normalizes and embeds as paths; may be large but valid |
| EC-003 | Math expression in slide title | OMML is for PPTX/DOCX; for PDF, math is rendered via pdf-writer path drawing. veraPDF must still pass. |
| EC-004 | Multilingual deck (slides in English and French) | `/Lang` at document level = deck lang; individual element lang override = TBD (v1.0: document-level lang only) |
| EC-005 | Font not available at export time | E-BRD-004 warning; fallback font used; ToUnicode CMap generated for fallback font |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3-slide deck (title + content + severity_cards), lang "en-US", all alts present | PDF passes veraPDF ua1; exit 0 from verapdf | happy-path (TV-10.1) |
| Same deck rendered to PDF | PDF text is searchable (verify via pdftotext or similar) | happy-path |
| Deck with decorative: true on all images | PDF passes veraPDF; Artifact tags present | edge-case |
| Deck with missing alt on image | This should be caught at compile time (BC-5.01.001); should not reach PDF export | blocked upstream |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | veraPDF CI gate passes for all fixture decks | CI: `verapdf --flavour ua1 output.pdf` exit 0 |
| VP-TBD | Structure tree element count = sum of all tagged elements across slides | unit test: count structure tree entries |
| VP-TBD | EMU-to-PDF-units coordinate mapping: 9144000 EMU = 720.0pt | kani proof (per S2 code sample) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 |
| Capability Anchor Justification | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 — "tagged, PDF/UA-1 compliant via [direct backend]" is the explicit PDF accessibility requirement in CAP-017 |
| L2 Domain Invariants | DI-014 (PDF output must be tagged PDF/UA-1 compliant) |
| Architecture Module | slideforge-pdf crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.03.002 — composes with (this BC specifies the compliance target; BC-4.03.002 specifies the stack)
- BC-4.03.005 — depends on (coordinate mapping correctness is required for structure tree positioning)
- BC-5.01.001 — depends on (alt text compile enforcement prevents missing alt from reaching PDF stage)

## Architecture Anchors

- `architecture/export-architecture.md` — pdf-writer + krilla + SlideTagEngine design (Spike S2)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
