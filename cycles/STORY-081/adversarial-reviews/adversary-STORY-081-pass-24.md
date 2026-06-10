---
document_type: adversary-pass-report
story_id: STORY-081
pass: 24
date: 2026-06-10
branch: feature/STORY-081
branch_head_at_review: 2e13fb6e
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
findings_obs: 4
findings_open: 0
findings_remediated: 1
---

# Adversary Pass 24 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (reset from 1/3).**

Fresh re-derivation at HEAD 2e13fb6e (code FROZEN — this finding is a BC-content defect in .factory/, consistent with LESSON-7 no-code-change-between-clean-passes). Deep-pass focus: rebase-readiness, error-taxonomy, public-API/docs, determinism. One MED (BC-content drift).

## Findings

### F-P24-MED-001 [MEDIUM] — BC-3.05.001 PC-1 + EC-004 describe aspirational Footnote PPTX behavior the v1.0 impl does NOT do (same drift class as the v1.4.1 Math correction, left uncorrected for Footnote)

BC PC-1 (132) + EC-004 (397) said "Footnote -> presenter note annotation + superscript reference number." Actual (ooxml_runs.rs:430-442, default_formatter.rs:16,476-481): renders inner content INLINE + `tracing::debug!("Footnote marker numbering deferred")`; numbered marker DEFERRED to STORY-085 F-010. Footnote is outside STORY-081's tested AC scope (the 8 forms); the defect is BC prose not reflecting the documented deferral.

REMEDIATED (BC-3.05.001 v1.4.2): full 12-form x 5-surface reconciliation against the v1.0 impl — Footnote PC-1/EC-004 corrected to actual inline+deferred-STORY-085 behavior; ALSO corrected additional drift the audit surfaced: PC-3 DOCX Code (RunFonts Courier New, not rStyle CodeSpan); PC-5 Strike/Highlight/Link (plain-text-only in v1.0, no geometric annotation, deferred STORY-085); PC-5 Math (skipped entirely, deferred STORY-009/045); DOCX/PDF Math + Xref (plain text, deferred). All corrected clauses cite source lines + deferral stories.

## Observations

### OBS-P24-001 [rebase-readiness]

Rebase cbebfd57->15838de1 will hit shared-type seams: FieldValue::Inlines (types/slide.rs), FrameContent::SubtitleInlines + TextRun->Vec<InlineNode> (layout/types.rs) vs develop's STORY-072/082/088 (gradient/sectionLst/bullets). Exhaustive non-wildcard matches fail-loud -> guaranteed manual-merge surface at PR step. Not a convergence blocker; flag for rebase.

### OBS-P24-002

Body ooxml_run_to_ooxmlsdk emits empty <a:rPr/> for plain runs vs notes serialize_ooxml_run emits none; within PC-2 final-conversion tolerance, schema-valid. Non-finding.

### OBS-P24-003

Determinism confirmed sound (OoxmlRun/InlineNode Hash+Eq; document-order DFS; symmetric rId resolver).

### OBS-P24-004

BC InlineDepthExceeded example was 2-field; code is 3-field {source_slide_index, depth, max} + E-LAY-005. REMEDIATED in v1.4.2 (BC examples updated to 3-field + error code).

## Trajectory

...->P22(1MED remediated)->P23(0,1/3)->P24(1MED BC-content, remediated, reset 0/3)

## Next Step

BC reconciled to v1.4.2 (code unchanged at 2e13fb6e). Restart cascade — adversary Pass 25 fresh: verify the v1.4.2 reconciled clauses actually match the code, + full re-derivation.
