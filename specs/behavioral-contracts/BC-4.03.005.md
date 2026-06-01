---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-07
capability: CAP-017
lifecycle_status: active
introduced: v1.0.0
modified:
  - date: 2026-05-31
    directive: DIR-044-001
    reason: "Coordinate model corrected for krilla top-left Y-down Surface; ir_y_to_pdf_y retained as pure function but removed from draw-time call sites."
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
top-left, Y increases downward) to PDF user units. The conversion formula is:
1 PDF point = 12700 EMU (from the OOXML spec). The exporter draws onto krilla's
`Surface`, which uses a top-left, Y-down coordinate system (surface.rs:44,
page.rs:262-263). krilla applies the PDF Y-axis flip internally during
serialization. Draw-time placement therefore uses `emu_to_pt(ir_y)` directly
(top edge in Surface coords) — no application of `ir_y_to_pdf_y` at draw time.
The function `ir_y_to_pdf_y` is retained as a documented pure function and VP-006
Kani proof target, but it is not called on the draw path.

## Coordinate System Context

krilla's Surface (krilla 0.6.0) is top-left, Y-down (surface.rs:44, geom.rs:145).
krilla applies the PDF bottom-left Y-up conversion via `page_root_transform`
(`Transform::from_row(1,0,0,-1,0,h)`) as the root transform of every page surface
(page.rs:262-263). This is invisible to the caller. The exporter passes IR Y
coordinates via `emu_to_pt(ir_y)` — no second y-flip.

## Preconditions

1. An `Emu` value from `LaidOutDeck` represents a coordinate or dimension.
2. The slide dimensions are 9,144,000 × 5,143,500 EMU (standard 16:9) or as specified in the brand template.
3. All input EMU values are non-negative integers.

## Postconditions

1. `emu_to_pt(Emu(9_144_000))` = 720.0 points (slide width, 10 inches).
2. `emu_to_pt(Emu(5_143_500))` = 405.0 points (slide height, 5.625 inches).
3. An element at IR `y=0` (slide top) is drawn at Surface Y = `emu_to_pt(Emu(0))` = 0.0 (top of Surface). krilla maps this to the top of the PDF page. `ir_y_to_pdf_y` is not called at draw time.
4. An element positioned at `ir_y = slide_height − element_height` is drawn at Surface Y = `emu_to_pt(slide_height − element_height)` = `SLIDE_HEIGHT_PT − element_height_pt` (bottom of Surface). krilla maps this to the bottom of the PDF page.
5. Converted values are f32 precision; rounding error < 0.001 points.

## Invariants

1. `emu_to_pt` is a pure function with no side effects. (Kani-provable)
2. `ir_y_to_pdf_y` is a pure function with no side effects. (Kani-provable)
3. No element is positioned outside the slide canvas after conversion (0 ≤ pdf_x ≤ SLIDE_WIDTH_PT, 0 ≤ pdf_y ≤ SLIDE_HEIGHT_PT).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | EMU value = 0 (origin) | emu_to_pt(0) = 0.0 |
| EC-002 | Element at top-left corner (ir_x=0, ir_y=0) | Surface position: (0.0, 0.0) — top-left of Surface, renders at top of PDF page |
| EC-003 | Element at bottom-right corner | Surface position: (SLIDE_WIDTH_PT - elem_w_pt, SLIDE_HEIGHT_PT - elem_h_pt) |
| EC-004 | Element with zero height | ir_y_to_pdf_y(ir_y, 0) = SLIDE_HEIGHT_PT - ir_y_pt |
| EC-005 | Non-standard slide size (4:3) | Conversion still correct; SLIDE_HEIGHT_PT derived from actual brand template dimensions |

## Canonical Test Vectors

**AC-001 — `emu_to_pt` conversions (draw-time mapping, unchanged):**

| Input | Expected Output | Category |
|-------|----------------|----------|
| `emu_to_pt(Emu(9_144_000))` | 720.0 | happy-path |
| `emu_to_pt(Emu(5_143_500))` | 405.0 | happy-path |
| `emu_to_pt(Emu(12700))` | 1.0 (1 point) | happy-path |

**AC-002 — `ir_y_to_pdf_y` pure-function arithmetic (NOT draw-time placement):**

> Note: These vectors test the arithmetic correctness of `ir_y_to_pdf_y` as a pure
> function and as a VP-006 Kani proof target. This function is NOT called on the
> krilla draw path — krilla's Surface owns the PDF Y-axis flip internally
> (page.rs:262-263). The vectors are unchanged from v1.1; only their scope is
> clarified.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ir_y_to_pdf_y(Emu(0), Emu(100 * 12700))` | 305.0 (= 405.0 - 0 - 100) | happy-path (S2 unit test) |
| `ir_y_to_pdf_y(Emu(5_143_500 - 100 * 12700), Emu(100 * 12700))` | 0.0 | edge-case (bottom edge) |

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
| Stories | STORY-044 |

## Related BCs

- BC-4.03.001 — composes with (PDF/UA-1 compliance depends on correct element positioning)
- BC-4.03.002 — composes with (this function is part of the pdf-writer + krilla stack)

## Architecture Anchors

- `architecture/export-architecture.md` — EMU-to-PDF conversion (Spike S2)

## Story Anchor

STORY-044

## VP Anchors

- VP-006 (EMU-PDF coordinate mapping; bc_trace=[BC-4.03.005])
