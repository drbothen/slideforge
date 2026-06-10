---
document_type: adversary-pass-report
story_id: STORY-081
pass: 22
date: 2026-06-10
branch: feature/STORY-081
branch_head_at_review: 1ef3dda3
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 1
findings_crit: 0
findings_high: 0
findings_med: 1
findings_low: 0
findings_obs: 2
findings_open: 0
findings_remediated: 1
---

# Adversary Pass 22 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (reset from 1/3).**

Fresh re-derivation at HEAD 1ef3dda3, deep-pass focus on AC/DoD coverage + degenerate inputs. Confirmed all 6 ACs load-bearing-tested EXCEPT the AC-004 direction half; eval→layout→export dual-title shadow flow correct across DOCX/PDF/HTML with correct source_slide_index (no off-by-one); unified engine (Inv-9) + HI-1..HI-5 hold by construction; depth-bound enforced at parser(E-PAR-021)+engine+layout; HTML escaping correct both positions. One MED.

## Findings

### ADV-P22-MED-001 [MEDIUM] — AC-004 / BC-3.05.001 PC-5 super/subscript baseline-DIRECTION half has no load-bearing test (untested path, TD-VSDD-059)

PC-5 + AC-004 specify a 2-part PASS: (a) reduced size font_size*0.583 AND (b) baseline-shift DIRECTION (super RAISED baseline_y - font_size*0.333; sub LOWERED + font_size*0.333). Size half (a) tested; direction half (b) — `pub fn adjusted_baseline_y` (slideforge-pdf/src/exporter.rs:1196, prod call :1453) — had ZERO test callers. A regression swapping super/sub Y-arms or zeroing the shift would pass all existing tests. Production code CORRECT — untested-path gap on a spec-mandated assertion.

REMEDIATED (commit a7700304): added load-bearing unit test `test_BC_3_05_001_ac004_pc5_superscript_baseline_raised_subscript_lowered` — super result STRICTLY < baseline_y, sub STRICTLY > baseline_y, normal (y_offset_units==0) exactly unchanged; uses named SUPER_RISE_FRACTION/SUB_DROP_FRACTION constants. Confirmed load-bearing (arm-swap → super=103.996 fails <100; zeroing → both 100.0 fail).

## Observations

### OBS-P22-001 — defense-in-depth depth-guard layering

title_inlines/SubtitleInlines bypass both layout TextRun guard and PPTX engine guard in DOCX/HTML recursion, but are PARSER-GATED at depth 64 (E-PAR-021) before eval — not reachable from user DSL. Layering note, NOT a live vuln. (Non-blocking; no fix required — programmatic-only depth-65 IR is not a v1.0 path.)

### OBS-P22-002 — carried observations

PDF PC-5 boundary; DOCX per-surface link semantics; FU-STORY-045 MathML; FU-PPTX-DUAL-RUN-GENERATOR resolved; re-anchor consistent.

## Trajectory

...->P20(2HIGH+1MED remediated)->P21(0,1/3)->P22(1MED, remediated, reset 0/3)

## Next Step

AC-004 test added (a7700304). ALSO: orchestrator found + cleaned a residual of the incomplete Pass-20 sweep — the old BC_3_02_002 HTML snapshot was still git-tracked (deletion uncommitted; the prior .rs-scoped completeness greps missed the .snap). git-rm'd + committed (2e13fb6e); all-filetype grep + insta --unreferenced=reject confirm zero residuals. Restart cascade — adversary Pass 23 fresh at 2e13fb6e.
