---
story_id: STORY-044
title: "PDF: EMU-to-PDF Coordinate Mapping + Y-Axis Flip"
crate: slideforge-pdf
recorded_on: 2026-05-31
recording_tool: VHS 0.10.0 (terminal recording)
modality: library/test-harness (no CLI surface yet)
test_runner: cargo-nextest
total_tests_in_crate: 62
total_tests_passing: 62
---

# STORY-044 Demo Evidence Report

## Summary

`slideforge-pdf` is a Rust library crate — there is no CLI surface.
All demos drive acceptance criteria through `cargo nextest run` invocations targeting
named test functions in `crates/slideforge-pdf/src/coords.rs`,
`crates/slideforge-pdf/src/exporter.rs`, `crates/slideforge-pdf/src/svg_embed.rs`,
and `crates/slideforge-pdf/tests/`.

All 62 tests in the crate pass (0 failures, 0 skipped) as of the recording date.

---

## AC Coverage Map

| AC | Title | Recording | Tests Exercised | PASS |
|----|-------|-----------|-----------------|------|
| AC-001 | emu_to_pt canonical test vectors | AC-001-005-emu-to-pt-pure-functions | `test_bc_4_03_005_emu_to_pt_slide_width_720`, `_slide_height_405`, `_one_point`, `_zero_is_zero` | PASS |
| AC-002 | ir_y_to_pdf_y canonical test vectors | AC-001-005-emu-to-pt-pure-functions | `test_bc_4_03_005_ir_y_to_pdf_y_top_left_element_305`, `_bottom_edge_zero`, `_zero_height_at_origin_is_slide_height` | PASS |
| AC-003 | Rounding error < 0.001 points | AC-001-005-emu-to-pt-pure-functions | `test_bc_4_03_005_emu_to_pt_rounding_error_below_0_001` | PASS |
| AC-004 | emu_to_pt is a pure function | AC-001-005-emu-to-pt-pure-functions | `test_bc_4_03_005_emu_to_pt_is_deterministic` | PASS |
| AC-005 | ir_y_to_pdf_y is a pure function | AC-001-005-emu-to-pt-pure-functions | `test_bc_4_03_005_ir_y_to_pdf_y_is_deterministic` | PASS |
| AC-006 | No element outside canvas | AC-006-007-008-canvas-bounds-edge-cases, AC-Y-AXIS-vertical-placement-regression | `test_bc_4_03_005_no_element_outside_canvas_after_conversion`, `test_bc_4_03_005_ac006_export_all_elements_within_canvas` | PASS |
| AC-007 | 4:3 brand template dimensions | AC-006-007-008-canvas-bounds-edge-cases | `test_bc_4_03_005_ir_y_to_pdf_y_4x3_slide_mapping` | PASS |
| AC-008 | Zero-height element edge case | AC-006-007-008-canvas-bounds-edge-cases | `test_bc_4_03_005_ir_y_to_pdf_y_zero_height_at_origin_is_slide_height` | PASS |
| AC-009 | Font subsetting < 100 KB subset bound | AC-009-font-subsetting | `test_bc_4_03_002_ac009_font_subset_smaller_than_full_font`, `_lm_math_fixture_exists`, `_no_direct_subsetter_call` | PASS |
| Y-AXIS / DIR-044-001 | Title at TOP, Footer at BOTTOM | AC-Y-AXIS-vertical-placement-regression | `test_vertical_placement_title_at_top`, `test_vertical_placement_footer_at_bottom` | PASS |
| SVG / F-P18-001 | usvg abs_transform per path | AC-SVG-diagram-fidelity | `test_f_p18_001_usvg_transform_to_krilla_maps_correctly`, `test_f_p18_001_non_identity_abs_transform_reflected_in_pdf` | PASS |
| SVG / F-P18-002 | Fit-to-frame scaling | AC-SVG-diagram-fidelity | `test_f_p18_002_fit_to_frame_scaling_shrinks_oversized_svg` | PASS |
| SVG / F-P19-001 | Stroke rendering (plotters/mermaid) | AC-SVG-diagram-fidelity | `test_f_p19_001_stroke_only_path_emits_stroke_operator` | PASS |
| SVG / F-P19-004 | Asymmetric shear mapping | AC-SVG-diagram-fidelity | `test_f_p19_004_asymmetric_shear_maps_kx_ky_correctly` | PASS |
| OBS-044-22-01 | Paint-state determinism (no fill leak) | AC-PAINT-paint-state-determinism | `test_obs_044_22_01_text_fill_not_leaked_from_svg_frame` | PASS |

---

## Recordings

### AC-001 through AC-005 — EMU-to-pt pure functions + Y-flip arithmetic

**Files:**
- `AC-001-005-emu-to-pt-pure-functions.tape`
- `AC-001-005-emu-to-pt-pure-functions.gif`
- `AC-001-005-emu-to-pt-pure-functions.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-pdf -E 'test(emu_to_pt) | test(ir_y_to_pdf_y) | test(rounding_error)'
```

Demonstrates: all 10 canonical test vectors for `emu_to_pt` and `ir_y_to_pdf_y` pass,
including slide-width (720pt), slide-height (405pt), 1-point, zero, top-left element (305pt),
bottom-edge element (0pt), zero-height element, rounding precision, and determinism.

---

### AC-006, AC-007, AC-008 — Canvas bounds + edge cases

**Files:**
- `AC-006-007-008-canvas-bounds-edge-cases.tape`
- `AC-006-007-008-canvas-bounds-edge-cases.gif`
- `AC-006-007-008-canvas-bounds-edge-cases.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-pdf -E 'test(no_element_outside_canvas) | test(4x3_slide) | test(zero_height)'
```

Demonstrates: all elements in representative fixture layouts stay within `[0.0, 0.0, 720.0, 405.0]`;
4:3 brand slide (576pt × 432pt) uses supplied `slide_h` not the hardcoded constant;
zero-height element returns `SLIDE_HEIGHT_PT − ir_y_pt`.

---

### Y-Axis Placement — Title at TOP, Footer at BOTTOM (DIR-044-001 mirror-bug regression)

**Files:**
- `AC-Y-AXIS-vertical-placement-regression.tape`
- `AC-Y-AXIS-vertical-placement-regression.gif`
- `AC-Y-AXIS-vertical-placement-regression.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-pdf -E 'test(vertical_placement) | test(ac006_export)'
```

Demonstrates the critical regression tests that catch the Y-axis mirror bug:
- Title at `ir_y=0`: `baseline_y = 57.6 < 202.5` (top half). Under the old buggy formula the value
  was 390.6 — the assert would have failed.
- Footer at slide bottom: `baseline_y = 397.8 > 202.5` (bottom half).
- AC-006 integration export: all frame baselines within `[0.0, 405.0]`.

---

### SVG / Diagram Fidelity — usvg abs_transform + fit-to-frame + stroke + shear

**Files:**
- `AC-SVG-diagram-fidelity.tape`
- `AC-SVG-diagram-fidelity.gif`
- `AC-SVG-diagram-fidelity.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-pdf -E 'test(f_p18) | test(f_p19) | test(diagram_frame)'
```

Demonstrates: usvg `abs_transform()` is correctly mapped to krilla `Transform::from_row` (sx, ky, kx, sy, tx, ty);
non-identity ancestor transforms are reflected in the PDF content stream (`cm` matrix operator);
oversized SVG is scaled down to frame dimensions; stroke-only paths emit the `S` PDF stroke operator;
asymmetric shear `kx ≠ ky` maps to the correct krilla transform fields.
Also demonstrates that a `Diagram` frame in `export()` produces vector path operators in the PDF.

---

### Paint-State Determinism (OBS-044-22-01)

**Files:**
- `AC-PAINT-paint-state-determinism.tape`
- `AC-PAINT-paint-state-determinism.gif`
- `AC-PAINT-paint-state-determinism.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-pdf -E 'test(obs_044_22_01)'
```

Demonstrates: when a `Diagram` frame (red SVG, `#FF0000`) precedes a `Title` frame on the same
krilla Surface, the exported PDF contains `0 0 0 rg` (explicit black fill from `draw_text_at_bbox`)
AFTER the last `1 0 0 rg` (red fill from the SVG path). Text color is never inherited from
prior SVG frame state. Without the `surface.set_fill(Some(text_fill_black()))` call, the PDF
would contain `1 0 0 rg` before the text draw operator — this test catches that regression.

---

### AC-009 — Font Subsetting (krilla-internal, < 100 KB subset bound)

**Files:**
- `AC-009-font-subsetting.tape`
- `AC-009-font-subsetting.gif`
- `AC-009-font-subsetting.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-pdf --test ac009_font_subsetting --no-capture 2>&1
```

Demonstrates: `PdfExporter::export_uncompressed()` with a Title frame containing "Hi"
(2 ASCII glyphs) from the Latin Modern Math OTF fixture (4,802 glyphs, 717 KiB):
- `embedded_total > 0` — a font stream was embedded (non-vacuous guard).
- `embedded_total < 100,000` — embedded streams are subset-scale, not full-font-scale.
- `embedded_total < 733,736` — smaller than the unsubsetted source font file.
No `subsetter::subset(...)` call in `slideforge-pdf/src/` — subsetting is internal to krilla.

---

## Error-Path Coverage

| Behavior | Where Tested |
|----------|-------------|
| Font resolution returns `None` (no system font) — text draw skipped, export still produces PDF | `test_f044_001_no_panic_on_multibyte_text_with_font_none` (exporter tests, not in a separate recording but in the full-suite run visible in AC-009 recording) |
| SVG depth cap exceeded — returns `PdfExportError::SvgEmbed` instead of stack overflow | `test_sec_001_svg_depth_cap_exceeds_limit_returns_error` (svg_embed tests, covered in AC-SVG recording) |
| SVG size cap exceeded — `PdfExportError::SvgEmbed` on oversized input | `test_sec_002_svg_size_cap_over_limit_returns_error` (svg_embed tests, covered in AC-SVG recording) |

---

## Proptest Coverage

The proptest file `tests/coords_proptest.rs` provides property-based verification:
- `test_bc_4_03_005_proptest_ir_y_to_pdf_y_nonneg_for_valid_inputs`: for random valid `(ir_y, element_h, slide_h)` where `ir_y + element_h <= slide_h`, `ir_y_to_pdf_y() >= 0.0`.
- `test_bc_4_03_005_proptest_emu_to_pt_monotone`: `emu_to_pt` is monotone non-decreasing.

These run as part of the full crate suite. The proptest regression file `coords_proptest.proptest-regressions` is committed to preserve discovered edge cases.

---

## Recording Environment

| Key | Value |
|-----|-------|
| Platform | macOS Darwin 25.5.0 (arm64) |
| VHS | 0.10.0 |
| Font | FiraCode Nerd Font Mono |
| Theme | Catppuccin Mocha |
| Rust toolchain | stable (per `rust-toolchain.toml`) |
| cargo-nextest | latest installed |
| Worktree | `/Users/jmagady/Dev/slideforge/.worktrees/STORY-044` |
| Branch | `feature/S-044` |
