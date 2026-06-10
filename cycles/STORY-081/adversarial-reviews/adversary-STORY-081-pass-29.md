---
document_type: adversary-pass-report
story_id: STORY-081
pass: 29
branch_head_at_review: f047348c
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: true
verdict_clean_pr_merge: true
streak_after: "2/3"
findings_count: 0
findings_crit: 0
findings_high: 0
findings_med: 0
findings_low: 0
findings_obs: 0
date: 2026-06-10
---

# Adversary Pass 29 — STORY-081

**CLEAN (strict): yes / streak 2/3**

## Verdict

CLEAN (strict): yes
CLEAN (PR-merge): yes
Streak after this pass: 2/3

## Scope

Fresh re-derivation at code HEAD f047348c, BC-3.05.001 v1.4.3, develop_base cbebfd57 (develop target for rebase: 15838de1). Comprehensive 13-seam independent re-derivation.

## Summary

Comprehensive 13-seam independent re-derivation against BC-3.05.001 v1.4.3. Every seam examined independently from source:

1. **eval (AC-001/006/EC-007)**: InlineNode evaluation pipeline correct; AC-006 E-EVL-015 strict-fatal path present with load-bearing distinguishing test; EC-007 depth-limit E-LAY-005 3-field correct.
2. **Unified PPTX engine (Inv-9, 12 forms, highlight child-element, HI-1..HI-5)**: render_inline_nodes_to_runs produces correct OoxmlRun for all 12 forms; Invariant-9 grep-zero holds (no residual dual-generator divergence possible); a:highlight uses child-element form (not attribute); HI-1..HI-5 hyperlink invariants all satisfied with load-bearing reference-set tests.
3. **DOCX (PC-3 + unsafe-scheme)**: build_hyperlink_display_runs recurses for display text formatting; unsafe-scheme hard-error ExportError::ValidationError per SEC-001/CWE-601 codified in BC-3.05.001 v1.4.3 EC-013.
4. **HTML (PC-4 + escaping)**: <span role="note"> for Footnote; <code class="math"> for Math (v1.0 behavior per BC v1.4.1); proper HTML escaping on all text nodes.
5. **PDF (PC-5 super/sub size+direction)**: adjusted_baseline_y direction test (super STRICTLY < baseline_y, sub STRICTLY > baseline_y); SUPER_SUB_SCALE/SUPER_RISE_FRACTION/SUB_DROP_FRACTION constants; load-bearing baseline-direction test present.
6. **Layout E-LAY-005 3-field**: InlineDepthExceeded {source_slide_index, depth, max} — all 3 fields present and correctly named per BC v1.4.2 OBS-P24-004 fix.
7. **Determinism**: normalize_inline_nodes deterministic; no wall-clock timing gates.
8. **Error recovery**: all 13 TemplateChunk variants in chunks_to_markup_source produce non-empty markup form; slide_title != stripped_text for all non-Plain variants.
9. **Security**: safe-URL-scheme guard present on PPTX/HTML paths; DOCX hard-errors on unsafe-scheme.
10. **E2E cross-format gate**: multi-exporter integration tests cover all 5 surfaces with per-exporter load-bearing assertions.
11. **BC anchor sweep**: zero BC_3_02_002 residuals in STORY-081 deliverable files (completeness grep); all anchors point to BC-3.05.001.
12. **Snapshot hygiene**: insta --unreferenced=reject clean; no stale BC_3_02_002 snapshots.
13. **Semantic anchoring**: all module-level traceability comments cite BC-3.05.001 per correct clause (AC-001→Inv-10; AC-003→PC-3; AC-004→PC-5; AC-005→PC-4; AC-006→Title Constraint/EC-011).

ALL seams: PASS. ZERO findings. Novelty 0.0.

## Findings

None (0 findings of any severity).

## Observations

None.

## Novelty

0.0. Every axis previously examined; no new gap identified.
