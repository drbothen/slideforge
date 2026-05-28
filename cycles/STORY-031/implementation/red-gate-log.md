---
story: STORY-031
title: "Chart Renderer: bar/line/pie/scatter/area/histogram/stacked-bar"
phase: red-gate
date: 2026-05-27
agent: test-writer
---

# Red Gate Log — STORY-031

## Summary

64 tests total: 15 PASS, 49 FAIL.

Red Gate: VERIFIED. All behavioral tests fail before any implementation exists.

## Test Run

```
cargo test -p slideforge-charts --no-fail-fast
test result: FAILED. 15 passed; 49 failed; 0 ignored; 0 measured
```

## Passing Tests (15) — Legitimately Pass Without Implementation

These tests verify type definitions and compile-time guarantees that are
intentionally implemented as part of the data model (not the rendering behavior):

| Test | Why It Passes |
|------|---------------|
| `types::test_bc_1_11_001_chart_type_from_keyword_all_supported` | Tests `ChartType::from_keyword` — type model, not rendering |
| `types::test_bc_1_11_001_chart_type_from_keyword_unknown_returns_none` | Same |
| `types::test_bc_1_11_001_chart_type_as_keyword_roundtrip` | Same |
| `types::test_bc_1_11_001_chart_svg_newtype_into_string` | Tests `ChartSvg` newtype accessor |
| `types::test_bc_1_11_001_chart_svg_as_str` | Same |
| `types::test_bc_1_11_001_chart_error_unsupported_type_message` | Tests `ChartError` display string |
| `types::test_bc_1_11_001_chart_error_missing_data_field_message` | Same |
| `types::test_bc_1_11_001_chart_error_render_error_message` | Same |
| `types::test_bc_1_11_001_fallback_palette_has_seven_colors` | Tests `FALLBACK_PALETTE.len()` constant |
| `types::test_bc_1_11_001_fallback_palette_all_hex_format` | Tests palette format — static data |
| `types::test_bc_1_11_001_default_dimensions` | Tests `DEFAULT_WIDTH/HEIGHT` constants |
| `types::test_bc_1_11_001_data_point_fields` | Tests struct field accessibility |
| `types::test_bc_1_11_001_data_series_fields` | Same |
| `tests::test_bc_1_11_001_ac001_chart_renderer_impl_id` | Tests `renderer.id()` literal constant |
| `tests::test_bc_1_11_001_ac001_chart_renderer_impl_is_send_sync` | Compile-time trait bound |

**Verdict:** These 15 tests are correctly passing. They test data model correctness
(type definitions, error message format, constants, trait bounds) which is a valid
part of the stub — NOT behavioral rendering. No vacuously true tests exist.

## Failing Tests (49) — All Fail With `todo!()` Panics

All 49 behavioral tests panic at `todo!()` in the appropriate stub function:

### Per-chart-type behavioral tests (7 types × 4 behaviors = 28 tests)
- `test_bc_1_11_001_{bar,line,pie,scatter,area,histogram,stacked_bar}_produces_nonempty_svg`
- `test_bc_1_11_001_{bar,line,pie,scatter,area,histogram,stacked_bar}_has_viewbox`
- `test_bc_1_11_001_{bar,line,pie,scatter,area,histogram,stacked_bar}_has_aria_label`
- `test_bc_1_11_001_{bar,line,pie,scatter,area,histogram,stacked_bar}_no_script`

### Accessibility tests (5 tests)
- `accessibility::test_bc_1_11_001_accessibility_injects_aria_label`
- `accessibility::test_bc_1_11_001_accessibility_injects_role_img`
- `accessibility::test_bc_1_11_001_accessibility_injects_title_element`
- `accessibility::test_bc_1_11_001_accessibility_alt_text_escaping`
- `accessibility::test_bc_1_11_001_accessibility_no_svg_tag_returns_error`

### Safety tests (4 tests)
- `safety::test_bc_1_11_001_safety_clean_svg_passes`
- `safety::test_bc_1_11_001_safety_rejects_script_element`
- `safety::test_bc_1_11_001_safety_rejects_foreign_object`
- `safety::test_bc_1_11_001_safety_rejects_script_case_insensitive`

### Additional behavioral tests (7 tests)
- `test_bc_1_11_001_brand_colors_applied`
- `test_bc_1_11_001_fallback_palette_when_brand_empty`
- `test_bc_1_11_001_unknown_type_error`
- `test_bc_1_11_001_pie_single_slice`
- `test_bc_1_11_001_brand_font_in_labels`

### Snapshot tests (7 tests)
- `test_bc_1_11_001_snapshot_{bar,line,pie,scatter,area,histogram,stacked_bar}`

## BC Coverage

| BC Clause | Tests |
|-----------|-------|
| BC-1.11.001 precondition 1 (supported type) | `test_bc_1_11_001_unknown_type_error` |
| BC-1.11.001 postcondition 1 (valid SVG) | `*_produces_nonempty_svg` (7 tests) |
| BC-1.11.001 postcondition 2 (brand colors) | `test_bc_1_11_001_brand_colors_applied`, `test_bc_1_11_001_fallback_palette_when_brand_empty` |
| BC-1.11.001 postcondition 3 (viewBox + dimensions) | `*_has_viewbox` (7 tests) |
| BC-1.11.001 postcondition 4 (aria-label + title) | `*_has_aria_label` (7 tests) + accessibility module (5 tests) |
| BC-1.11.001 postcondition 5 (no script/foreignObject) | `*_no_script` (7 tests) + safety module (4 tests) |
| BC-1.11.001 invariant 3 (brand font) | `test_bc_1_11_001_brand_font_in_labels` |
| BC-1.11.001 EC-002 (single-slice pie) | `test_bc_1_11_001_pie_single_slice` |
| Snapshot regression | 7 insta snapshot tests |

## Handoff Instruction for Implementer

All 49 behavioral tests fail with `not yet implemented` panics from `todo!()` macros.
Implement each stub function to make each test pass, one at a time.

**Recommended implementation order:**
1. `safety::assert_no_forbidden_elements` (simplest — string scan)
2. `accessibility::inject_aria_attributes` (quick-xml string manipulation)
3. `bar::render_bar` (reference implementation for other types)
4. `line::render_line`, `area::render_area` (similar to bar)
5. `scatter::render_scatter` (similar pattern)
6. `histogram::render_histogram` (frequency bins)
7. `pie::render_pie` (polar math)
8. `stacked_bar::render_stacked_bar` (accumulation logic)
9. `lib::extract_accent_colors` (color extraction from Brand)
10. `lib::build_internal_spec` + `lib::dispatch_and_process` + `ChartRendererImpl::render`
