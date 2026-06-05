# STORY-001: IR Core Types — Demo Evidence

## Date: 2026-05-25

## AC Coverage

| AC | Evidence | Status |
|----|----------|--------|
| AC-001 through AC-016 | Test suite output below (137 unit tests + 17 doc-tests + 3 compile_fail doc-tests) | PASS |

---

## Test Suite Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running unittests src/lib.rs (target/debug/deps/slideforge_types-a7a5596da0b48322)

running 137 tests
test block::tests::test_bc_1_01_005_content_block_exactly_8_variants ... ok
test block::tests::test_bc_1_01_005_content_block_debug ... ok
test brand::tests::test_bc_1_01_brand_debug ... ok
test block::tests::test_bc_1_01_005_content_block_kind_names ... ok
test block::tests::test_bc_1_01_005_content_block_clone ... ok
test deck::tests::test_bc_1_01_001_deck_debug ... ok
test brand::tests::test_bc_1_01_brand_hash ... ok
test brand::tests::test_bc_1_01_brand_fields ... ok
test block::tests::test_bc_1_01_005_block_wrapper_fields ... ok
test block::tests::test_bc_1_01_005_bullet_item_nested ... ok
test brand::tests::test_bc_1_01_brand_clone ... ok
test brand::tests::test_bc_1_01_layout_definition_fields ... ok
test deck::tests::test_bc_1_01_001_deck_debug ... ok
test deck::tests::test_bc_1_01_001_deck_clone ... ok
test deck::tests::test_bc_1_01_001_deck_eq ... ok
test block::tests::test_bc_1_01_005_text_block_fields ... ok
test block::tests::test_bc_1_01_005_content_block_hash ... ok
test deck::tests::test_bc_1_01_001_deck_fields_present ... ok
test deck::tests::test_bc_1_01_001_deck_hash ... ok
test deck::tests::test_bc_1_01_001_deck_registers_field ... ok
test deck::tests::test_bc_1_01_001_deck_vars_field ... ok
test deck::tests::test_bc_1_01_001_deck_vars_is_ordered_map ... ok
test deck::tests::test_bc_1_01_001_laid_out_deck_fields ... ok
test deck::tests::test_bc_1_01_001_laid_out_element_fields ... ok
test deck::tests::test_bc_1_01_001_semantic_role_custom ... ok
test deck::tests::test_bc_1_01_010_deck_metadata_clone ... ok
test deck::tests::test_bc_1_01_010_deck_metadata_debug ... ok
test deck::tests::test_bc_1_01_010_deck_metadata_fields ... ok
test deck::tests::test_bc_1_01_010_deck_metadata_hash ... ok
test deck::tests::test_bc_1_01_010_deck_metadata_none_title_lang ... ok
test deck::tests::test_bc_1_01_010_deck_metadata_with_author ... ok
test emu::tests::test_bc_1_01_001_emu_add ... ok
test emu::tests::test_bc_1_01_001_emu_canvas_width_constant ... ok
test emu::tests::test_bc_1_01_001_emu_default_is_zero ... ok
test emu::tests::test_bc_1_01_001_emu_from_inches_half_inch ... ok
test emu::tests::test_bc_1_01_001_emu_from_inches_one_inch ... ok
test emu::tests::test_bc_1_01_001_emu_from_points_one_point ... ok
test emu::tests::test_bc_1_01_001_emu_from_points_twelve_points ... ok
test emu::tests::test_bc_1_01_001_emu_implements_copy ... ok
test emu::tests::test_bc_1_01_001_emu_implements_hash ... ok
test emu::tests::test_bc_1_01_001_emu_mul_scalar ... ok
test emu::tests::test_bc_1_01_001_emu_mul_scalar_commutative ... ok
test emu::tests::test_bc_1_01_001_emu_ordering ... ok
test emu::tests::test_bc_1_01_001_emu_per_inch_constant ... ok
test emu::tests::test_bc_1_01_001_emu_per_point_constant ... ok
test emu::tests::test_bc_1_01_001_emu_roundtrip_inches ... ok
test emu::tests::test_bc_1_01_001_emu_slide_width_constant ... ok
test emu::tests::test_bc_1_01_001_emu_sub ... ok
test error::tests::test_bc_1_01_type_error_clone ... ok
test error::tests::test_bc_1_01_type_error_debug ... ok
test error::tests::test_bc_1_01_type_error_index_out_of_bounds ... ok
test error::tests::test_bc_1_01_type_error_key_not_found ... ok
test error::tests::test_bc_1_01_type_error_undefined_variable ... ok
test error::tests::test_bc_1_01_type_error_wrong_type ... ok
test inline::tests::test_bc_1_01_008_inline_node_clone ... ok
test inline::tests::test_bc_1_01_008_inline_node_debug ... ok
test inline::tests::test_bc_1_01_008_inline_node_eq ... ok
test inline::tests::test_bc_1_01_008_inline_node_exactly_11_variants ... ok
test inline::tests::test_bc_1_01_008_inline_node_hash ... ok
test inline::tests::test_bc_1_01_008_inline_node_kind_names ... ok
test inline::tests::test_bc_1_01_008_inline_node_ne ... ok
test inline::tests::test_bc_1_01_008_inline_node_nested ... ok
test math::tests::test_bc_1_01_006_math_node_clone ... ok
test math::tests::test_bc_1_01_006_math_node_debug ... ok
test math::tests::test_bc_1_01_006_math_node_display_constructor ... ok
test math::tests::test_bc_1_01_006_math_node_eq ... ok
test math::tests::test_bc_1_01_006_math_node_fields ... ok
test math::tests::test_bc_1_01_006_math_node_hash ... ok
test math::tests::test_bc_1_01_006_math_node_inline_constructor ... ok
test math::tests::test_bc_1_01_006_math_node_ne ... ok
test ordered_map::tests::test_bc_1_01_ordered_map_clone ... ok
test ordered_map::tests::test_bc_1_01_ordered_map_default_is_empty ... ok
test ordered_map::tests::test_bc_1_01_ordered_map_deref_gives_indexmap_api ... ok
test ordered_map::tests::test_bc_1_01_ordered_map_hash_order_matters ... ok
test ordered_map::tests::test_bc_1_01_ordered_map_hash_stable ... ok
test ordered_map::tests::test_bc_1_01_ordered_map_usable_as_hashmap_key ... ok
test ordered_map::tests::test_ordered_map_eq_is_order_sensitive ... ok
test ordered_map::tests::test_ordered_map_eq_same_order_is_equal ... ok
test register::tests::test_bc_1_01_007_register_clone ... ok
test register::tests::test_bc_1_01_007_register_copy ... ok
test register::tests::test_bc_1_01_007_register_debug ... ok
test register::tests::test_bc_1_01_007_register_default ... ok
test register::tests::test_bc_1_01_007_register_display ... ok
test register::tests::test_bc_1_01_007_register_equality ... ok
test register::tests::test_bc_1_01_007_register_hash ... ok
test register::tests::test_bc_1_01_007_register_keywords ... ok
test register::tests::test_bc_1_01_007_register_ordering ... ok
test register::tests::test_bc_1_01_007_register_predicates ... ok
test register::tests::test_bc_1_01_007_register_sorted ... ok
test slide::tests::test_bc_1_01_002_field_value_variants ... ok
test slide::tests::test_bc_1_01_002_slide_clone ... ok
test slide::tests::test_bc_1_01_002_slide_debug ... ok
test slide::tests::test_bc_1_01_002_slide_eq ... ok
test slide::tests::test_bc_1_01_002_slide_fields_present ... ok
test slide::tests::test_bc_1_01_002_slide_hash ... ok
test slide::tests::test_bc_1_01_002_slide_register_is_option ... ok
test slide::tests::test_bc_1_01_002_slide_title_str_absent ... ok
test slide::tests::test_bc_1_01_002_slide_title_str_present ... ok
test slide::tests::test_bc_1_01_002_slide_with_tags ... ok
test slide::tests::test_bc_1_01_002_string_part_expr ... ok
test slide::tests::test_bc_1_01_002_string_part_literal ... ok
test span::tests::test_bc_1_01_009_source_span_clone ... ok
test span::tests::test_bc_1_01_009_source_span_debug ... ok
test span::tests::test_bc_1_01_009_source_span_default ... ok
test span::tests::test_bc_1_01_009_source_span_default_is_unknown ... ok
test span::tests::test_bc_1_01_009_source_span_display_known ... ok
test span::tests::test_bc_1_01_009_source_span_display_unknown ... ok
test span::tests::test_bc_1_01_009_source_span_eq ... ok
test span::tests::test_bc_1_01_009_source_span_hash ... ok
test span::tests::test_bc_1_01_009_source_span_ne ... ok
test span::tests::test_bc_1_01_009_source_span_new ... ok
test specs::tests::test_bc_1_01_specs_all_implement_clone ... ok
test specs::tests::test_bc_1_01_specs_all_implement_hash ... ok
test specs::tests::test_bc_1_01_specs_chart_spec_fields ... ok
test specs::tests::test_bc_1_01_specs_diagram_spec_fields ... ok
test specs::tests::test_bc_1_01_specs_image_spec_fields ... ok
test specs::tests::test_bc_1_01_specs_shape_spec_fields ... ok
test specs::tests::test_bc_1_01_specs_table_spec_fields ... ok
test value::tests::test_bc_1_01_003_nan_hashset_insert ... ok
test value::tests::test_bc_1_01_003_nan_ordered_float_eq ... ok
test value::tests::test_bc_1_01_003_nan_same_hash ... ok
test value::tests::test_bc_1_01_003_value_bool_variant ... ok
test value::tests::test_bc_1_01_003_value_float_variant ... ok
test value::tests::test_bc_1_01_003_value_implements_clone ... ok
test value::tests::test_bc_1_01_003_value_implements_debug ... ok
test value::tests::test_bc_1_01_003_value_implements_eq ... ok
test value::tests::test_bc_1_01_003_value_implements_hash ... ok
test value::tests::test_bc_1_01_003_value_int_variant ... ok
test value::tests::test_bc_1_01_003_value_list_variant ... ok
test value::tests::test_bc_1_01_003_value_map_variant ... ok
test value::tests::test_bc_1_01_003_value_no_coercion_int_vs_bool ... ok
test value::tests::test_bc_1_01_003_value_no_coercion_str_no_vs_bool ... ok
test value::tests::test_bc_1_01_003_value_no_coercion_str_vs_int ... ok
test value::tests::test_bc_1_01_003_value_no_from_bool_impl ... ok
test value::tests::test_bc_1_01_003_value_no_from_i64_impl ... ok
test value::tests::test_bc_1_01_003_value_null_variant ... ok
test value::tests::test_bc_1_01_003_value_str_variant ... ok
test value::tests::test_bc_1_01_003_value_type_names ... ok

test result: ok. 137 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests slideforge_types

running 17 tests
test crates/slideforge-types/src/register.rs - register::Register (line 24) ... ok
test crates/slideforge-types/src/math.rs - math::MathNode::inline (line 38) ... ok
test crates/slideforge-types/src/slide.rs - slide::Slide::title_str (line 87) ... ok
test crates/slideforge-types/src/math.rs - math::MathNode::display (line 53) ... ok
test crates/slideforge-types/src/emu.rs - emu::Emu::to_points (line 116) ... ok
test crates/slideforge-types/src/span.rs - span::SourceSpan::new (line 39) ... ok
test crates/slideforge-types/src/value.rs - value::Value::is_null (line 88) ... ok
test crates/slideforge-types/src/register.rs - register::Register::as_keyword (line 67) ... ok
test crates/slideforge-types/src/inline.rs - inline::InlineNode::kind_name (line 71) ... ok
test crates/slideforge-types/src/span.rs - span::SourceSpan::is_unknown (line 55) ... ok
test crates/slideforge-types/src/value.rs - value::Value::as_str (line 102) ... ok
test crates/slideforge-types/src/emu.rs - emu::Emu::from_inches (line 59) ... ok
test crates/slideforge-types/src/emu.rs - emu::Emu::from_points (line 74) ... ok
test crates/slideforge-types/src/block.rs - block::ContentBlock::kind_name (line 90) ... ok
test crates/slideforge-types/src/emu.rs - emu::Emu::to_inches (line 102) ... ok
test crates/slideforge-types/src/emu.rs - emu::Emu::as_i64 (line 89) ... ok
test crates/slideforge-types/src/value.rs - value::Value::type_name (line 166) ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 3 tests
test crates/slideforge-types/src/value.rs - value::Value (line 27) - compile fail ... ok
test crates/slideforge-types/src/value.rs - value::Value (line 32) - compile fail ... ok
test crates/slideforge-types/src/value.rs - value::Value (line 37) - compile fail ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

all doctests ran in 0.39s; merged doctests compilation took 0.34s
```

**Summary: 157 total tests — 137 unit tests + 17 doc-tests + 3 compile_fail doc-tests. 0 failed.**

---

## Clippy Output

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

**Summary: No warnings, no errors. Exit code 0.**

---

## Doc Build Output

```
 Documenting slideforge-types v0.1.0 (/Users/jmagady/Dev/slideforge/.worktrees/STORY-001/crates/slideforge-types)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.41s
   Generated /Users/jmagady/Dev/slideforge/.worktrees/STORY-001/target/doc/slideforge_types/index.html
```

**Summary: Documentation built with zero warnings (RUSTDOCFLAGS="-D warnings" enforced). Exit code 0.**

---

## Workspace Check Output

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
```

**Summary: Full workspace compiles cleanly with zero errors or warnings. Exit code 0.**
