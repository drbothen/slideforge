---
document_type: binding-directive
directive_id: DIR-044-001
issued_by: architect
issued_date: 2026-05-31
status: binding
targets: [implementer, product-owner]
story: STORY-044
bc: BC-4.03.005
vp: VP-006
---

# Binding Directive DIR-044-001: Coordinate Model Correction for krilla Backend

## Status

BINDING. Overrides the draw-time coordinate model in STORY-044 spec and BC-4.03.005
wherever those documents describe applying `ir_y_to_pdf_y` at draw time. This
directive takes precedence under the Source-of-Truth Precedence rule (architecture
correction trumps spec prose when the spec is factually wrong about the chosen library).

---

## 1. Confirmed Coordinate Model — Exact Corrected Draw-Time Formulas

### krilla Source Evidence

Three lines of krilla 0.6.0 source establish the coordinate model unambiguously:

- `surface.rs:44` — doc comment: "The origin of the coordinate axis is in the
  top-left corner."
- `geom.rs:145` — doc comment: "Where the coordinate system is Y-down" (for
  `Quadrilateral`, which documents the shared coordinate convention).
- `page.rs:262-263` — `page_root_transform`:
  ```
  Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, height)
  ```
  This matrix `[a=1, b=0, c=0, d=-1, e=0, f=height]` is applied as the root
  transform of every `ContentBuilder` before any user drawing call (page.rs:239-241,
  via `Surface::new`). It is the standard PDF y-up-to-y-down flip: it maps Surface
  Y-down coordinates to PDF Y-up coordinates during serialization. The Surface API
  therefore accepts top-left, Y-down coordinates from the caller and flips internally.

**Conclusion:** `krilla::Surface` is a top-left, Y-down drawing surface. The PDF
Y-axis flip is applied exactly once, inside krilla, unconditionally for every page.
Callers must NOT apply a second y-flip before passing coordinates to `Surface`.

### Correct Draw-Time Formulas

For an IR bounding box `(bbox.x, bbox.y, bbox.width, bbox.height)` in EMU, where
`bbox.y` is the top edge measured downward from the slide top (same origin as
the IR and as krilla's Surface):

**For `draw_text` (text baseline):**

```
surface_x  = emu_to_pt(bbox.x)
surface_y  = emu_to_pt(bbox.y) + emu_to_pt(bbox.height) * 0.8
```

- `emu_to_pt(bbox.y)` is the top edge of the bounding box in Surface coordinates
  (Y-down, top-left origin — matches IR exactly).
- Adding `element_h_pt * 0.8` moves DOWN from the box top to the text baseline
  (80% of box height from top = 20% descender allowance below baseline). Y is
  increasing downward in Surface space, so this addition is correct.
- No `ir_y_to_pdf_y` call at draw time.
- `Point::from_xy(surface_x, surface_y)` is passed directly to `surface.draw_text`.

**For `push_transform` (SVG / diagram placement):**

```
surface_x  = emu_to_pt(bbox.x)
surface_y  = emu_to_pt(bbox.y)
Transform::from_translate(surface_x, surface_y)
```

- The SVG top-left corner lands at the Surface point `(surface_x, surface_y)`.
  Because Surface is Y-down, `bbox.y=0` correctly places content at the top of
  the page. No `ir_y_to_pdf_y` call at draw time.

**Verification of correctness:**

A title frame at `ir_y=0` (slide top): `emu_to_pt(Emu(0)) = 0.0`. The Surface
point `y=0` is the top edge of the page. krilla's internal root transform
`[1,0,0,-1,0,h]` maps Surface `y=0` to PDF `y=h` (top of page in PDF Y-up space).
This is correct. Under the current (buggy) code, `ir_y_to_pdf_y(Emu(0), h, slide_h)`
= `405 - 0 - h_pt` ≈ large positive number near the bottom of the Surface, which
the root transform then maps near PDF `y=0` (bottom of page). That is the mirror bug.

### Summary Table

| Property | Current (buggy) | Correct |
|----------|-----------------|---------|
| Text X | `emu_to_pt(bbox.x)` | `emu_to_pt(bbox.x)` (unchanged) |
| Text Y (box bottom) | `ir_y_to_pdf_y(bbox.y, bbox.height, slide_h)` | (eliminated) |
| Text baseline Y | `box_bottom_pdf_y + elem_h_pt * 0.8` | `emu_to_pt(bbox.y) + elem_h_pt * 0.8` |
| SVG translate X | `emu_to_pt(bbox.x)` | `emu_to_pt(bbox.x)` (unchanged) |
| SVG translate Y | `ir_y_to_pdf_y(bbox.y, bbox.height, slide_h)` | `emu_to_pt(bbox.y)` |

All draw-time paths: `draw_text_at_bbox` (exporter.rs:459-500), `FrameContent::Diagram`
branch (exporter.rs:428-434), and `FrameContent::ErrorSlidePlaceholder` branch
(exporter.rs:436-443) must be corrected.

---

## 2. Fate of `ir_y_to_pdf_y`

**Decision: RETAIN the function; remove it from all krilla draw-time call sites.**

Rationale:

- `ir_y_to_pdf_y` is a correct pure mathematical function. Its formula
  `slide_h_pt - ir_y_pt - element_h_pt` is valid arithmetic. It is simply
  wrong to apply it when the target API (krilla Surface) is not a raw PDF
  bottom-left writer.
- VP-006 targets `ir_y_to_pdf_y` for a Kani proof in Phase 6. The property
  statement in VP-006 ("for valid `(ir_y, element_h, slide_h)`, the output is
  in `[0.0, SLIDE_HEIGHT_PT]`") describes the function's mathematical range,
  which remains correct regardless of whether the function is called at draw time.
  Removing the function would require retiring VP-006; retaining it costs nothing.
- AC-001 through AC-008 test vectors in `coords.rs` for `ir_y_to_pdf_y` remain
  valid as tests of the pure function's arithmetic. They should NOT be deleted.
- The function's docstring MUST be updated to state clearly that it computes the
  PDF-space Y coordinate under a raw bottom-left PDF coordinate system, and that
  the krilla draw path does NOT call it (krilla owns the y-flip internally).

**What the draw path calls:**

```rust
// CORRECT draw path (no ir_y_to_pdf_y at draw time):
let surface_x = emu_to_pt(bbox.x);
let surface_y = emu_to_pt(bbox.y);                     // top edge, Y-down
let baseline_y = surface_y + emu_to_pt(bbox.height) * 0.8;  // down from top
surface.draw_text(Point::from_xy(surface_x, baseline_y), ...);

// SVG:
let transform = Transform::from_translate(emu_to_pt(bbox.x), emu_to_pt(bbox.y));
```

`ir_y_to_pdf_y` is NOT called anywhere in `exporter.rs`. It remains compiled
and tested in `coords.rs` as a pure-function / VP-006 Kani target.

---

## 3. BC-4.03.005 Amendment (for Product Owner)

### What Must Change

BC-4.03.005 version 1.1 describes the coordinate model as if the exporter writes
raw PDF bottom-left coordinates and applies the Y-flip at draw time. That model
is wrong for krilla 0.6.0, which exposes a top-left Y-down Surface API and applies
the PDF flip internally. The BC must be amended to reflect reality.

**Exact changes the PO must make:**

1. **Description paragraph** — replace the sentence:

   > "The Y-axis is flipped: `pdf_y = slide_height_pt − (ir_y_pt + element_height_pt)`."

   with:

   > "The exporter draws onto krilla's `Surface`, which uses a top-left, Y-down
   > coordinate system (surface.rs:44, page.rs:262-263). krilla applies the
   > PDF Y-axis flip internally during serialization. Draw-time placement therefore
   > uses `emu_to_pt(ir_y)` directly (top edge in Surface coords) — no application
   > of `ir_y_to_pdf_y` at draw time. The function `ir_y_to_pdf_y` is retained as
   > a documented pure function and VP-006 Kani proof target, but it is not called
   > on the draw path."

2. **Postcondition 3** — replace:

   > "`ir_y_to_pdf_y(Emu(0), Emu(element_h))` = `SLIDE_HEIGHT_PT − element_height_pt`
   > (top-left origin maps to top-left position in PDF)"

   with:

   > "An element at IR `y=0` (slide top) is drawn at Surface Y = `emu_to_pt(Emu(0))` = 0.0
   > (top of Surface). krilla maps this to the top of the PDF page. `ir_y_to_pdf_y`
   > is not called at draw time."

3. **Edge Case EC-002** — replace:

   > "Element at top-left corner (ir_x=0, ir_y=0) | PDF position: (0, SLIDE_HEIGHT_PT - element_height_pt)"

   with:

   > "Element at top-left corner (ir_x=0, ir_y=0) | Surface position: (0.0, 0.0) — top-left of Surface, renders at top of PDF page"

4. **Edge Case EC-003** — replace:

   > "Element at bottom-right corner | PDF position: (SLIDE_WIDTH_PT - element_width_pt, 0)"

   with:

   > "Element at bottom-right corner | Surface position: (SLIDE_WIDTH_PT - elem_w_pt, SLIDE_HEIGHT_PT - elem_h_pt)"

5. **Canonical Test Vectors table** — the `ir_y_to_pdf_y` rows remain valid as tests
   of the pure function arithmetic. Add a column header or note: "These test the pure
   function's arithmetic, not draw-time placement (krilla handles draw-time flip)."

6. **Add a new subsection: "Coordinate System Context"** after the Description:

   > **Coordinate System Context**
   >
   > krilla's Surface (krilla 0.6.0) is top-left, Y-down (surface.rs:44, geom.rs:145).
   > krilla applies the PDF bottom-left Y-up conversion via `page_root_transform`
   > (`Transform::from_row(1,0,0,-1,0,h)`) as the root transform of every page surface
   > (page.rs:262-263). This is invisible to the caller. The exporter passes IR Y
   > coordinates via `emu_to_pt(ir_y)` — no second y-flip.

### AC-002 Canonical Vectors: Do They Change?

**No.** The AC-002 test vectors (e.g., `ir_y_to_pdf_y(Emu(0), Emu(100*12700), SLIDE_HEIGHT_EMU) == 305.0`)
test the arithmetic of `ir_y_to_pdf_y` as a pure function. That arithmetic is
unchanged. The tests in `coords.rs` stay exactly as written. What changes is the
docstring of `ir_y_to_pdf_y` to clarify that this function is NOT called on the
draw path and that its output represents a raw PDF bottom-left coordinate (useful
if a future exporter targets a raw PDF writer rather than krilla's Surface).

### Version Bump

Yes. BC-4.03.005 must be bumped from version `"1.1"` to `"1.2"` with `modified`
frontmatter entry citing this directive (DIR-044-001, 2026-05-31) and the reason:
"Coordinate model corrected for krilla top-left Y-down Surface; `ir_y_to_pdf_y`
retained as pure function but removed from draw-time call sites."

---

## 4. Mandatory Vertical-Placement Regression Test

The existing `test_f044_004_text_frame_draws_font_resource_in_export` and the
AC-006 canvas-bounds test do NOT catch the mirror bug. They assert:
- A `/Font` resource exists in the PDF (operator presence only).
- Computed coordinates are within `[0.0, SLIDE_HEIGHT_PT]` using the
  `ir_y_to_pdf_y` formula — which vacuously passes even when the formula is wrong,
  because `ir_y_to_pdf_y` always returns a value in `[0, SLIDE_HEIGHT_PT]`.

### Required Test: `test_vertical_placement_title_at_top`

The implementer MUST add the following test to `exporter.rs`'s `#[cfg(test)]` block.
This is a MANDATORY gate before the story can be declared GREEN.

**Test specification:**

Name: `test_vertical_placement_title_at_top`

Setup: compute the draw-time Surface Y for a title frame at `ir_y = Emu(0)`
(slide top), with `element_h = Emu(72 * 12_700)` (72pt = 1 inch, representative title height).

Assert:

```
// Surface Y of the top edge of the box:
let surface_top_y = emu_to_pt(Emu(0));          // = 0.0
// Surface Y of the baseline (80% down from box top):
let baseline_y = surface_top_y + emu_to_pt(Emu(72 * 12_700)) * 0.8;  // = 57.6

// The baseline must be NEAR THE TOP of the page in Surface coords,
// i.e., a SMALL positive Y value (top = 0, bottom = SLIDE_HEIGHT_PT = 405).
assert!(
    baseline_y < SLIDE_HEIGHT_PT / 2.0,
    "title baseline at ir_y=0 must be in the top half of the Surface \
     (Surface-Y < 202.5); got {baseline_y:.3}. A large value (> 202.5) \
     indicates the mirror bug: ir_y_to_pdf_y is being applied at draw time."
);
// The baseline must be >= 0 (on the page).
assert!(
    baseline_y >= 0.0,
    "title baseline must be >= 0.0; got {baseline_y:.3}"
);
```

This test FAILS under the current buggy code. With `ir_y_to_pdf_y`:
`box_bottom_pdf_y = 405 - 0 - 72 = 333`; `baseline_y = 333 + 72*0.8 = 390.6`.
The assert `390.6 < 202.5` is false — the test catches the bug.

With the correct code: `surface_top_y = 0.0`; `baseline_y = 0 + 57.6 = 57.6`.
The assert `57.6 < 202.5` is true — the test passes.

**Complementary test for bottom-of-page element:**

Name: `test_vertical_placement_footer_at_bottom`

Setup: frame at `ir_y = Emu(SLIDE_HEIGHT_EMU.0 - 36 * 12_700)` (36pt footer, 0.5 inch
from slide bottom), `element_h = Emu(36 * 12_700)`.

```
let surface_top_y = emu_to_pt(ir_y);  // = 405 - 36 = 369.0
let baseline_y = surface_top_y + emu_to_pt(element_h) * 0.8;  // = 369 + 28.8 = 397.8

assert!(
    baseline_y > SLIDE_HEIGHT_PT / 2.0,
    "footer baseline at bottom of slide must be in the bottom half of the Surface \
     (Surface-Y > 202.5); got {baseline_y:.3}."
);
assert!(
    baseline_y <= SLIDE_HEIGHT_PT + 0.001,
    "footer baseline must be <= SLIDE_HEIGHT_PT ({SLIDE_HEIGHT_PT}); got {baseline_y:.3}"
);
```

These two tests together create a directional constraint: a frame at the top renders
near Surface-Y=0; a frame at the bottom renders near Surface-Y=405. This is the
minimum test specification that prevents the mirror bug from silently regressing.

### Why This Test Cannot Be Skipped

The AC-006 canvas-bounds test (`test_bc_4_03_005_no_element_outside_canvas_after_conversion`)
only checks that computed Y values are in `[0, SLIDE_HEIGHT_PT]`. Both the buggy
formula and the correct formula satisfy that range. A directional / ordering assertion
is required to distinguish top-of-page from bottom-of-page. Without these tests, a
future code change could reintroduce the mirror bug and no automated check would catch it.

---

## Implementer Checklist

- [ ] Remove all `ir_y_to_pdf_y(...)` calls from `exporter.rs` draw paths (lines 433, 442, 487-489).
- [ ] Replace with `emu_to_pt(bbox.y)` for Surface-Y top edge.
- [ ] Recompute text baseline as `emu_to_pt(bbox.y) + emu_to_pt(bbox.height) * 0.8`.
- [ ] Update `ir_y_to_pdf_y` docstring in `coords.rs` to clarify it is not called on the krilla draw path.
- [ ] Add `test_vertical_placement_title_at_top` test — this must FAIL before the fix and PASS after.
- [ ] Add `test_vertical_placement_footer_at_bottom` test.
- [ ] Run `cargo nextest run -p slideforge-pdf --no-fail-fast` to confirm both placement tests pass and no regressions.
- [ ] Confirm AC-002 canonical vector tests still pass (they should; `ir_y_to_pdf_y` arithmetic is unchanged).

## Product Owner Checklist

- [ ] Amend BC-4.03.005 per Section 3 above.
- [ ] Bump version to `"1.2"`, add `modified` frontmatter entry for DIR-044-001.
- [ ] VP-006 text: add a clarifying note that the Kani proof targets the function's
      arithmetic correctness (range invariant), not draw-time application.
      VP-006 status remains `draft` and phase remains P6 — no other changes needed.
