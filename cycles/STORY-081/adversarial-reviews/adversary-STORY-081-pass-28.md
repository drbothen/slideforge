---
document_type: adversary-pass-report
story_id: STORY-081
pass: 28
branch_head_at_review: f047348c
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: true
verdict_clean_pr_merge: true
streak_after: "1/3"
findings_count: 0
findings_crit: 0
findings_high: 0
findings_med: 0
findings_low: 0
findings_obs: 1
date: 2026-06-10
---

# Adversary Pass 28 — STORY-081

**CLEAN (strict): yes / streak 1/3**

## Verdict

CLEAN (strict): yes
CLEAN (PR-merge): yes
Streak after this pass: 1/3

## Scope

Fresh re-derivation at code HEAD f047348c, BC-3.05.001 v1.4.3, develop_base cbebfd57 (develop target for rebase: 15838de1).

## Summary

Verified the F-P27-MED-001 fix (chunks_to_markup_source correct for all 13 TemplateChunk variants; slide_title shows markup form distinct from stripped_text per BC EC-011 canonical vector). Confirmed:

- `chunks_to_markup_source` is pub(crate) and handles all 13 TemplateChunk variants with canonical DSL delimiters per DIR-077-002 §1.
- Red-Gate tests assert markup-delimiters-present + slide_title != stripped_text + rendered diagnostic message differs for bold/italic/code title inputs.
- AC-006 test strengthened to assert rendered message content, not just error-code presence.
- 14 unit tests for chunks_to_markup_source cover all variant arms.
- "for now" rationalization comments removed.
- No regression on any other seam. Holistic re-derivation across all 13 seams (eval, PPTX body/notes, DOCX, HTML, PDF, layout, depth-bound, escaping, security, determinism, error recovery, e2e cross-format) found no new gap.

## Findings

None (0 findings of any severity meeting strict-CLEAN threshold).

## Observations (non-blocking)

**OBS-P28-001** [cosmetic]: stale `_stub_exists` test-name in one test module — a holdover from the initial stub phase. Non-blocking. Does not affect behavior or spec compliance. Carried for PR-step cosmetic cleanup.

## Novelty

ZERO. No new architectural concerns discovered.
