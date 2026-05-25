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

# BC-4.03.005: PDF Coordinate Mapping — EMU to PDF User Units with Y-Axis Flip

## Description

The PDF exporter converts EMU coordinates from the `LaidOutDeck` IR (origin at
top-left, Y increases downward) to PDF user units (origin at bottom-left, Y
increases upward). The conversion formula is: 1 PDF point = 12700 EMU (from the
OOXML spec). The Y-axis is flipped: `pdf_y = slide_height_pt − (ir_y_pt + element_height_pt)`.
This pure mathematical conversion is provably correct via Kani.

## Preconditions

1. An `Emu` value from `LaidOutDeck` represents a coordinate or dimension.
2. The slide dimensions are 9,144,000 × 5,143,500 EMU (standard 16:9) or as specified in the brand template.
3. All input EMU values are non-negative integers.

## Postconditions

1. `emu_to_pt(Emu(9_144_000))` = 720.0 points (slide width, 10 inches).
2. `emu_to_pt(Emu(5_143_500))` = 405.0 points (slide height, 5.625 inches).
3. `ir_y_to_pdf_y(Emu(0), Emu(element_h))` = `SLIDE_HEIGHT_PT − element_height_pt` (top-left origin maps to top-left position in PDF).
4. An element positioned at `ir_y = slide_height − element_height` maps to `pdf_y = 0.0` (bottom edge).
5. Converted values are f32 precision; rounding error < 0.001 points.

## Invariants

1. `emu_to_pt` is a pure function with no side effects. (Kani-provable)
2. `ir_y_to_pdf_y` is a pure function with no side effects. (Kani-provable)
3. No element is positioned outside the slide canvas after conversion (0 ≤ pdf_x ≤ SLIDE_WIDTH_PT, 0 ≤ pdf_y ≤ SLIDE_HEIGHT_PT).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | EMU value = 0 (origin) | emu_to_pt(0) = 0.0 |
| EC-002 | Element at top-left corner (ir_x=0, ir_y=0) | PDF position: (0, SLIDE_HEIGHT_PT - element_height_pt) |
| EC-003 | Element at bottom-right corner | PDF position: (SLIDE_WIDTH_PT - element_width_pt, 0) |
| EC-004 | Element with zero height | ir_y_to_pdf_y(ir_y, 0) = SLIDE_HEIGHT_PT - ir_y_pt |
| EC-005 | Non-standard slide size (4:3) | Conversion still correct; SLIDE_HEIGHT_PT derived from actual brand template dimensions |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `emu_to_pt(Emu(9_144_000))` | 720.0 | happy-path |
| `emu_to_pt(Emu(5_143_500))` | 405.0 | happy-path |
| `ir_y_to_pdf_y(Emu(0), Emu(100 * 12700))` | 305.0 (= 405.0 - 0 - 100) | happy-path (S2 unit test) |
| `ir_y_to_pdf_y(Emu(5_143_500 - 100 * 12700), Emu(100 * 12700))` | 0.0 | edge-case (bottom edge) |
| `emu_to_pt(Emu(12700))` | 1.0 (1 point) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `emu_to_pt` is exact to within 0.001 for standard slide dimensions | kani (bounded EMU range: 0..=9_144_000) |
| VP-TBD | `ir_y_to_pdf_y` is the precise inverse of IR Y-origin convention | kani proof: forall emu_y, emu_h: ir_y_to_pdf_y(emu_y, emu_h) >= 0.0 when within slide bounds |
| VP-TBD | No element mapped outside slide canvas bounds (0..slide_pt) | kani |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 |
| Capability Anchor Justification | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 — correct coordinate mapping is a prerequisite for the layout-accurate PDF output that CAP-017 requires |
| L2 Domain Invariants | DI-010 (integer EMU for all coordinates) |
| Architecture Module | slideforge-pdf crate — coordinate mapping (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.03.001 — composes with (PDF/UA-1 compliance depends on correct element positioning)
- BC-4.03.002 — composes with (this function is part of the pdf-writer + krilla stack)

## Architecture Anchors

- `architecture/export-architecture.md` — EMU-to-PDF conversion (Spike S2)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
