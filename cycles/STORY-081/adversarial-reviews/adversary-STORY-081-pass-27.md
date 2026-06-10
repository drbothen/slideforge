---
document_type: adversary-pass-report
story_id: STORY-081
pass: 27
date: 2026-06-10
branch: feature/STORY-081
branch_head_at_review: b97386a0
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

# Adversary Pass 27 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (reset from 1/3).**

Fresh re-derivation at HEAD b97386a0, deep-pass focus on whole-diff hygiene + public-API surface + CLAUDE.md conventions. Whole-diff hygiene otherwise clean (no dbg!/eprintln!/todo! in STORY-081 surface; #[allow(struct_excessive_bools)] on OoxmlRun documented; public API rustdoc-complete + Hash+Eq+Clone; OoxmlRun.text:String is a serialization intermediate not IR — acceptable). One MED.

## Findings

### F-P27-MED-001 [MEDIUM] — EvalError::InlineMarkupInTitle.slide_title carried stripped text (not the markup form); diagnostic message shows identical before/after; contradicts BC EC-011 canonical vector + field doc
- for_eval.rs:350,361-368 set BOTH slide_title and stripped_text to the same stripped value → miette message renders "Inline markup in title field: 'Bold Title' … Stripped to: 'Bold Title'" (identical, misleading). Field doc (error.rs:380-387) says slide_title is "as it appeared (may include markup)". BC EC-011 + canonical vector require slide_title:"**Bold Title**", stripped_text:"Bold Title". Author left a "for now/for simplicity/minimum" rationalization comment (production-grade-default smell). No test asserted slide_title != stripped_text (presence-only).
- REMEDIATED (commit f047348c): added `chunks_to_markup_source` (pub(crate), all 13 TemplateChunk variants, canonical DSL delimiters per DIR-077-002 §1) reconstructing the markup form; slide_title now shows "**Bold Title**" distinct from stripped_text "Bold Title". Red-Gate tests (bold/italic/code) assert markup-delimiters-present + slide_title!=stripped_text + rendered message differs; strengthened AC-006 test to assert distinguishing fields; 14 unit tests for chunks_to_markup_source; "for now" comments removed. SourceSpan::default() left as-is (full span threading needs STORY-012 SourceMap — documented systematic gap, separate concern).

## Observations

### OBS-P27-001 [process-gap] — AC-006 eval test asserted diagnostic presence-by-error-code only, no distinguishing payload-field assertion → let F-P27-MED-001 survive 26 passes. BC canonical-test-vector struct field values (EC-011 slide_title/stripped_text split) aren't mechanically pinned. "Diagnostic emitted" tests should assert the rendered message / distinguishing field values, not just the error code. → lessons-codification.

### OBS-P27-002 — whole-diff hygiene + public-API + conventions clean (re-confirmed).

## Trajectory
...->P25(remediated)->P26(0,1/3)->P27(1MED, remediated, reset 0/3)

## Next Step
Fix landed f047348c. Restart cascade — adversary Pass 28 fresh.
