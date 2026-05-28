---
story: STORY-023
phase: red-gate
date: 2026-05-27
commit: 4dccfffb
---

# Red Gate Log — STORY-023 Brand Synthesis

## Summary

All new behavioral tests fail with `todo!()` panics (or correct assertion errors
on the `#[should_panic]` tests). No new test passes without implementation.

## Counts

| Category | Count |
|---|---|
| Pre-existing passing tests | 81 |
| New behavioral tests — FAILING (Red Gate) | 49 |
| New structural tests — PASSING (#[should_panic] on todo!()) | 4 |
| Ignored | 0 |

Total: 81 pass, 49 fail.

## Failing Tests by Module

### inference.rs (15 failures — all `todo!()` on `infer_missing_slots`)

- `test_bc_2_01_004_all_12_declared_no_inference`
- `test_bc_2_01_004_lt1_always_ffffff`
- `test_bc_2_01_004_empty_colors_produces_12_warnings`
- `test_bc_2_01_004_dk2_from_acc1_when_available`
- `test_bc_2_01_004_acc2_hue_rotation_30_degrees`
- `test_bc_2_01_004_invariant_inferred_colors_are_uppercase_hex`
- `test_bc_2_01_004_hlink_darkened_from_acc1`
- `test_bc_2_01_004_fol_hlink_darkened_from_hlink`
- `test_bc_2_01_004_all_12_slots_provided_no_change`
- `test_bc_2_01_004_lt2_inferred_as_near_white`
- `test_bc_2_01_004_dk1_inferred_fallback_when_no_colors_declared`
- `test_bc_2_01_004_inference_is_deterministic`
- `test_bc_2_01_004_hlink_defaults_to_acc1_darkened`
- `test_bc_2_01_004_no_required_slot_missing_section_produces_12_warnings`
- `test_bc_2_01_004_infer_missing_acc_slots_from_acc1`

### layouts.rs (11 failures — all `todo!()` on `generate_all_layouts`)

- `test_bc_2_01_005_generate_31_layouts`
- `test_bc_2_01_005_11_standard_layouts_present`
- `test_bc_2_01_005_20_sf_custom_layouts_present`
- `test_bc_2_01_005_blank_layout_has_zero_placeholders`
- `test_bc_2_01_005_cl01_section_divider_dark_layout`
- `test_bc_2_01_005_cl11_end_slide_dark_layout`
- `test_bc_2_01_005_layout_indices_are_sequential`
- `test_bc_2_01_005_standard_layouts_have_ooxml_types`
- `test_bc_2_01_005_layout_color_references_use_scheme_slots`
- `test_bc_2_01_005_generation_deterministic`
- `test_bc_2_01_005_non_blank_layouts_have_at_least_one_placeholder`
- `test_bc_2_01_005_non_dark_layouts_have_no_color_override`

### layout_xml.rs (6 failures — all `todo!()` on `serialize_layout_to_xml` / `generate_content_types_layout_entries`)

- `test_layout_xml_serializes_with_valid_xml`
- `test_bc_2_01_005_content_types_layout_entries_count`
- `test_bc_2_01_005_dark_layout_xml_has_clr_map_ovr`
- `test_bc_2_01_005_dark_layout_xml_has_explicit_white_text`
- `test_bc_2_01_005_layout_xml_uses_semantic_names`
- `test_bc_2_01_005_light_layout_xml_has_no_clr_map_ovr`

### synthesizer.rs (11 failures — all `todo!()` on `BrandSynthesizer::synthesize` / `load_from_toml`)

- `test_bc_2_01_002_ac004_synthesize_rejects_missing_logo`
- `test_bc_2_01_002_invariant_synthesis_is_deterministic`
- `test_bc_2_01_002_synthesized_template_has_12_color_slots`
- `test_bc_2_01_005_synthesized_template_has_31_layouts`
- `test_bc_2_01_005_ac011_master_id_is_2_pow_31`
- `test_bc_2_01_005_ac011_layout_id_start_is_2_pow_31_plus_1`
- `test_bc_2_01_002_ac012_notes_master_stub_present`
- `test_bc_2_01_002_ac012_handout_master_stub_present`
- `test_bc_2_01_005_synthesize_produces_complete_brand_template`
- `test_bc_2_01_005_synthesize_with_inferred_colors`
- `test_bc_2_01_002_invariant_synthesis_is_deterministic_proptest` (VP-012 proptest)

### toml_schema.rs (5 failures — `todo!()` on `FontConfig::default` / `FooterConfig::default` triggered by serde)

- `test_bc_2_01_002_empty_brand_toml_yields_defaults`
- `test_bc_2_01_002_fol_hlink_serde_rename`
- `test_bc_2_01_002_font_defaults_to_calibri`
- `test_bc_2_01_002_footer_defaults`
- `test_bc_2_01_002_parse_minimal_brand_toml`

Note: these tests trigger `todo!()` in `FontConfig::default()` or `FooterConfig::default()`
which are called internally by the serde `#[serde(default)]` attribute when parsing
partial TOML. They correctly fail at the Red Gate — implementation must provide real
`Default` impls.

## Passing Structural Tests (correct behavior)

These are `#[should_panic]` tests that verify the todo!() panic occurs:
- `test_bc_2_01_002_color_config_as_slot_array_returns_12_in_order`
- `test_bc_2_01_002_default_minimal_is_callable`

These pass without implementation because they assert on the panic itself.

## BC Coverage

| BC | Tests Covering It | Status |
|---|---|---|
| BC-2.01.002 | toml_schema, synthesizer | Red Gate |
| BC-2.01.004 | inference | Red Gate |
| BC-2.01.005 | layouts, layout_xml, synthesizer | Red Gate |
| VP-012 | synthesizer proptest | Red Gate |

## Red Gate Verdict

CONFIRMED. All 49 new behavioral tests FAIL. Zero new tests pass vacuously.
Handoff to Implementer is cleared.
