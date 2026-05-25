---
document_type: architecture-section
section: export-architecture
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Export Architecture

## Shared Input Contract

All exporters consume the same two IRs via the `Exporter` trait:
- `Deck` — semantic information (register assignments, slide types, alt text)
- `LaidOutDeck` — geometric information (EMU coordinates, text flows, reading order)
- `Brand` — theme colors, fonts, layout templates

## PPTX Exporter (SS-06, ADR-001)

Library: `ooxmlsdk = "=0.6.1"`. Spike S1 confirmed 55/57 capability checks PASS.

Two workarounds are required (implemented in `slideforge-pptx`):

**W1 — Table serialization:** `GraphicData` has no typed `a_table` field. Tables
are serialized to XML string via `table.to_xml_bytes()` then placed in
`GraphicData.xml_children`. Encapsulated in `table_builder.rs`.

**W2 — Content_Types Default entries:** ooxmlsdk emits only `<Override>` entries.
ECMA-376 requires `<Default>` entries for `.rels` and `.xml` extensions.
Post-processed in `opc_postprocess.rs` (~50 lines): parse ZIP in-memory, inject
Default entries, repackage.

Key implementation notes (S1 API findings):
- All slide shapes, pictures, graphic frames go via `ShapeTreeChoice` enum variants
- Navigate parts via relationships — never hard-code part paths (they are auto-numbered)
- Import PML and DML types in separate `use` blocks to avoid namespace collision

Visual parity gate: SSIM ≥ 0.99 AND PSNR ≥ 35dB per slide vs LibreOffice renders
(ADR-002, S6). CI pipeline: PPTX → PDF via LibreOffice Still 25.8.7 → PNG via
ImageMagick 300 DPI → scikit-image comparison.

## PDF Exporter (SS-07, ADR-003)

Library stack: `pdf-writer = "=0.14"` + `krilla = "=0.6"` + `SlideTagEngine` (custom).

The SlideTagEngine is the critical component: it builds the `/StructTreeRoot`
required for PDF/UA-1. It MUST be designed with a decoupled draw/tag interface
(Feasibility Note 1):

**Draw layer:** krilla handles path fills, strokes, glyph placement, image/SVG
embedding. This layer is stable regardless of tagging approach.

**Tag layer (primary):** krilla's experimental tagged PDF hooks assign `/MCID`
values and link to structure tree elements. Available in krilla v0.6.0.

**Tag layer (fallback):** If krilla's experimental tagging fails PDF/UA-1 validation
after Phase 6 hardening (≤5 story-days effort), fall back to direct `pdf-writer`
structure tree construction. The draw layer (krilla) remains unchanged. The fallback
trigger: `veraPDF --flavour ua1` exits non-zero on a full test fixture deck.

**Fallback-of-fallback:** Typst template approach (generate `.typ` source from
LaidOutDeck via Typst's `place()` function). Escalates to human decision.

Coordinate mapping: `pdf_y = page_height_pt - (ir_y_pt + element_height_pt)`.
This is a pure function; VP-006 specifies a Kani proof for the EMU-to-PDF arithmetic.

Accessibility gate: `veraPDF --flavour ua1` in CI via Docker sidecar (`verapdf/rest:1.26.0`).

## DOCX Exporter (SS-08)

OOXML DOCX serialization via `ooxmlsdk`. The `report` register content (DI-012)
maps to document sections. Shares the OOXML element-ordering knowledge from PPTX.
Brand template provides the DOCX template via `BrandProvider::synthesize_docx()`.

## HTML/Preview Exporter (SS-09, ADR-008)

The web preview is an embedded `axum` server with WebSocket push. SVG-based
canvas rendering is required (not `<canvas>`) per S3 WCAG constraint:
a bare `<canvas>` is opaque to axe-core and screen readers (WCAG 1.1.1).

Slides render as SVG elements with ARIA attributes. The Node.js footprint is
test-harness only (`@axe-core/playwright` in `crates/slideforge-preview/tests/`).
The production binary has zero Node.js dependency.

## Format Matrix

| Format | Library | Phase | Key Constraint | Gate |
|--------|---------|-------|---------------|------|
| PPTX | ooxmlsdk 0.6.1 | 3 | SSIM ≥ 0.99, PSNR ≥ 35dB | Visual regression CI |
| DOCX | ooxmlsdk 0.6.1 | 3 | Multi-renderer round-trip | Snapshot test |
| PDF | pdf-writer + krilla | 4 | PDF/UA-1 compliant | veraPDF CI gate |
| HTML | axum + SVG | 3-4 | WCAG AA zero violations | axe-core CI gate |
| Preview | axum + WebSocket | 3 | SVG canvas (not canvas element) | axe-core CI gate |
