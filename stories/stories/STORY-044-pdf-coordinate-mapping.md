---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-044
title: "PDF: EMU-to-PDF Coordinate Mapping + Y-Axis Flip"
epic: EPIC-13
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.03.005, BC-4.03.002]
verification_properties: [VP-006]
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-pdf
target_module: slideforge-pdf
subsystems: [SS-07]
depends_on:
  - STORY-043
blocks:
  - STORY-045
  - STORY-049
  - STORY-050
estimated_days: 2
---

# STORY-044: PDF — EMU-to-PDF Coordinate Mapping + Y-Axis Flip

## Subsystem Anchor Justification

SS-07 (PDF Export) owns coordinate mapping because it is a PDF-specific concern:
the origin and axis orientation differ between the PPTX IR (top-left origin,
Y increases downward) and PDF (bottom-left origin, Y increases upward). No other
subsystem needs to perform this inversion. Per ARCH-INDEX, SS-07 is the
"Effectful shell. pdf-writer + krilla. Phase 4."

## Dependency Anchor Justifications

- Depends on STORY-043: `emu_to_pt()` and `ir_y_to_pdf_y()` are pure functions
  inside `slideforge-pdf`. They require the crate foundation (Cargo.toml, error types,
  lib.rs) established in STORY-043. Without STORY-043, the crate does not exist.
- Blocks STORY-045: The PDF/UA-1 tagging and element positioning in the structure tree
  require correct coordinate mapping. veraPDF validates element positioning against
  page dimensions.

## Summary

Implement the two pure coordinate-mapping functions inside `slideforge-pdf`:

1. `emu_to_pt(emu: Emu) -> f32` — converts an EMU value to PDF user units (points).
   Formula: `emu.0 as f32 / 12700.0`. One inch = 914,400 EMU = 72 PDF points,
   therefore 1 PDF point = 914,400 / 72 = 12,700 EMU.

2. `ir_y_to_pdf_y(ir_y: Emu, element_h: Emu, slide_h: Emu) -> f32` — a pure
   PDF-coordinate-flip function. Formula:
   `emu_to_pt(slide_h) - emu_to_pt(ir_y) - emu_to_pt(element_h)`.
   This function computes the raw PDF bottom-left Y coordinate. It is retained as a
   documented pure function and VP-006 Kani proof target, but it is **NOT** called on
   the krilla draw path (see coordinate system context below).

Both functions are pure (no side effects) and Kani-amenable (VP-006 targets them in
Phase 6).

**Coordinate System Context (BC-4.03.005 v1.2, DIR-044-001 2026-05-31):**
krilla's `Surface` (krilla 0.6.0) uses a top-left, Y-down coordinate system
(`surface.rs:44`, `geom.rs:145`). krilla applies the PDF Y-axis flip internally via
`page_root_transform` (`Transform::from_row(1,0,0,-1,0,h)`) as the root transform of
every page surface (`page.rs:262-263`). This is invisible to the caller. Draw-time
placement in `PdfExporter::export()` therefore passes `emu_to_pt(ir_y)` (top edge,
Y-down) directly to the Surface — **`ir_y_to_pdf_y` is NOT applied at draw time**.
`ir_y_to_pdf_y` is retained in `coords.rs` as a pure function and VP-006 Kani target.

Standard slide dimensions (from BC-4.03.005):
- Width: 9,144,000 EMU = 720.0 PDF points (10 inches)
- Height: 5,143,500 EMU = 405.0 PDF points (5.625 inches)

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.03.005 | PDF Coordinate Mapping: EMU to PDF User Units with Y-Axis Flip | AC-001 through AC-008 |
| BC-4.03.002 | PDF Produced via pdf-writer + krilla + SlideTagEngine (No Chrome/Headless) | AC-009 (font subsetting, re-scoped from STORY-043 per scope-directive Decision 2) |

## Acceptance Criteria

### AC-001: emu_to_pt canonical test vectors pass
(traces to BC-4.03.005 postcondition 1 and 2)

The following assertions pass as unit tests:
- `emu_to_pt(Emu(9_144_000))` == `720.0_f32`  (slide width, 10 inches)
- `emu_to_pt(Emu(5_143_500))` == `405.0_f32`  (slide height, 5.625 inches)
- `emu_to_pt(Emu(12700))`     == `1.0_f32`    (1 PDF point)
- `emu_to_pt(Emu(0))`         == `0.0_f32`    (origin)

### AC-002: ir_y_to_pdf_y canonical test vectors pass
(traces to BC-4.03.005 postcondition 3 and 4)

The following assertions pass as unit tests (using standard 16:9 slide height
5,143,500 EMU = 405.0pt):
- `ir_y_to_pdf_y(Emu(0), Emu(100 * 12700), SLIDE_H_EMU)` == `305.0_f32`
  (top-left element of height 100pt maps to pdf_y = 405 - 0 - 100 = 305)
- `ir_y_to_pdf_y(Emu(5_143_500 - 100 * 12700), Emu(100 * 12700), SLIDE_H_EMU)`
  == `0.0_f32`  (bottom-edge element)
- `ir_y_to_pdf_y(Emu(0), Emu(0), SLIDE_H_EMU)` == `405.0_f32`
  (zero-height element at top = top of page in PDF)

### AC-003: Rounding error less than 0.001 points
(traces to BC-4.03.005 postcondition 5)

For all standard slide dimension values (0..=9_144_000 EMU), the absolute rounding
error of `emu_to_pt()` relative to the exact rational value is less than 0.001 PDF
points. This is verified by a parameterized unit test against a reference computed
with `f64` precision.

### AC-004: emu_to_pt is a pure function
(traces to BC-4.03.005 invariant 1)

`emu_to_pt` takes only `emu: Emu` as input and returns `f32`. No mutable state, no
I/O, no thread-local access. The function is declared `#[inline]` and `const`-
compatible (no floating-point `const fn` in stable Rust — but it must be a free
function with no hidden dependencies).

### AC-005: ir_y_to_pdf_y is a pure function
(traces to BC-4.03.005 invariant 2)

`ir_y_to_pdf_y` takes only `(ir_y: Emu, element_h: Emu, slide_h: Emu)` and returns
`f32`. No mutable state, no I/O. Declared `#[inline]`.

### AC-006: No element positioned outside slide canvas
(traces to BC-4.03.005 invariant 3)

For all elements in `LaidOutSlide` whose `(cx, cy, w, h)` fields satisfy
`0 <= cx <= slide_width - w` and `0 <= cy <= slide_height - h` (valid layout
invariant from STORY-026), the converted Surface coordinates satisfy:
- `0.0 <= surface_x <= SLIDE_WIDTH_PT`
- `0.0 <= surface_y <= SLIDE_HEIGHT_PT`

where `surface_x = emu_to_pt(cx)` and `surface_y = emu_to_pt(cy) + emu_to_pt(h) * 0.8`
(text baseline; Y-down from box top). Draw-time Y uses `emu_to_pt(ir_y)` directly
(top-left, krilla owns the PDF flip) — `ir_y_to_pdf_y` is not applied at draw time.
The bounding box `[0.0, 0.0, 720.0, 405.0]` assertion remains valid for all Surface
coordinates since both the `emu_to_pt` path and the canonical `ir_y_to_pdf_y` range
are constrained to `[0, SLIDE_HEIGHT_PT]`.

This is verified by an integration test that renders a fixture deck and asserts all
element bounding boxes are within `[0.0, 0.0, 720.0, 405.0]`.

### AC-007: Non-standard slide size (4:3) uses brand template dimensions
(traces to BC-4.03.005 edge case EC-005)

When the brand template specifies a 4:3 slide (7,315,200 × 5,486,400 EMU,
= 576pt × 432pt), `ir_y_to_pdf_y()` receives the brand's actual slide height as
`slide_h`, not the hard-coded 16:9 constant. A unit test verifies 4:3 coordinate
mapping with the expected output values.

### AC-008: Zero-height element edge case
(traces to BC-4.03.005 edge case EC-004)

`ir_y_to_pdf_y(ir_y, Emu(0), SLIDE_H_EMU)` ==
`SLIDE_HEIGHT_PT - emu_to_pt(ir_y)`. Verified as a unit test.

### AC-009: Font subsetting verified — krilla internal, no system tooling
(traces to BC-4.03.002 invariant 3)

When `PdfExporter::export()` draws text elements via krilla's Surface/text API in
STORY-044, krilla internally subsets fonts via its `subsetter` transitive dependency
(krilla 0.6.0 declares `^0.2.3`; Cargo.lock resolves to 0.2.4).

A unit test verifies: for a deck using only ASCII glyphs from a large Unicode font
(e.g., a font with full CJK coverage), the embedded font program in the output PDF
is smaller than the unsubsetted font file. Assertion: `embedded_font_bytes.len() <
full_font_file_bytes.len()`.

No `subsetter::subset(...)` call appears in `slideforge-pdf` source code — subsetting
is entirely internal to krilla. No system font tooling (`fonttools`, `pyftsubset`,
`hb-subset`) is invoked.

This AC was re-scoped from STORY-043 AC-004 (which could not be verified without text
drawing). Scope rationale: `.factory/cycles/STORY-043/scope-directive.md` Decision 2.
BC-4.03.002 invariant 3 is unchanged — only the test vehicle moved to STORY-044
where glyphs are actually drawn onto the Surface.

## Tasks

- [ ] Create `crates/slideforge-pdf/src/coords.rs`:
  - `pub fn emu_to_pt(emu: Emu) -> f32`
  - `pub fn ir_y_to_pdf_y(ir_y: Emu, element_h: Emu, slide_h: Emu) -> f32`
  - `pub const SLIDE_WIDTH_EMU: Emu = Emu(9_144_000);`
  - `pub const SLIDE_HEIGHT_EMU: Emu = Emu(5_143_500);`
- [ ] Export `coords` module from `src/lib.rs`
- [ ] Update `PdfExporter::export()` (from STORY-043) to use `emu_to_pt()` for all
  element placement on the krilla draw path — `emu_to_pt(ir_y)` for Surface top edge
  (Y-down), `emu_to_pt(bbox.y) + emu_to_pt(bbox.height) * 0.8` for text baseline.
  `ir_y_to_pdf_y()` is NOT called from the draw path; remove any ad-hoc arithmetic.
  Update `ir_y_to_pdf_y` docstring in `coords.rs` to clarify it is not called on the
  krilla draw path (krilla owns the y-flip via `page_root_transform` — DIR-044-001)
- [ ] Write unit tests in `src/coords.rs` `#[cfg(test)]` block:
  - All canonical test vectors from AC-001 and AC-002 (pure-function arithmetic)
  - Rounding error < 0.001 for range 0..=9_144_000 (sampled at 1000-point intervals)
  - 4:3 slide size test (AC-007)
  - Zero-height element test (AC-008)
- [ ] Write vertical-placement regression tests in `src/exporter.rs` `#[cfg(test)]` block
  (required gate per DIR-044-001 Section 4 — these are MANDATORY before GREEN):
  - `test_vertical_placement_title_at_top`: title at ir_y=0 → Surface baseline_y = 57.6,
    assert `baseline_y < SLIDE_HEIGHT_PT / 2.0` (catches mirror bug if `ir_y_to_pdf_y`
    is erroneously applied at draw time: buggy value = 390.6, assert fails)
  - `test_vertical_placement_footer_at_bottom`: footer at slide bottom → Surface
    baseline_y ≈ 397.8, assert `baseline_y > SLIDE_HEIGHT_PT / 2.0`
- [ ] Write proptest in `tests/coords_proptest.rs`:
  - For all valid `(ir_y, element_h, slide_h)` where `ir_y + element_h <= slide_h`,
    assert `ir_y_to_pdf_y() >= 0.0` (tests pure-function range; not draw-time behavior)

## Previous Story Intelligence

STORY-043 established `crates/slideforge-pdf/src/lib.rs`, `error.rs`, and the
`PdfExporter` struct. This story adds `coords.rs` and wires it into the exporter.

Key invariant from STORY-043: all element placement in `PdfExporter::export()` was
written with TODO comments marking where coordinate conversion goes. This story
fills those TODOs by introducing `emu_to_pt()` and `ir_y_to_pdf_y()` and calling
them from the element-placement loop.

**Coordinate model correction (2026-05-31):** BC-4.03.005 was bumped to v1.2 via
DIR-044-001. krilla owns the PDF Y-axis flip internally; the draw path uses top-left
`emu_to_pt(ir_y)` directly on the Surface — `ir_y_to_pdf_y` is NOT applied at draw
time. The prior model (applying `ir_y_to_pdf_y` at draw time) caused a vertical-mirror
bug and had propagated to four architecture artifacts, all now corrected. See
`.factory/cycles/STORY-044/coord-model-directive.md` for full evidence and remediation
checklist.

## Architecture Compliance Rules

1. **Pure functions mandatory (BC-4.03.005 invariants 1 and 2)**: `emu_to_pt` and
   `ir_y_to_pdf_y` must have no side effects. No logging, no `tracing::` calls, no
   mutation. They are candidates for Kani proof (VP-006, Phase 6).
2. **Centralized coordinate logic**: All EMU-to-pt conversions in `slideforge-pdf`
   go through `coords::emu_to_pt()`. Ad-hoc `emu / 12700.0` inline arithmetic is
   forbidden — it prevents the Kani proof from covering all conversion sites. On the
   krilla draw path, element Y placement uses `emu_to_pt(ir_y)` for the Surface top
   edge (Y-down); `ir_y_to_pdf_y` is NOT called at draw time (krilla owns the
   PDF-space flip via `page_root_transform` — DIR-044-001).
3. **Brand-dynamic slide dimensions**: `SLIDE_WIDTH_EMU`/`SLIDE_HEIGHT_EMU` constants
   are defaults. The `ir_y_to_pdf_y()` function accepts `slide_h` as a parameter;
   the caller passes `brand.slide_height_emu` at runtime when invoking the pure
   function for non-draw-time purposes (e.g., tests, future raw-PDF backends).

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-pdf` (self, STORY-043) | workspace | Extends coordinate module |
| `slideforge-types` | workspace | `Emu` newtype |
| `proptest` | `=1.5.0` | Proptest for coordinate bounds check |

Note: No additional crate dependencies required. This story is pure math.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pdf/src/coords.rs` | Create | EMU/PDF coordinate functions + constants |
| `crates/slideforge-pdf/src/lib.rs` | Modify | `pub mod coords;` export |
| `crates/slideforge-pdf/src/exporter.rs` | Modify | Replace ad-hoc math with `coords::` calls |
| `crates/slideforge-pdf/tests/coords_proptest.rs` | Create | Proptest bounds check |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| BC-4.03.005 | ~1,200 |
| BC-4.03.002 (invariant 3 — font subsetting, AC-009) | ~400 |
| STORY-043 exporter.rs (context for wiring) | ~1,500 |
| Emu type from slideforge-types | ~400 |
| Test files to write | ~1,800 |
| **Total** | **~7,800** |

Context budget: ~7% of a 100k-token context window. Well within limit.

## Test Strategy

- **Unit tests**: All AC-001 and AC-002 canonical vectors as `assert_eq!` tests in
  `coords.rs` `#[cfg(test)]`. Rounding error assertion (AC-003). 4:3 slide test.
- **Proptest**: For random `(ir_y, element_h, slide_h)` tuples where
  `ir_y + element_h <= slide_h`, assert `ir_y_to_pdf_y() >= 0.0` (in-bounds
  guarantee from BC-4.03.005 invariant 3).
- **Phase 6**: VP-006 provides Kani proofs for `emu_to_pt` and `ir_y_to_pdf_y`.
  The pure-function structure established here is required for Kani to model-check.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `emu_to_pt(Emu(0))` | Returns `0.0_f32` |
| EC-002 | Element at top-left corner (ir_x=0, ir_y=0, element_h=100pt) | Surface position: `(0.0, 0.0)` — top-left of Surface; krilla maps this to top of PDF page. `ir_y_to_pdf_y` pure-function value for the same input is 305.0 (tests the arithmetic, not draw-time) |
| EC-003 | Element at bottom-right corner (ir_y = SLIDE_HEIGHT_PT - elem_h_pt) | Surface position: `(SLIDE_WIDTH_PT - elem_w_pt, SLIDE_HEIGHT_PT - elem_h_pt)` — bottom-right of Surface; krilla maps this to bottom of PDF page |
| EC-004 | Zero-height element | pdf_y = SLIDE_HEIGHT_PT - ir_y_pt |
| EC-005 | 4:3 slide brand template (576pt × 432pt) | Correct mapping using brand slide_h |

## Forbidden Dependencies

No new dependencies beyond those in STORY-043. This story adds no Cargo.toml
changes — it is pure math implemented within the existing `slideforge-pdf` crate.
