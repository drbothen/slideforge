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
capability: CAP-023
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

# BC-3.04.001: shape: Block Declares Custom Shape with type/position/fill/text/alt

## Description

A `shape:` block inside a slide declaration adds a custom visual shape to that slide.
The shape DSL supports `type` (rect, ellipse, arrow, line, star, etc.), `position`
(x/y/width/height in inches or em), `fill` (color or gradient), `text` (inline content),
and required `alt` (accessibility description). The shape is rendered as an explicit OOXML
`<p:sp>` element positioned using integer EMU coordinates.

## Preconditions

1. The `shape:` block is nested inside a `slide <type>:` block.
2. The `shape:` block declares `type:` with a recognized shape type.
3. The `shape:` block declares `alt "..."` OR `decorative: true`.
4. `position:` specifies x, y, width, height as numbers with units (inches or em).

## Postconditions

1. The shape is added to the slide's `Deck` IR node as a `Shape` element with:
   - Type (resolved to OOXML preset geometry or custom path)
   - Position in integer EMU (converted from user units at parse time)
   - Fill color (hex RGB or gradient spec)
   - Text content (if declared; inline formatting applied)
   - Alt text (from `alt:` field) or empty alt + decorative flag
2. The shape is rendered into the .pptx as `<p:sp>` with `<p:cNvPr name="<alt text>">`
   (semantic name from alt text per accessibility rule).
3. The shape appears in all output formats that support positioned shapes (PPTX, PDF, HTML).
4. Reading order in PPTX `spTree`: shapes appear after slide placeholders, in source order.

## Invariants

1. `alt` or `decorative: true` is REQUIRED on every shape. A shape without either is a
   compile error (E-A11-001). (DI-001)
2. Position values are converted to integer EMU at parse time; no float coordinates
   in the IR. (DI-010)
3. `raw` keyword is NOT available in user .sf files — the shape DSL is the only way
   to add custom shapes. (DI-021, BC-3.04.002)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | shape: without alt and without decorative: true | E-A11-001: missing alt text on shape at <file>:<line>:<col> |
| EC-002 | shape: with negative position (off-canvas) | E-LAY-001-class warning: "Shape at position (-0.5in, 1.0in) extends outside slide canvas" |
| EC-003 | shape: with fill color that doesn't meet WCAG contrast with text | E-A11-004 contrast warning (if text is present in the shape) |
| EC-004 | shape: in DOCX export | Shape rendered as a floating `<w:drawing>` inline image (SVG rasterized at 150 DPI) |
| EC-005 | shape: type "custom" with no path: field | E-PAR-007-class: "Shape type 'custom' requires a path: field" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `shape: type rect position x 0.5in y 1.0in width 2.0in height 1.0in fill "#003766" alt "Blue rectangle highlight"` | PPTX contains `<p:sp>` with EMU coordinates and srgbClr fill; exit 0 | happy-path |
| `shape: type rect` (no alt, no decorative) | E-A11-001 compile error | error |
| `shape: type ellipse decorative: true fill "#FF6F00"` | PPTX `<p:sp>` with empty `descr=""` attribute and PDF Artifact tag; exit 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Shape position in PPTX XML matches EMU conversion of declared user-unit values | unit test (known input → known EMU assertion) |
| VP-TBD | Shape without alt or decorative always produces E-A11-001 | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-023 ("Structured Shape DSL") per capabilities.md §CAP-023 |
| Capability Anchor Justification | CAP-023 ("Structured Shape DSL") per capabilities.md §CAP-023 — "shape: blocks with type, position, size, fill, text, and required alt text" is the verbatim description of CAP-023 |
| L2 Domain Invariants | DI-001 (alt text required on visual elements), DI-010 (integer EMU for all coordinates), DI-021 (raw keyword rejected) |
| Architecture Module | slideforge-layout crate — shape block parsing; slideforge-pptx crate — shape to sp element (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.04.002 — depends on (raw keyword rejection is the complementary contract to shape DSL availability)
- BC-5.01.001 — depends on (alt text enforcement applies to shape: blocks)

## Architecture Anchors

- `architecture/crate-architecture.md` — shape block parsing and EMU conversion

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
