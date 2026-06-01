---
document_type: architecture-section
section: ir-design
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# IR Design

## Two-IR Model (ADR-005, DI-009)

Two distinct intermediate representations separate semantic authoring concerns
from geometric layout concerns:

**Deck IR** — semantic, pre-layout. Output of `slideforge-eval`.
- Contains: slide type, field values, variable bindings, data-source results,
  register assignments (notes/report/detail), conditional/iteration results.
- Does NOT contain: coordinates, dimensions, font metrics, pixel positions.
- All `{{ expr }}` interpolations are resolved; `@for` loops are expanded.

**LaidOutDeck IR** — geometric, post-layout. Output of `slideforge-layout`.
- Contains: all `Deck` semantic information PLUS positioned shapes with EMU
  coordinates, computed text flows, font metrics, reading order, MCID assignments.
- Exporters consume both IRs: PPTX needs semantic Deck for placeholder inheritance;
  PDF needs LaidOutDeck for precise coordinate mapping.

## Type Constraints (DI-010, DI-011)

All IR types MUST implement `Hash + Eq + Clone` from initial declaration:
- Required for future `comemo` incremental compilation (v1.x)
- Required for Kani bounded-model checking (Phase 6)
- Adding these traits retroactively is API-breaking

Use `Arc<str>` for shared string values (not `String`) to make cloning cheap
and make `Hash + Eq` work correctly for interned strings.

## EMU Coordinate System (ADR-013)

All coordinates and dimensions in `LaidOutDeck` use integer English Metric Units:
- 1 inch = 914,400 EMU
- 1 point = 12,700 EMU (used for PDF coordinate mapping)
- Standard 16:9 slide: 9,144,000 × 5,143,500 EMU

Float-to-EMU conversion happens ONLY at DSL parse / layout boundary. All
internal computation uses `i64` EMU values. This enables:
- Exact arithmetic (no floating-point rounding)
- Kani provable arithmetic (bounded integer operations)
- `Hash + Eq` on all coordinate-bearing types

## Coordinate Mapping at Export Boundaries

PDF (krilla backend): krilla's `Surface` uses a top-left, Y-down coordinate system
(surface.rs:44, geom.rs:145). krilla applies the PDF Y-axis flip internally via
`page_root_transform` (page.rs:262-263). The exporter passes IR Y coordinates
directly: `surface_y = emu_to_pt(ir_y)`. No exporter-applied Y-flip on the draw path.
The pure function `ir_y_to_pdf_y` (formula: `page_height_pt - (ir_y_pt + element_height_pt)`)
is a VP-006 Kani proof target for raw PDF coordinate arithmetic but is NOT called on
the krilla draw path. See BC-4.03.005 v1.2 and DIR-044-001 (2026-05-31).

PPTX: EMU values pass through directly to OOXML `cx`/`cy`/`x`/`y` attributes.

## Math IR (Feasibility Note 3)

LaTeX math nodes are stored in the Deck IR as `MathNode { latex: Arc<str>, display: bool }`.
Conversion to output format (OMML for PPTX/DOCX, MathML for HTML, PDF paths for PDF)
happens at export time via the `MathRenderer` plugin surface.

**Dependency flag:** OMML generation (BC-1.10.003) depends on spike S-MATH-01.
The `slideforge-math` crate ships the LaTeX → MathML path in v1.0 Wave 1.
LaTeX → OMML is a Wave 2 dependency gated on S-MATH-01 resolution.

## SemanticRole Mapping

The `LaidOutElement.semantic_role` field maps to PDF structure element types
per ISO 32000-1 §14.8.4:

| SemanticRole | PDF Type | OOXML Type |
|-------------|---------|-----------|
| `Heading { level }` | H1-H6 | ph type=title or inline heading |
| `Paragraph` | P | body text run |
| `Figure` | Figure (requires /Alt) | p:pic with descr attr |
| `Table` | Table/TR/TH/TD | a:tbl structure |
| `Decorative` | Artifact (excluded) | cNvPr decorative="1" |
