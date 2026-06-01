# Red Gate Log — STORY-073

**Story:** Layout: ContentBlock::Bullets → FrameContent::TextRun frame generation  
**BC:** BC-3.05.001 v1.3.4  
**Date:** 2026-06-01  
**Branch:** feature/S-073  
**Commit:** 09cdbdf0  

## Red Gate Result

**PASS** — All STORY-073 tests fail before implementation begins.

## Test Counts

| Category | Count |
|----------|-------|
| Total tests in slideforge-layout | 339 |
| Passing (pre-existing + spec-valid) | 320 |
| Failing (STORY-073 Red Gate) | 19 |
| Skipped | 0 |

## Failing Tests (STORY-073)

### lib.rs unit tests (calling layout::run end-to-end)

| Test Name | AC / EC | Failure Reason |
|-----------|---------|----------------|
| `test_bc_3_05_001_story073_ac001_bullets_produce_text_run_frames` | AC-001 | 0 TextRun frames for bullets; expected 2 |
| `test_bc_3_05_001_story073_ac001_bullets_source_order_preserved` | AC-001 | 0 TextRun frames; inline content not in any frame |
| `test_bc_3_05_001_story073_ac001_three_bullet_items_three_frames` | AC-001 | 0 TextRun frames; expected 3 |
| `test_bc_3_05_001_story073_ac002_xref_unknown_in_bullet_layout_run` | AC-002 | No XrefTargetNotFound warning; bullets not scanned |
| `test_bc_3_05_001_story073_ac002_canonical_vector_xref_in_bullet` | AC-002 | 0 warnings; expected 1 |
| `test_bc_3_05_001_story073_ac003_depth_exceeded_in_bullet_is_hard_error` | AC-003 | Returns Ok; expected Err(InlineDepthExceeded) |
| `test_bc_3_05_001_story073_ec003_nested_bullet_produces_frame_per_item` | EC-003 | 0 TextRun frames for nested bullet items |
| `test_bc_3_05_001_story073_ec004_xref_inside_bold_in_bullet_layout_run` | EC-004 | No warning for deep-nested xref in bullet |
| `test_bc_3_05_001_story073_ac_int1_well_formed_bullets_no_errors_no_warnings` | AC-INT-1 | 0 TextRun frames for 3 bullet items; expected 3 |

### tests/bullets_layout_integration.rs (integration tests, AC-INT-1)

| Test Name | AC / EC | Failure Reason |
|-----------|---------|----------------|
| `test_bc_3_05_001_story073_ac_int1_bullets_end_to_end` | AC-INT-1 | 0 TextRun frames on bullet slide; expected 3 |
| `test_bc_3_05_001_story073_ac_int1_text_run_carries_inlines_verbatim` | AC-001 | No TextRun frame found with bullet inlines |
| `test_bc_3_05_001_story073_ac_int1_unknown_xref_in_bullet_warns` | AC-002 | 0 warnings; expected 1 XrefTargetNotFound |
| `test_bc_3_05_001_story073_ac_int1_depth_65_bullet_is_hard_error` | AC-003 | Returns Ok; expected Err(InlineDepthExceeded) |
| `test_bc_3_05_001_story073_ec002_empty_bullet_item_inlines_one_frame_no_error` | EC-002 | 0 TextRun frames; expected 1 |
| `test_bc_3_05_001_story073_ec003_nested_bullet_produces_frame_per_item` | EC-003 | 0 TextRun frames; expected 2 |
| `test_bc_3_05_001_story073_ec004_deep_nested_xref_in_bullet_layout_run` | EC-004 | 0 warnings |
| `test_bc_3_05_001_story073_ec005_multiple_depth_exceeded_bullets_returns_error` | EC-005 | Returns Ok |
| `test_bc_3_05_001_story073_ac_int1_mixed_text_and_bullets_blocks` | AC-INT-1 | 1 TextRun frame (from Text); expected 4 (1+3) |
| `test_bc_3_05_001_story073_vp047_all_12_inline_variants_in_bullet_survive_layout` | VP-047 | No TextRun frame with 12 variants |

## Tests Passing at Red Gate (STORY-073)

These pass at Red Gate for documented structural reasons:

| Test | Reason |
|------|--------|
| `inline::tests::test_bc_3_05_001_story073_ac002_xref_unknown_in_bullet_warns` | Tests `run_inline_validation` directly with pre-built TextRun frames — validation logic works on frames; tests verify logic not frame production |
| `inline::tests::test_bc_3_05_001_story073_ec004_xref_inside_nested_bold_in_bullet_warns` | Same as above |
| `inline::tests::test_bc_3_05_001_story073_ac003_depth_exceeded_in_bullet_is_error` | Same as above |
| `inline::tests::test_bc_3_05_001_story073_ec002_empty_bullet_inlines_no_warnings` | Same as above |
| `inline::tests::test_bc_3_05_001_story073_ec005_depth_exceeded_is_hard_error_not_warning` | Same as above |
| `tests::test_bc_3_05_001_story073_ec001_empty_bullet_list_no_frames_no_error` | Empty list → 0 frames → assert 0 is trivially satisfied pre-implementation; regression guard post-implementation |
| `tests::test_story073_no_exporter_crate_dependency_is_compile_verified` | Compile-time architectural constraint; passes by virtue of crate building |
| `bullets_layout_integration::test_bc_3_05_001_story073_ec001_empty_bullets_no_frames_no_error_no_warning` | Same as EC-001 above |
| `bullets_layout_integration::test_bc_3_05_001_story073_bullet_text_run_bboxes_are_valid` | No bullet TextRun frames → frame loop iterates nothing → vacuously true; becomes load-bearing post-implementation |

## Files Written

| File | Location | Description |
|------|----------|-------------|
| `inline.rs` (extended) | `crates/slideforge-layout/src/inline.rs` | 5 unit tests for `run_inline_validation` bullet scanning logic |
| `lib.rs` (extended) | `crates/slideforge-layout/src/lib.rs` | 9 unit tests calling `layout::run` end-to-end with bullet blocks |
| `bullets_layout_integration.rs` (new) | `crates/slideforge-layout/tests/bullets_layout_integration.rs` | 10 integration tests (AC-INT-1) |

## Spec Ambiguities / Notes

**BulletItem field names:** The story spec fixture code (AC-001, lines 91-96) uses
`.content` and `depth: 0` fields on `BulletItem`. The actual production type in
`slideforge-types/src/block.rs` uses `.inlines: Vec<InlineNode>` and
`.children: Vec<BulletItem>` (no depth field; nesting is via children). Tests
use the real API. The BC is the source of truth for contract semantics (CLAUDE.md
precedence rule 1).

**`run_inline_validation` inline tests pass:** The 5 inline.rs unit tests for
STORY-073 pass at Red Gate because they test the validation logic by constructing
`LaidOutSlide` with `FrameContent::TextRun` frames directly — bypassing the missing
`ContentBlock::Bullets → FrameContent::TextRun` frame-production step. This is
intentional: these tests verify the validation logic is correct; the integration
tests (failing) verify that `layout::run` produces the frames.

## Implementer Instructions

Make each failing test pass in this order:

1. **AC-001 (frame generation):** Extend the inline-text pass in `layout.rs`
   (~line 252) to also iterate `ContentBlock::Bullets` blocks. For each
   `BulletItem`, recursively flatten the item and its `.children` into
   `FrameContent::TextRun` frames (one per item, in traversal order).
   Use the same bbox clamping pattern as the Text block pass.

2. **AC-002 / AC-003 (validation):** Once frames are produced, `run_inline_validation`
   will automatically scan them (it already handles `FrameContent::TextRun`).
   No changes to `inline.rs` needed for validation — the existing logic handles it.

3. Verify the rustdoc change in `slideforge-types/src/inline.rs`:
   update "Exactly 11 variants" to "Exactly 12 variants" with STORY-073 citation.
