---
story: STORY-035
phase: red-gate
date: 2026-05-31
status: VERIFIED
commit: 3392b598
---

# Red Gate Log — STORY-035: Writing Register Routing in Evaluator

## Result: RED GATE VERIFIED

- **Failing tests:** 18 (all in `register_routing::tests`)
- **Passing pre-existing tests:** 193
- **Compile result:** CLEAN (full workspace: `cargo build --workspace` exits 0)
- **Failure reason:** All 18 new tests panic with `not yet implemented` — correct `todo!()` stubs

## Type Reconciliation: Spec vs. Reality

The spec's type sketches diverged from the real codebase in several ways:

| Spec Says | Reality | Disposition |
|-----------|---------|-------------|
| `LaidOutSlide` in `slideforge-types` | `LaidOutSlide` in `slideforge-layout/src/types.rs` | Added `register_content` field to real location |
| `InlineSpan` as content type | `InlineNode` (defined in `slideforge-types/src/inline.rs`) | Used `InlineNode` throughout |
| `EvalSlide` as input to `extract_register_content` | No `EvalSlide` type exists — the pipeline uses `slideforge_types::Slide` | Signature uses `&Slide` |
| `Register` enum needs to be created | `Register` already fully implemented in `slideforge-types/src/register.rs` with `Hash+Eq+Clone+Copy+Debug+Ord+Display+Default` | No change needed to `Register` itself |
| `RegisteredContent` struct needs to be created | Did not exist | Added to `register.rs` |

## Files Created / Modified

### New Files
- `crates/slideforge-eval/src/register_routing.rs` — stub module with 21 tests

### Modified Files
- `crates/slideforge-types/src/register.rs` — added `RegisteredContent` struct with `plain()`, `is_empty()`
- `crates/slideforge-types/src/lib.rs` — re-exported `RegisteredContent`
- `crates/slideforge-layout/src/types.rs` — added `register_content: Vec<RegisteredContent>` to `LaidOutSlide`; re-exported `RegisteredContent`
- `crates/slideforge-layout/src/layout.rs` — seeded `register_content: vec![]` in slide construction
- `crates/slideforge-layout/src/lib.rs` — seeded `register_content: vec![]` in test helper
- `crates/slideforge-layout/src/inline.rs` — seeded `register_content: vec![]` in test helpers
- `crates/slideforge-plugin-api/src/registry.rs` — seeded `register_content: vec![]`
- `crates/slideforge-plugin-api/tests/dog_food_test.rs` — seeded `register_content: vec![]`
- 31 slide type files in `crates/slideforge-plugin-api/src/slide_types/` — seeded `register_content: vec![]`
- `crates/slideforge-eval/src/lib.rs` — added `pub mod register_routing` and `pub use register_routing::extract_register_content`

## Failing Test Names (18)

All panics: `not yet implemented: STORY-035: implement register routing pass...`

```
register_routing::tests::test_bc_1_14_001_notes_extracted_and_tagged
register_routing::tests::test_bc_1_14_001_notes_content_captured
register_routing::tests::test_bc_1_14_001_notes_only_slide_has_one_entry
register_routing::tests::test_bc_1_14_001_visual_only_slide_produces_empty_register_content
register_routing::tests::test_bc_1_14_001_interpolation_resolved_before_tagging
register_routing::tests::test_bc_1_14_001_empty_string_notes_produces_one_entry
register_routing::tests::test_bc_1_14_001_null_register_field_produces_no_entry
register_routing::tests::test_bc_1_14_001_ec001_notes_only_no_visual_content
register_routing::tests::test_bc_1_14_001_ec002_notes_with_resolved_interpolation
register_routing::tests::test_bc_1_14_001_ec005_for_loop_notes_per_iteration
register_routing::tests::test_bc_1_14_002_report_extracted_and_tagged
register_routing::tests::test_bc_1_14_002_report_content_captured_correctly
register_routing::tests::test_bc_1_14_002_ec004_report_and_detail_both_present
register_routing::tests::test_bc_1_14_003_detail_extracted_and_tagged
register_routing::tests::test_bc_1_14_003_detail_content_captured_correctly
register_routing::tests::test_bc_1_14_004_all_three_registers_on_one_slide
register_routing::tests::test_bc_1_14_004_register_content_ordered_notes_report_detail
register_routing::tests::test_bc_1_14_004_three_register_snapshot
```

## Passing Tests in register_routing::tests (3, type invariants only)

```
register_routing::tests::test_registered_content_implements_hash_eq_clone_debug
register_routing::tests::test_registered_content_plain_is_not_empty
register_routing::tests::test_registered_content_empty_content_is_empty
```

These 3 pass because they exercise `RegisteredContent` type methods directly
without calling the `extract_register_content` stub.

## BC Coverage

| BC Clause | Test(s) |
|-----------|---------|
| BC-1.14.001 invariant 1: notes extracted at eval stage | `test_bc_1_14_001_notes_extracted_and_tagged` |
| BC-1.14.001 postcondition 1: notes content captured | `test_bc_1_14_001_notes_content_captured` |
| BC-1.14.001 postcondition 1: interpolation resolved | `test_bc_1_14_001_interpolation_resolved_before_tagging` |
| BC-1.14.001 EC-001: notes-only slide | `test_bc_1_14_001_ec001_notes_only_no_visual_content` |
| BC-1.14.001 EC-002: notes with interpolation | `test_bc_1_14_001_ec002_notes_with_resolved_interpolation` |
| BC-1.14.002 invariant 1: report extracted at eval stage | `test_bc_1_14_002_report_extracted_and_tagged` |
| BC-1.14.002 postcondition 3: report excluded from PPTX slide | `test_bc_1_14_002_report_content_captured_correctly` |
| BC-1.14.002 EC-004: detail+report same slide | `test_bc_1_14_002_ec004_report_and_detail_both_present` |
| BC-1.14.003 invariant 1: detail extracted at eval stage | `test_bc_1_14_003_detail_extracted_and_tagged` |
| BC-1.14.003 postcondition 3: detail excluded from PPTX | `test_bc_1_14_003_detail_content_captured_correctly` |
| BC-1.14.004 invariant 3: LaidOutSlide carries register-tagged content | `test_bc_1_14_004_all_three_registers_on_one_slide` |
| BC-1.14.004 invariant 3 (snapshot): regression baseline | `test_bc_1_14_004_three_register_snapshot` |
| AC-001: notes fields extracted + tagged | `test_bc_1_14_001_notes_extracted_and_tagged` |
| AC-002: notes excluded from visual frames | `test_bc_1_14_001_visual_only_slide_produces_empty_register_content` |
| AC-003: report extracted + tagged | `test_bc_1_14_002_report_extracted_and_tagged` |
| AC-004: report excluded from visual frames | `test_bc_1_14_002_report_content_captured_correctly` |
| AC-005: detail extracted + tagged | `test_bc_1_14_003_detail_extracted_and_tagged` |
| AC-006: detail excluded from visual frames | `test_bc_1_14_003_detail_content_captured_correctly` |
| AC-007: interpolation evaluated before tagging | `test_bc_1_14_001_interpolation_resolved_before_tagging` |
| AC-008: all three registers on same slide | `test_bc_1_14_004_all_three_registers_on_one_slide` |
| EC-001: notes-only slide | `test_bc_1_14_001_ec001_notes_only_no_visual_content` |
| EC-002: notes with interpolation (resolved) | `test_bc_1_14_001_ec002_notes_with_resolved_interpolation` |
| EC-004: detail+report same slide | `test_bc_1_14_002_ec004_report_and_detail_both_present` |
| EC-005: @for loop notes per-iteration | `test_bc_1_14_001_ec005_for_loop_notes_per_iteration` |

## Notes for Implementer

1. `extract_register_content(_slide: &Slide) -> Vec<RegisteredContent>` — the
   function must inspect `slide.fields` for keys `"notes"`, `"report"`, `"detail"`.
   Each present key must produce one `RegisteredContent` entry.

2. Field values arrive as `FieldValue::Literal(Value::Str(s))` after evaluation.
   Plain text → `vec![InlineNode::Plain(s)]`. The `FieldValue::Inlines` variant
   carries pre-parsed inline nodes for rich formatting.

3. `Value::Null` must NOT produce an entry (test: `test_bc_1_14_001_null_register_field_produces_no_entry`).
   Empty strings MUST produce an entry (test: `test_bc_1_14_001_empty_string_notes_produces_one_entry`).

4. Output ordering must be Notes < Report < Detail regardless of field insertion
   order (test: `test_bc_1_14_004_register_content_ordered_notes_report_detail`).

5. The `field_value_to_inlines` private helper is stubbed in `register_routing.rs`
   with `todo!()` — implement it first, then wire it into `extract_register_content`.

6. The `#[cfg(test)]` insta snapshot test (`test_bc_1_14_004_three_register_snapshot`)
   will need `cargo insta review` to accept the snapshot on first green run.
