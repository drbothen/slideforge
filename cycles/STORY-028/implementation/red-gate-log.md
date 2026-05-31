---
story_id: STORY-028
phase: test-writer
timestamp: 2026-05-28
status: RED_GATE_VERIFIED
---

# Red Gate Log — STORY-028

## Summary

75 failing tests written for BC-3.04.001 (shape: block layout) and BC-3.05.001
(Rich Inline Formatting). All 75 behavioral AC tests fail via `todo!()` panics.
7 structural/type-system tests pass correctly (compile-time contract verification).

## Test Run Results

```
249 tests run: 174 passed, 75 failed, 0 skipped
```

- `174 passed` — all pre-existing tests from STORY-026 and other modules
- `75 failed` — new STORY-028 AC tests (Red Gate confirmed)
- `0 skipped`

## Failing Tests by AC

### BC-3.04.001 — shape: Block Layout

#### AC-001 — EMU conversion from user-declared units (10 tests)

| Test | Drives |
|------|--------|
| `test_bc_3_04_001_ac001_half_inch_to_emu` | `from_inches(500) → Emu(457_200)` |
| `test_bc_3_04_001_ac001_one_inch_to_emu` | `from_inches(1000) → Emu(914_400)` |
| `test_bc_3_04_001_ac001_two_inches_to_emu` | `from_inches(2000) → Emu(1_828_800)` |
| `test_bc_3_04_001_ac001_zero_inches_to_emu` | `from_inches(0) → Emu(0)` |
| `test_bc_3_04_001_ac001_negative_half_inch_to_emu` | `from_inches(-500) → Emu(-457_200)` |
| `test_bc_3_04_001_ac001_one_em_to_emu` | `from_em(1000, DEFAULT_EM_IN_EMU) → Emu(457_200)` |
| `test_bc_3_04_001_ac001_two_em_to_emu` | `from_em(2000, DEFAULT_EM_IN_EMU) → Emu(914_400)` |
| `test_bc_3_04_001_ac001_unit_to_emu_inches` | `unit_to_emu(Inches(500), …) → Emu(457_200)` |
| `test_bc_3_04_001_ac001_unit_to_emu_em` | `unit_to_emu(Em(1000), …) → Emu(457_200)` |
| `test_bc_3_04_001_ac001_full_position_vector_inches` | canonical 4-field vector from story spec |

#### AC-002 — Shape Frame added to LaidOutSlide (4 tests)

| Test | Drives |
|------|--------|
| `test_bc_3_04_001_ac002_layout_shapes_returns_one_frame_per_shape` | 1 shape → 1 frame |
| `test_bc_3_04_001_ac002_layout_shapes_preserves_source_order` | 2 shapes → 2 frames in order |
| `test_bc_3_04_001_ac002_layout_shapes_empty_slice_produces_empty_output` | 0 shapes → 0 frames |
| `test_bc_3_04_001_ac002_frame_content_is_shape_variant` | FrameContent::Shape variant |

#### AC-003 — Off-canvas warning (7 tests)

| Test | Drives |
|------|--------|
| `test_bc_3_04_001_ac003_negative_x_is_off_canvas` | `x < 0` detection |
| `test_bc_3_04_001_ac003_negative_y_is_off_canvas` | `y < 0` detection |
| `test_bc_3_04_001_ac003_exceeds_right_edge_is_off_canvas` | `x + w > page_w` |
| `test_bc_3_04_001_ac003_exceeds_bottom_edge_is_off_canvas` | `y + h > page_h` |
| `test_bc_3_04_001_ac003_valid_position_not_off_canvas` | on-canvas → no warning |
| `test_bc_3_04_001_ac003_shape_fills_entire_page_not_off_canvas` | full-page shape |
| `test_bc_3_04_001_ac003_layout_shapes_emits_offcanvas_warning_and_produces_frame` | warning + frame |
| `test_bc_3_04_001_ac003_layout_shapes_valid_shape_no_warnings` | happy-path |

#### AC-004 — Decorative shape (3 tests)

| Test | Drives |
|------|--------|
| `test_bc_3_04_001_ac004_decorative_true_produces_alt_decorative` | `AltText::Decorative` |
| `test_bc_3_04_001_ac004_explicit_alt_produces_alt_provided` | `AltText::Provided` |
| `test_bc_3_04_001_ac004_layout_shapes_decorative_shape_frame` | full path |

#### EC-001 — MissingAlt error (3 tests)

| Test | Drives |
|------|--------|
| `test_bc_3_04_001_ec001_missing_alt_returns_error` | `build_shape_frame` direct |
| `test_bc_3_04_001_ec001_layout_shapes_missing_alt_returns_error` | `layout_shapes` path |
| `test_bc_3_04_001_ec001_second_shape_missing_alt_returns_error` | error on 2nd shape |

#### ShapeType parsing (7 tests)

- `parse_shape_type("rect")` → `Rect`
- `parse_shape_type("ellipse")` → `Ellipse`
- `parse_shape_type("arrow")` → `Arrow`
- `parse_shape_type("line")` → `Line`
- `parse_shape_type("star")` → `Star`
- `parse_shape_type("frobnicator")` → `Custom`
- `parse_shape_type("")` → `Custom`

#### FillSpec helpers (5 tests)

- `parse_hex_color("#003766")` → `Some(Rgb { r: 0, g: 55, b: 102 })`
- `parse_hex_color("#FF6F00")` → `Some(Rgb { r: 255, g: 111, b: 0 })`
- `parse_hex_color("#000000")` → black
- `parse_hex_color("#FFFFFF")` → white
- `parse_hex_color("not-a-color")` → None
- `parse_hex_color("#GGGGGG")` → None
- `parse_hex_color("#12345")` → None (too short)
- `parse_hex_color("003766")` → None (missing #)
- `build_fill_spec(Some("#003766"))` → `SolidColor`
- `build_fill_spec(None)` → `FillSpec::None`
- `build_fill_spec(Some("none"))` → `FillSpec::None`

### BC-3.05.001 — Rich Inline Formatting

#### AC-005 — All 12 InlineNode variants (12 tests)

One test per variant (Plain, Bold, Italic, Code, Link, Math, Footnote, Xref,
Superscript, Subscript, Strikethrough, Highlight) verifying no warnings produced
for non-xref nodes. Plus empty-slice, nodes-unchanged, and full-12-variants tests.

#### AC-006 — Nested inline formatting preserved (3 tests)

- `Bold(Italic(Plain))` nesting preserved + no mutation
- `Highlight(Superscript(Plain))` nesting preserved
- `check_inline_node` recurses into Bold children (nested Xref found)

#### AC-007 — Xref target validation (8 tests)

- Known xref → no warning
- Unknown xref → `LayoutWarning::XrefTargetNotFound`
- Multiple unknown xrefs → multiple warnings (DI-018 accumulate all)
- Mixed known+unknown → one warning
- `check_inline_node` known xref → no warning
- `check_inline_node` unknown xref → warning

#### collect_slide_titles (3 tests)

- Multiple titles → HashSet with all titles
- Empty deck → empty set
- Single slide → set of size 1

#### run_inline_validation integration (5 tests)

- No TextRun frames → no warnings
- TextRun with no xref → no warnings
- Unknown xref in TextRun → warning
- Known xref → no warning
- Accumulates across multiple slides

## Passing Tests (Type-System / Green-By-Design)

| Test | Reason passes |
|------|--------------|
| `test_bc_3_05_001_ac005_helper_covers_all_12_variants` | Count assertion on test helper — no impl code |
| `test_bc_3_05_001_ac005_all_kind_names_present` | Exercises `InlineNode::kind_name()` — already implemented |
| `test_layout_warning_offcanvas_constructable` | Struct construction + Hash — types already defined |
| `test_layout_warning_xref_not_found_constructable` | Same |
| `test_shape_frame_implements_hash_eq_clone` | Type-level contract — types defined in stub |
| `test_shape_layout_output_implements_hash_eq_clone` | Type-level contract |
| `test_shape_unit_variants_constructable` | Pattern match on enum variants |

These 7 tests verify structural contracts that are ALREADY satisfied by the stub types.
None are vacuously true — they make real assertions against real type implementations.

## Red Gate Decision

**ALL 75 behavioral AC tests FAIL.** Red Gate verified.
Implementer may proceed to green-phase each test, one at a time.
