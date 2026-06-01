---
document_type: demo-evidence-report
product: "slideforge"
story_id: "STORY-073"
title: "Layout: ContentBlock::Bullets → FrameContent::TextRun frame generation"
pipeline_run: "2026-06-01"
demo_type: "library"
recording_tool: "vhs + cargo-nextest"
status: complete
---

# Demo Evidence Report — STORY-073: Layout — ContentBlock::Bullets → FrameContent::TextRun

**Story:** STORY-073 — Layout: ContentBlock::Bullets → FrameContent::TextRun frame generation
**Behavioral Contracts:** BC-3.05.001 v1.3.5
**Verification Properties:** VP-045 (depth-65 produces InlineDepthExceeded), VP-047 (all 12 variants survive layout)
**Date:** 2026-06-01
**Evidence type:** VHS recordings of cargo nextest runs (Rust library crate — no interactive CLI or UI)
**Crate under test:** `slideforge-layout`

---

## Rationale: VHS Recordings of nextest Output

STORY-073 delivers pure Rust library modules:
- `crates/slideforge-layout/src/lib.rs` — `ContentBlock::Bullets` iteration → `FrameContent::TextRun` frame generation in `layout::run()`
- `crates/slideforge-layout/src/inline.rs` — `run_inline_validation` extended to traverse `BulletItem.content` fields (xref + depth bound)
- `crates/slideforge-layout/tests/bullets_layout_integration.rs` — AC-INT-1 integration tests

There is no user-facing CLI command that exercises these paths in isolation at this pipeline stage.
All acceptance criteria are verified via unit and integration tests. VHS recordings capture
`cargo nextest` invocations against each AC's test filter, making the test output visually
auditable in the PR diff.

**Overall result: 344/344 slideforge-layout tests PASS. 4/4 ACs demonstrated. 33/33 STORY-073-specific tests pass.**

```
cargo nextest run -p slideforge-layout --no-fail-fast

────────────
 Summary [0.671s] 344 tests run: 344 passed, 0 skipped
```

---

## Per-AC Evidence

| AC | Title | Recording | Tests Demonstrated | Result |
|----|-------|-----------|-------------------|--------|
| AC-001 | ContentBlock::Bullets produces one FrameContent::TextRun frame per BulletItem in source order | AC-001-bullets-produce-textrun-frames.gif | `test_bc_3_05_001_story073_ac_int1_bullets_end_to_end` (integration end-to-end, 3-item list → 3 frames, slide count preserved, zero warnings) | PASS |
| AC-002 | Xref validation scans bullet inline content — unknown target → XrefTargetNotFound warning | AC-002-xref-warning-in-bullet.gif | `test_bc_3_05_001_story073_ac_int1_unknown_xref_in_bullet_warns` (unknown xref "missing-slide" in bullet → exactly 1 XrefTargetNotFound warning, target and slide_index verified) | PASS |
| AC-003 | Inline depth bound (64) enforced in bullet content — depth-65 tree → InlineDepthExceeded hard error | AC-003-depth-65-hard-error.gif | `test_bc_3_05_001_story073_ac_int1_depth_65_bullet_is_hard_error` (65-deep Bold tree in bullet → Err, error variant = InlineDepthExceeded{depth:65, max:64, source_slide_index:0}) | PASS |
| AC-INT-1 | layout::run end-to-end integration with ContentBlock::Bullets — all paths | AC-INT-1-end-to-end-bullets-layout.gif | All 13 integration tests in `bullets_layout_integration` module — covers AC-001, AC-002, AC-003, EC-001 through EC-005, VP-047 (all 12 variants), mixed Text+Bullets blocks, bbox validity | PASS |

---

## Integration Test Coverage Detail (AC-INT-1)

The `AC-INT-1-end-to-end-bullets-layout.gif` recording captures all 13 integration tests passing:

| Test | AC / EC Covered | What It Proves |
|------|----------------|----------------|
| `test_bc_3_05_001_story073_ac_int1_bullets_end_to_end` | AC-001, AC-INT-1 | 3-item bullet list → 3 TextRun frames; slide count preserved; zero warnings |
| `test_bc_3_05_001_story073_ac_int1_text_run_carries_inlines_verbatim` | AC-001, AC-INT-1 | InlineNode sequences appear verbatim in FrameContent::TextRun (no transform at layout time) |
| `test_bc_3_05_001_story073_ac_int1_unknown_xref_in_bullet_warns` | AC-002, AC-INT-1 | Unknown xref in bullet → 1 XrefTargetNotFound, target="missing-slide", slide_index=0 |
| `test_bc_3_05_001_story073_ac_int1_depth_65_bullet_is_hard_error` | AC-003, AC-INT-1, VP-045 | 65-deep Bold tree → Err(InlineDepthExceeded{depth:65, max:64}) |
| `test_bc_3_05_001_story073_ac_int1_mixed_text_and_bullets_blocks` | AC-INT-1 | ContentBlock::Text (1 item) + ContentBlock::Bullets (3 items) → 4 TextRun frames total |
| `test_bc_3_05_001_story073_ec001_empty_bullets_no_frames_no_error_no_warning` | EC-001 | ContentBlock::Bullets(vec![]) → 0 frames, no error, no warning |
| `test_bc_3_05_001_story073_ec002_empty_bullet_item_inlines_one_frame_no_error` | EC-002 | BulletItem{inlines:vec![]} → 1 frame (empty inline sequence), no error |
| `test_bc_3_05_001_story073_ec003_nested_bullet_produces_frame_per_item` | EC-003 | 1 parent + 1 child bullet → 2 frames; parent and child inlines present |
| `test_bc_3_05_001_story073_ec003_integration_frame_order_parent_before_child` | EC-003 | 3-level nested bullet (parent/child/grandchild) → frames in depth-first parent-before-child order (OBS-3 load-bearing positional assertion) |
| `test_bc_3_05_001_story073_ec004_deep_nested_xref_in_bullet_layout_run` | EC-004 | Xref inside Bold(Italic(Xref)) in bullet → XrefTargetNotFound still accumulated |
| `test_bc_3_05_001_story073_ec005_multiple_depth_exceeded_bullets_returns_error` | EC-005 | Two depth-65 bullets on same slide → Err(InlineDepthExceeded) (hard error, not suppressed) |
| `test_bc_3_05_001_story073_vp047_all_12_inline_variants_in_bullet_survive_layout` | VP-047 | All 12 InlineNode variants in bullet inlines appear verbatim in TextRun frame; zero warnings |
| `test_bc_3_05_001_story073_bullet_text_run_bboxes_are_valid` | BC-3.06.003 | Bullet TextRun frame bounding boxes are valid (non-negative, non-zero, within page bounds; integer EMU) |

---

## Architecture Compliance Verified

- **No exporter crates in slideforge-layout**: `test_story073_no_exporter_crate_dependency_is_compile_verified` confirms no forbidden dependency (the crate compiles without pptx/docx/pdf/html).
- **Integer EMU coordinates (DI-010, ADR-013)**: Bounding box test uses `Emu(i64)` values, not `f64`.
- **All 12 InlineNode variants exhaustive**: VP-047 test enumerates all 12 variants explicitly; missing a variant would be a compile error.
- **Xref validation scope includes bullets**: AC-002 and EC-004 confirm the validation pass is not silently skipped for bullet content.
