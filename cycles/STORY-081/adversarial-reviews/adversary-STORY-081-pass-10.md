---
document_type: adversary-pass-report
story_id: STORY-081
pass: 10
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: c739d59c84f54f8e779c45817b606194a3e99ca4
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 1
findings_crit: 0
findings_high: 1
findings_med: 0
findings_low: 0
findings_obs: 1
findings_open: 0
findings_remediated: 1
---

# Adversary Pass 10 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (RESET).**

Fresh-context independent re-derivation at HEAD c739d59c. Prior passes (P2-P9) converged the eval/layout/title-leg/body-span axes, independently re-confirmed sound. A fresh examination of the PPTX `InlineNode::Highlight` body-rendering seam — which no prior pass and no test ever exercised — surfaced a genuine rendering-correctness defect (ADV-P10-HIGH-001) built on a factually-false premise repeated across four code comments, with zero distinguishing test coverage. Resets the streak.

## Important Findings

### ADV-P10-HIGH-001 [HIGH] — PPTX body `InlineNode::Highlight` uses `<a:solidFill>` (glyph color) instead of `<a:highlight>`; highlighted text renders as yellow glyphs (illegible on light backgrounds), not a yellow highlight
- Where: `crates/slideforge-pptx/src/slide_serializer.rs` — Highlight arm 962-975, make_run 1031-1044, false-premise comments 836-837/876-878/963-965/1033.
- DrawingML `a:rPr/a:solidFill` is the glyph (foreground) fill — recolors the TEXT yellow, illegible on light backgrounds; does NOT produce a background highlight. Violates BC-3.02.002 PC8.
- Premise FALSE: ooxmlsdk =0.6.1 drawingml schema line 8143 exposes `pub a_highlight: Option<Box<Highlight>>` on `RunProperties`. Correct fix was available throughout. EC-008 warn-degrade branch never triggered (schema does not lack the element), so neither EC-008 branch satisfied.
- Sibling inconsistency (TD-VSDD-060): DOCX uses `<w:highlight>` (document_body.rs:558-569); HTML uses `<mark>`. Only PPTX body diverged.
- Paper-fix/no-distinguishing-output (TD-VSDD-059): no test anywhere asserted body Highlight output; survived P2-P9.
- REMEDIATED: see Remediation section.

## Observations

### OBS-P10-001 [OBS — carried forward → RESOLVED in fix burst]
slideforge-diagrams cold_budget wall-clock timing gate + double `load_system_fonts` per PDF export (carried from OBS-P09-001). Did not affect streak. RESOLVED in the fix burst (see Remediation).

## Remediation (post-Pass-10 fix burst)
- ADV-P10-HIGH-001 CLOSED — commit 0b2ce53e: `make_run` highlight branch now emits `rpr.a_highlight = Some(Box::new(Highlight { HighlightChoice::ASrgbClr(FFFF00) }))`; four false comments corrected; load-bearing Red-Gate test `test_adv_p10_high_001_highlight_uses_a_highlight_not_solid_fill` (inline_markup_tests.rs) asserts `<a:highlight>` present + `<a:solidFill>` absent (failed against old path, passes now).
- OBS-P10-001 RESOLVED — commit 49b1fc8e: root cause was a REAL double-load defect in slideforge-pdf/src/font.rs (`load_system_fonts` called twice per `resolve_font_set` — in resolve_font_set AND resolve_regular_face). Fixed to load once and share `&fontdb::Database`. Flaky cold_budget test replaced with deterministic `font_db_load_count == 1`. Commit f7c26fba: normalize_under_budget warm-path wall-clock test also converted to deterministic load-count invariance. Perf budget remains gated by criterion benches warm_render.rs (NFR-004) + cold_render.rs (NFR-003).

## Trajectory
P2(5)->P3(3)->P4(2)->P5(1)->P6(1MED+2OBS)->P7(0,1/3)->P8(1HIGH,reset)->P9(0,1/3)->P10(1HIGH,reset->0/3)

## Next Step
Fix landed at new HEAD f7c26fba. Restart LOCAL 3-CLEAN cascade from 0/3 — adversary Pass 11 (fresh context) at HEAD f7c26fba.
