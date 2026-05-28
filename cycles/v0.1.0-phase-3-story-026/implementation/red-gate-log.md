---
story_id: STORY-026
phase: red-gate
date: 2026-05-27
agent: test-writer
status: VERIFIED
---

# Red Gate Log — STORY-026: Core Layout (Deck → LaidOutDeck)

## Summary

Red Gate verified. All tests that exercise `layout::run()` fail with
`todo!()` panic. All supporting type, error, region, and text-flow tests
pass (they test the stubs themselves, not the unimplemented run() function).

## Build Status

```
cargo build -p slideforge-layout
```
Result: **CLEAN** (0 errors, 0 warnings after fixing unused import)

## Test Run

```
cargo test -p slideforge-layout --no-fail-fast
```
Result: **34 passed; 20 failed** (Red Gate confirmed)

## Failing Tests (20) — All Expected

All failures are for the right reason: `layout::run()` panics with `todo!()`.

### lib.rs tests (15 failing)

| Test | BC/AC | Fail Reason |
|------|-------|-------------|
| `test_bc_3_06_001_layout_run_returns_result` | AC-001 | `todo!()` panic |
| `test_bc_3_06_001_layout_run_preserves_slide_count` | AC-002/BC-3.06.001 | `todo!()` panic |
| `test_bc_3_06_001_layout_run_empty_deck_error` | EC-001 | `todo!()` panic |
| `test_bc_3_06_002_layout_run_deterministic` | BC-3.06.002 | `todo!()` panic |
| `test_bc_3_06_001_layout_run_default_page_size` | AC-004 | `todo!()` panic |
| `test_bc_3_06_001_layout_run_page_size_from_brand` | AC-004 | `todo!()` panic |
| `test_bc_3_06_003_layout_run_all_frames_valid_bounding_boxes` | AC-014/BC-3.06.003 | `todo!()` panic |
| `test_bc_3_06_001_laid_out_slide_source_index_correct` | AC-005 | `todo!()` panic |
| `test_bc_3_06_001_laid_out_slide_type_keyword_matches` | AC-005 | `todo!()` panic |
| `test_bc_3_06_002_title_slide_has_frames` | AC-006 | `todo!()` panic |
| `test_bc_3_06_002_blank_slide_has_zero_frames` | AC-006 | `todo!()` panic |
| `test_bc_3_06_002_unknown_slide_type_returns_error` | EC-002 | `todo!()` panic |
| `test_bc_3_06_001_laid_out_deck_implements_hash_eq_clone` | AC-010 | `todo!()` panic |
| `test_vp_011_layout_run_preserves_slide_count_proptest` | VP-011 | `todo!()` panic |
| `test_vp_011_layout_run_is_deterministic_proptest` | BC-3.06.002 | `todo!()` panic |

### regions.rs snapshot tests (5 failing)

| Test | BC/AC | Fail Reason |
|------|-------|-------------|
| `test_bc_3_06_002_title_slide_regions_snapshot` | AC-006 | No stored snapshot (first run) |
| `test_bc_3_06_002_content_slide_regions_snapshot` | AC-006 | No stored snapshot (first run) |
| `test_bc_3_06_002_section_break_regions_snapshot` | AC-006 | No stored snapshot (first run) |
| `test_bc_3_06_002_stat_callout_regions_snapshot` | AC-006 | No stored snapshot (first run) |
| `test_bc_3_06_002_chart_regions_snapshot` | AC-006 | No stored snapshot (first run) |

Note: Snapshot test failures are expected Red Gate behavior on first run.
The implementer must run `cargo insta review` to accept the initial snapshots
after verifying the region map values are correct.

## Passing Tests (34) — All Expected

Passing tests cover the stubs that can be validated without `layout::run()`:

- **error.rs** (5 passing): All `LayoutError` variants can be constructed, cloned, hashed, and display correct messages.
- **regions.rs** (8 passing): Region maps for all 31 slide types return valid `BoundingBox` values; unknown keyword returns `None`.
- **text_flow.rs** (7 passing): `compute_text_flow` correctly handles empty text, short text, overflowing text, and custom char widths.
- **types.rs** (14 passing): All layout IR types (`LaidOutDeck`, `LaidOutSlide`, `Frame`, `BoundingBox`, `TextFlow`, `PageSize`, `TextOverflow`, `FrameContent`) implement `Hash + Eq + Clone + Debug`; constants are correct.

## BC Coverage

| BC | AC | Test(s) | Status |
|----|-----|---------|--------|
| BC-3.06.001 | AC-001 | `test_bc_3_06_001_layout_run_returns_result` | FAILING (Red Gate) |
| BC-3.06.001 | AC-002 | `test_bc_3_06_001_layout_run_preserves_slide_count` | FAILING (Red Gate) |
| BC-3.06.001 | EC-001 | `test_bc_3_06_001_layout_run_empty_deck_error` | FAILING (Red Gate) |
| BC-3.06.001 | AC-004 | `test_bc_3_06_001_layout_run_default_page_size` | FAILING (Red Gate) |
| BC-3.06.001 | AC-004 | `test_bc_3_06_001_layout_run_page_size_from_brand` | FAILING (Red Gate) |
| BC-3.06.001 | AC-005 | `test_bc_3_06_001_laid_out_slide_source_index_correct` | FAILING (Red Gate) |
| BC-3.06.001 | AC-005 | `test_bc_3_06_001_laid_out_slide_type_keyword_matches` | FAILING (Red Gate) |
| BC-3.06.001 | AC-010 | `test_bc_3_06_001_laid_out_deck_implements_hash_eq_clone` | FAILING (Red Gate) |
| BC-3.06.002 | AC-013 | `test_bc_3_06_002_layout_run_deterministic` | FAILING (Red Gate) |
| BC-3.06.002 | AC-006 | `test_bc_3_06_002_title_slide_has_frames` | FAILING (Red Gate) |
| BC-3.06.002 | AC-006 | `test_bc_3_06_002_blank_slide_has_zero_frames` | FAILING (Red Gate) |
| BC-3.06.002 | EC-002 | `test_bc_3_06_002_unknown_slide_type_returns_error` | FAILING (Red Gate) |
| BC-3.06.003 | AC-014 | `test_bc_3_06_003_layout_run_all_frames_valid_bounding_boxes` | FAILING (Red Gate) |
| VP-011 | proptest | `test_vp_011_layout_run_preserves_slide_count_proptest` | FAILING (Red Gate) |

## Files Created

| File | Purpose |
|------|---------|
| `crates/slideforge-layout/Cargo.toml` | Updated with correct deps (slideforge-types, slideforge-plugin-api, thiserror, insta, proptest) |
| `crates/slideforge-layout/src/lib.rs` | Crate root with pub modules + lib.rs tests (15 failing + 2 proptest) |
| `crates/slideforge-layout/src/layout.rs` | `layout::run` stub returning `todo!()` |
| `crates/slideforge-layout/src/types.rs` | Layout IR types: LaidOutDeck, LaidOutSlide, Frame, BoundingBox, TextFlow, PageSize, FrameContent, TextOverflow |
| `crates/slideforge-layout/src/error.rs` | LayoutError enum with thiserror |
| `crates/slideforge-layout/src/regions.rs` | Region maps for all 31 slide types |
| `crates/slideforge-layout/src/text_flow.rs` | TextFlow computation (compile-time stub, logic implemented) |

## Implementation Instructions for Implementer

Make each test pass by implementing `layout::run()` in
`crates/slideforge-layout/src/layout.rs`. The function must:

1. Return `Err(LayoutError::EmptyDeck)` when `deck.slides.is_empty()`
2. Derive `PageSize` from `brand.layouts[0]` if present, otherwise use `PageSize::default()`
3. For each slide, call `regions::region_frames_for(slide.slide_type.as_ref(), page_w, page_h)`
   - Return `Err(LayoutError::UnknownSlideType)` if `None` is returned
4. Construct `LaidOutSlide` with `source_index`, `slide_type_keyword`, `frames`, `speaker_notes: None`
5. Run post-layout integrity check on each `BoundingBox`; return `Err(LayoutError::InvalidBoundingBox)` on violation
6. Verify `output.slides.len() == deck.slides.len()`; return `Err(LayoutError::SlideCountMismatch)` if not
7. After implementing, run `cargo insta review` to accept initial snapshots

All region map values are already implemented in `regions.rs`.
All type/error infrastructure is already implemented and passing.
The proptest library (proptest v1.5.0) is already added to dev-dependencies.
