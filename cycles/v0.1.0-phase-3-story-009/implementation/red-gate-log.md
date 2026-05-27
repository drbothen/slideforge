# Red Gate Log — STORY-009

**Date:** 2026-05-26
**Branch:** feature/S-1.09
**Agent:** test-writer
**Status:** RED GATE VERIFIED

## Summary

33 failing tests (STORY-009 behavioral requirements). 0 pre-existing test regressions.
195 pre-existing tests continue to pass.

## Command

```
cargo test -p slideforge-syntax
```

## Result

```
test result: FAILED. 195 passed; 33 failed; 0 ignored; 0 measured; 0 filtered out
```

## Failing Tests (all STORY-009, all expected to fail)

### keywords::tests — classify_keyword / is_slide_type_keyword / is_directive_keyword / is_structural_keyword stubs

| Test | AC | Fails because |
|------|----|---------------|
| `test_bc_1_09_006_classify_keyword_at_fn_is_reserved` | AC-012 | `classify_keyword` is `todo!()` |
| `test_bc_1_09_006_classify_keyword_at_mixin_is_reserved` | AC-012 | `classify_keyword` is `todo!()` |
| `test_bc_1_09_006_is_directive_keyword_fn_mixin_recognized` | AC-012 | `is_directive_keyword` is `todo!()` |
| `test_bc_1_09_006_is_structural_keyword_vars_set_recognized` | AC-006 | `is_structural_keyword` is `todo!()` |
| `test_bc_1_09_008_classify_keyword_chart_data_is_safe` | AC-014 | `classify_keyword` is `todo!()` |
| `test_bc_1_09_008_classify_keyword_chart_is_slide_type` | AC-013 | `classify_keyword` is `todo!()` |
| `test_bc_1_09_008_is_slide_type_keyword_all_31_types` | AC-013 | `is_slide_type_keyword` is `todo!()` |
| `test_bc_1_09_008_is_slide_type_keyword_suffix_not_collision` | AC-014 | `is_slide_type_keyword` is `todo!()` |
| `test_bc_1_09_008_safe_identifier_not_classified` | AC-014 | `classify_keyword` is `todo!()` |
| `test_bc_1_09_009_classify_keyword_raw_maps_to_e_par_009` | AC-010 | `classify_keyword` is `todo!()` |

### parser::shape::tests — shape_block() combinator stub

| Test | AC | Fails because |
|------|----|---------------|
| `test_bc_1_09_009_shape_block_all_five_fields_parsed` | AC-009 | `shape_block()` is `todo!()` |
| `test_bc_1_09_009_shape_block_partial_fields_parsed` | AC-009 | `shape_block()` is `todo!()` |

### parser::tests — integration tests requiring full implementation

| Test | AC | Fails because |
|------|----|---------------|
| `test_bc_1_09_006_empty_inline_math_is_error_or_empty` | AC-006 / EC-001 | Math mode parser not implemented |
| `test_bc_1_09_006_nested_math_delimiters_are_errors` | AC-006 / EC-002 | Math mode parser not implemented |
| `test_bc_1_09_006_math_inline_dollar_single_produces_math_inline_chunk` | AC-006 | Math mode parser not implemented |
| `test_bc_1_09_006_at_fn_directive_rejected_with_e_par_006` | AC-012 | Reserved keyword check not wired |
| `test_bc_1_09_006_at_mixin_directive_rejected_with_e_par_006` | AC-012 / EC-006 | Reserved keyword check not wired |
| `test_bc_1_09_006_math_inline_chunk_snapshot` | AC-006 | No snapshot file (expected before impl) |
| `test_bc_1_09_007_math_display_dollar_double_produces_math_display_chunk` | AC-007 | Math mode parser not implemented |
| `test_bc_1_09_007_math_display_chunk_snapshot` | AC-007 | No snapshot file (expected before impl) |
| `test_bc_1_09_008_at_brace_in_math_produces_math_interp_chunk` | AC-008 | Math mode parser not implemented |
| `test_bc_1_09_008_double_brace_in_math_is_literal_not_interp` | AC-008 | Math mode parser not implemented |
| `test_bc_1_09_008_math_interp_chunk_snapshot` | AC-008 | No snapshot file (expected before impl) |
| `test_bc_1_09_008_raw_in_vars_is_e_par_008_not_e_par_009` | AC-011 | Vars collision check not wired |
| `test_bc_1_09_008_vars_chart_collision_emits_e_par_008` | AC-013 | Vars collision check not wired |
| `test_bc_1_09_008_vars_title_collision_emits_e_par_008` | AC-013 / EC-007 | Vars collision check not wired |
| `test_bc_1_09_009_bare_raw_field_rejected_with_e_par_009` | AC-010 | Raw keyword check not wired |
| `test_bc_1_09_009_raw_pptx_rejected_with_e_par_009` | AC-010 | Raw keyword check not wired |
| `test_bc_1_09_010_missing_version_emits_e_par_010` | AC-003 | Version gate not implemented |
| `test_bc_1_09_010_version_0_rejected` | AC-002 / EC-004 | Version gate not implemented |
| `test_bc_1_09_010_version_2_rejected_with_e_par_010` | AC-002 | Version gate not implemented |
| `test_bc_1_09_010_version_non_numeric_rejected` | AC-002 / EC-005 | Version gate not implemented |
| `test_bc_1_09_010_version_whitespace_only_is_error` | AC-002 / EC-003 | Version gate not implemented |

## Passing Tests (expected to pass — type-level correctness already delivered)

These tests pass because they verify the AST types and error constructors,
which are implemented as part of the stubs:

- `ast::tests::test_bc_1_09_009_*` — ShapeNode struct derives and construction
- `error::tests::test_bc_1_09_*` — new error variant constructors and diagnostic codes
- `parser::shape::tests::test_bc_1_09_009_empty_shape_node_constructible` — helper
- `parser::shape::tests::test_bc_1_09_009_field_value_shape_variant_constructible` — AST type
- `parser::shape::tests::test_bc_1_09_009_shape_node_derives_hash_eq_clone_debug` — derives
- `parser::tests::test_bc_1_09_001_version_1_accepted_stored_in_deck_node` — existing behavior
- `parser::tests::test_bc_1_09_001_version_1_dot_2_accepted` — vacuously true until gate
- `parser::tests::test_bc_1_09_008_vars_chart_data_suffix_is_not_a_collision` — negative (no false positive)
- `parser::tests::test_bc_1_09_010_version_2_is_always_fatal` — constructor test
- `template::tests::test_bc_1_09_006_*` — new TemplateChunk variant derives

## AC Coverage Map

| AC | Tests | Status |
|----|-------|--------|
| AC-001 (version "1" stored) | `test_bc_1_09_001_version_1_accepted_stored_in_deck_node` | Passes (existing impl) |
| AC-002 (version "2" fatal) | `test_bc_1_09_010_version_2_rejected_with_e_par_010` | FAILS (need gate) |
| AC-003 (missing version) | `test_bc_1_09_010_missing_version_emits_e_par_010` | FAILS (need gate) |
| AC-004 (version "1.2" accepted) | `test_bc_1_09_001_version_1_dot_2_accepted` | Passes (existing behavior) |
| AC-005 (version 2 always fatal) | `test_bc_1_09_010_version_2_is_always_fatal` | Passes (constructor) |
| AC-006 (math inline $...$) | 4 tests | FAILS (need math parser) |
| AC-007 (math display $$...$$) | 2 tests | FAILS (need math parser) |
| AC-008 (@{var} in math) | 2 tests | FAILS (need math parser) |
| AC-009 (shape: block) | 4 tests (2 parser, 2 type-level) | 2 FAIL (need parser), 2 pass (type-level) |
| AC-010 (raw rejected) | 2 tests | FAILS (need keyword check) |
| AC-011 (raw in vars → E-PAR-008) | 1 test | FAILS (need collision check) |
| AC-012 (@fn → E-PAR-006) | 3 tests | FAILS (need keyword check) |
| AC-013 (chart in vars) | 3 tests | FAILS (need collision check) |
| AC-014 (chart_data safe) | 2 tests | Passes (negative) |
| AC-015 (derives, no unsafe) | Passes via type-level tests + build |
| AC-016 (rustdoc) | Passes via `cargo doc -D warnings` |

## Instruction for Implementer

Make each test pass, one at a time, with minimum code. Recommended order:

1. **keywords.rs** — implement `classify_keyword`, `is_slide_type_keyword`,
   `is_directive_keyword`, `is_structural_keyword` using the static tables
2. **Version gate** — in `parser/deck.rs`, add a `validate()` call on
   `version_decl_parser` that extracts the major version and emits
   `SyntaxError::VersionError` for versions != "1" and for missing version
3. **vars name collision** — in `parser/deck.rs`, add a `validate()` call in
   `vars_block_parser` that checks `is_slide_type_keyword` on each entry name
4. **Reserved keyword check** — in `parser/deck.rs`, add a check for `@fn`,
   `@mixin`, `@while` etc. at the deck item level
5. **Raw keyword check** — in slide field parsing, reject `raw` as a field name
6. **Math mode parser** — in `parser/template.rs`, extend `template_value` to
   scan for `$...$` and `$$...$$` and emit `MathInline`/`MathDisplay` chunks;
   extend `@{var}` handling for `MathInterp`
7. **shape_block parser** — in `parser/shape.rs`, implement `shape_block()`
   and wire it into `parser/slide.rs`
