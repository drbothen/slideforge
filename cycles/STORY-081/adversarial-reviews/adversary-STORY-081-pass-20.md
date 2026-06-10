---
document_type: adversary-pass-report
story_id: STORY-081
pass: 20
date: 2026-06-10
branch: feature/STORY-081
branch_head_at_review: 3b76a9ba
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 3
findings_crit: 0
findings_high: 2
findings_med: 1
findings_low: 0
findings_obs: 3
findings_open: 0
findings_remediated: 3
---

# Adversary Pass 20 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (held).**

Fresh re-derivation at HEAD 3b76a9ba. Verified Pass-19 reference-set helper + 2 multi-leaf export tests CORRECT + load-bearing. But re-deriving the "stale-name sweep complete" claim independently (not inheriting it) surfaced that the Pass-19 sweep was INCOMPLETE — a partial-fix (S-7.01).

## Findings

### F-P20-HIGH-001 [HIGH] — STORY-081 slide-level deliverables in OTHER crates still mis-anchored to BC-3.02.002 PC8 (incomplete sweep)

Pass-19 only swept 2 pptx test files. Still citing BC_3_02_002/PC8: slideforge-eval/src/tests/slide_inline_markup_eval_tests.rs (table + 16 fn names), for_eval.rs (69/95/297), eval.rs (2631), slideforge-pdf inline_markup_pdf_snapshot.rs (table + ac004 fns) + tag_engine.rs (468), slideforge-html inline_markup_html_snapshot.rs (table + ac005 fns), slideforge-docx inline_markup_docx_snapshot.rs (table + ac003 fns). BC-3.02.002 (section blocks) PC8 EXCLUDES slide-level — mis-anchoring misdirects maintainers; Semantic-Anchoring-Audit: mis-anchor always blocks convergence. Same-document contradiction (docx production cites BC-3.05.001 PC-3 while sibling snapshot cites BC-3.02.002 PC8) → HIGH.

**REMEDIATED (commit 1ef3dda3):** swept BC_3_02_002→BC_3_05_001, PC8→correct clause (AC-001→precondition 5/Inv-10; AC-003→PC-3; AC-004→PC-5; AC-005→PC-4; AC-006→Slide-Level Title Constraint/EC-011) across all STORY-081 deliverable files (eval tests+for_eval+eval+error, pdf snapshot+tag_engine+slide_pdf, html snapshot+snap rename, docx snapshot, types shape_types, e2e). Genuine STORY-077/078 section-block refs preserved. Exit-gate grep: zero STORY-081 slide-level stale refs remain.

### F-P20-HIGH-002 [HIGH] — sibling doc-comments assert the retired false count-equality ∀ invariant (partial-fix S-7.01(b))

notes_tests.rs:3049/3062/1529/2702/2881 still claimed `external_rel_count == hlinkclick_count ∀` — FALSE per HI-1, contradicting the Pass-19-corrected slide_serializer.rs:949/1038.

**REMEDIATED (commit 1ef3dda3):** 5 sites rewritten to HI-1 reference-set wording.

### F-P20-MED-001 [MED] — surviving count-equality assertions in notes_tests.rs (HI-1 forbids the form)

7 `assert_eq!(external_rel_count, hlinkclick_count)` at notes_tests.rs (2937/3016/3109/3188/3255/3318/3398). Single/zero-leaf (not false-green) but BC HI-1 forbids the form.

**REMEDIATED (commit 1ef3dda3):** replaced with assert_hyperlink_reference_set_invariant; specific count assertions preserved.

## Observations

### OBS-P20-1 [process-gap] — re-anchor/stale-name sweeps must include a COMPLETENESS GREP exit gate

Re-anchor and stale-identifier sweeps MUST run a completeness grep (old token scoped to the story's deliverable file set must return zero) as a required exit gate before being declared done. The Pass-19 sweep + its verification both reported "complete" while 5+ files retained the old anchor. S-7.01(c). → lessons-codification.

### OBS-P20-2 — HTML escaping verified correct (no XSS). PDF PC-5 boundary carried.

### OBS-P20-3 — Process lesson: "sweep declared done" claims require a grep exit gate citation in the commit message or remediation record. No grep citation = sweep incomplete by default.

## Trajectory

...->P18(0,1/3)->P19(1MED, remediated, reset)->P20(2HIGH+1MED, remediated, reset 0/3)

## Next Step

All remediated at HEAD 1ef3dda3 (test/doc/naming only; no production behavior change). Restart cascade — adversary Pass 21 fresh at 1ef3dda3.
