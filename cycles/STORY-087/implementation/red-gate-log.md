---
document_type: red-gate-log
story: STORY-087
pass: 2
agent: test-writer
timestamp: 2026-06-06
status: VERIFIED
---

# Red Gate Log — STORY-087 Pass-2

## Summary

IR-type stubs added; TD-VSDD-060 sweep complete; behavioral Red Gate tests
written and verified FAILING. All new behavioral tests fail for the correct
reasons (threading/materialization/routing not implemented). No regressions
on existing tests. One pre-existing flake (`cold_budget` under STORY-080) tolerated.

## Part A — IR-Type Stubs Added

### New Types

| Type | Location | Status |
|------|----------|--------|
| `TextTag::ColorLabel` | `slideforge-types/src/block.rs` | Compile-only stub + tracing::warn no-op in layout.rs |
| `ContentBlock::ColorBar(ColorBarSpec)` | `slideforge-types/src/block.rs` | Compile-only stub |
| `ColorBarSpec { percent: u8 }` | `slideforge-types/src/block.rs` | Compile-only struct |
| `FrameContent::ColorBar { filled_width_emu, total_width_emu, color }` | `slideforge-layout/src/types.rs` | Compile-only stub |

### TD-VSDD-060 Sweep Sites Touched

All exhaustive matches across the workspace were updated:

| File | Match Type | Action |
|------|-----------|--------|
| `slideforge-layout/src/inline.rs` | `FrameContent` | Added `ColorBar { .. } => {}` |
| `slideforge-layout/src/lib.rs` | `FrameContent` | Added `ColorBar { .. } => String::new()` |
| `slideforge-layout/src/layout.rs` | `TextTag` (fill_region_slot_or_append) | Added `ColorLabel => RegionRole::Body` |
| `slideforge-layout/src/layout.rs` | `ContentBlock::Text` match | Added `ColorLabel => { tracing::warn!(...) }` (stub, no routing behavior) |
| `slideforge-pptx/src/a11y.rs` | `FrameContent` | Added `ColorBar { .. } => None` |
| `slideforge-pptx/src/slide_serializer.rs` | `FrameContent` | Added `ColorBar` stub with `tracing::warn!` |
| `slideforge-pdf/src/exporter.rs` | `FrameContent` (draw_frame) | Added `ColorBar` stub with `tracing::warn!` |
| `slideforge-pdf/src/exporter.rs` | `ContentBlock` (body_item_baselines) | Added `ColorBar(_) => {}` |
| `slideforge-pdf/src/exporter.rs` | `ContentBlock` (draw_body_blocks) | Added `ColorBar(_) => {}` |
| `slideforge-pdf/src/exporter.rs` | `ContentBlock` (tagged draw) | Added `ColorBar(_) => {}` |
| `slideforge-pdf/src/tag_engine.rs` | `FrameContent` (tag_slide) | Added `ColorBar` stub → Artifact |
| `slideforge-pdf/src/tag_engine.rs` | `ContentBlock` (tag_content_block) | Added `ColorBar(_) => Ok(None)` |
| `slideforge-validate/src/alt_text.rs` | `ContentBlock` | Added `ColorBar(_) => {}` |
| `slideforge-types/src/block.rs` | `kind_name()` | Added `ColorBar(_) => "ColorBar"` |
| `slideforge-types/src/block.rs` | `produces_structure_group()` | Added `ColorBar(_) => false` |

### Re-exports Added

- `ColorBarSpec` re-exported from `slideforge-types` crate root
- `RegionRole` re-exported from `slideforge-layout` crate root

## Part B — Failing Tests (Red Gate)

### §10.1 Stage-2b Threading Tests

File: `crates/slideforge-eval/tests/story_087_threading.rs`

| Test name | Expected | Red Gate reason |
|-----------|----------|----------------|
| `test_status_label_threads_as_color_label` | FAIL | Stage 2b has no `label` arm for `status` |
| `test_status_no_label_no_block` | PASS | Vacuous guard (no label → no block already correct) |
| `test_progress_bar_label_threads_as_color_label` | FAIL | Stage 2b has no `label` arm for `progress_bar` |
| `test_progress_bar_value_threads_as_color_bar` | FAIL | Stage 2b has no `value` arm |
| `test_progress_bar_value_zero_threads_as_color_bar` | FAIL | Same |
| `test_progress_bar_value_100_threads_as_color_bar` | FAIL | Same |
| `test_weighted_composite_label_threads_as_color_label` | FAIL | Stage 2b has no `label` arm for `weighted_composite` |
| `test_weighted_composite_2_components_produce_2_body_blocks` | FAIL | Stage 2b has no `components` arm |
| `test_weighted_composite_component_text_contains_name_score_label` | FAIL | Same |
| `test_stat_callout_stat1_threads_as_body` | FAIL | Stage 2b has no `stat_1` arm for `stat_callout` |

**Result: 9 FAIL, 1 PASS**

### §10.2 Layout Routing Tests

File: `crates/slideforge-layout/tests/story_087_layout_routing.rs`

| Test name | Expected | Red Gate reason |
|-----------|----------|----------------|
| `test_status_label_fills_body_slot` | FAIL | `TextTag::ColorLabel` arm is a no-op stub |
| `test_progress_bar_label_fills_body_slot` | FAIL | Same |
| `test_progress_bar_color_bar_width_proportional` | FAIL | No ColorBar materialization pass |
| `test_progress_bar_color_bar_value_0` | FAIL | Same |
| `test_progress_bar_color_bar_value_100` | FAIL | Same |
| `test_weighted_composite_agg_label_fills_body_slot` | FAIL | `ColorLabel` arm is a no-op stub |
| `test_weighted_composite_2_components_fill_2_generic_slots` | FAIL | ColorLabel no-op → 2 Body frames not 3 |
| `test_weighted_composite_5_components_fill_5_generic_slots` | FAIL | ColorLabel no-op causes incorrect counts |

**Result: 8 FAIL, 0 PASS**

### §10.3 ValueRangeValidator Empty-Components Test

File: `crates/slideforge-validate/src/value_range.rs` (in `#[cfg(test)]`)

| Test name | Expected | Red Gate reason |
|-----------|----------|----------------|
| `test_BC_1_17_003_empty_components_is_error` | FAIL | `validate_weighted_composite_components` silently returns for empty list |

**Result: 1 FAIL**

### §10.4 Build-Level Visible Output Tests

File: `crates/slideforge/tests/e2e/story_087_content_rendering.rs`

| Test name | Expected | Red Gate reason |
|-----------|----------|----------------|
| `test_AC_002_status_label_visible_in_output` | FAIL | ColorLabel no-op → no Body frame with label text |
| `test_AC_003_status_missing_label_is_error` | PASS | LabelCheck already implemented (STORY-087 pass-1) |
| `test_AC_005_status_missing_label_warn_only_is_ok` | PASS | Warn-only mode already works |
| `test_AC_008_progress_bar_label_and_bar_visible` | FAIL | No ColorLabel routing + no ColorBar materialization |
| `test_AC_015_weighted_composite_labels_visible` | FAIL | Same |
| `test_BC_1_17_003_build_weighted_composite_visible_output` | IGNORED | Pending STORY-088 DSL list-of-map parser |

**Result: 3 FAIL, 2 PASS, 1 IGNORED**

## Red Gate Evidence

```
cargo nextest run --workspace --no-fail-fast
Summary: 3385 tests run: 3364 passed, 21 failed, 17 skipped

Failing tests (new Red Gate — expected to fail):
  slideforge-eval::story_087_threading              (9 tests)
  slideforge-layout::story_087_layout_routing       (8 tests)
  slideforge-validate::value_range::test_BC_1_17_003_empty_components_is_error (1 test)
  slideforge::e2e_tests::e2e_story_087_content_rendering::test_AC_002 (1 test)
  slideforge::e2e_tests::e2e_story_087_content_rendering::test_AC_008 (1 test)
  slideforge::e2e_tests::e2e_story_087_content_rendering::test_AC_015 (1 test)
  slideforge-diagrams::cold_budget (1 test — pre-existing flake, STORY-080)

Total new Red Gate failures: 20
Pre-existing flakes: 1 (cold_budget, STORY-080)
```

## Implementer Instructions

Make each failing test pass ONE AT A TIME with minimum code:

1. **Stage 2b extension** (`slideforge-eval/src/field_to_block.rs`): Add dispatch for
   `status | progress_bar | weighted_composite | stat_callout` per adjudication §4.2.
   - Thread `label` → `ContentBlock::Text(TextTag::ColorLabel)` for all four types.
   - Thread `value` → `ContentBlock::ColorBar(ColorBarSpec { percent })` for `progress_bar`.
   - Thread `components` → one `ContentBlock::Text(TextTag::Body)` per component for `weighted_composite`.
   - Thread `stat_1`, `label_1`, `stat_2`, `label_2`, `stat_3`, `label_3` → `ContentBlock::Text(TextTag::Body)` for `stat_callout`.
   - Add `compose_component_row_text` pure helper.

2. **Layout ColorLabel routing** (`slideforge-layout/src/layout.rs`): Replace the
   `TextTag::ColorLabel => { tracing::warn!(... stub ...) }` no-op with the real
   `FrameContent::Body` routing (same as `TextTag::Body` arm but pass `TextTag::ColorLabel`).

3. **ColorBar materialization pass** (`slideforge-layout/src/layout.rs`): Add after the
   `fill_region_slot_or_append` loop per adjudication §4.4. Find the first Generic-role
   Empty frame, compute `filled_width_emu = (percent as i64 * total_width_emu.0) / 100`,
   use brand primary color or `Rgb { r: 0, g: 112, b: 192 }`.

4. **ValueRangeValidator empty-list check** (`slideforge-validate/src/value_range.rs`):
   Add empty-list arm per adjudication §7.
