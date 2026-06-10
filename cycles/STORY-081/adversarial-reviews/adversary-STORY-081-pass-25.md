---
document_type: adversary-pass-report
story_id: STORY-081
pass: 25
date: 2026-06-10
branch: feature/STORY-081
branch_head_at_review: 2e13fb6e
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 3
findings_crit: 0
findings_high: 1
findings_med: 1
findings_low: 1
findings_obs: 5
findings_open: 0
findings_remediated: 3
---

# Adversary Pass 25 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (held).**

Fresh re-derivation: PRIMARY verified the BC v1.4.2 12-form×5-surface reconciliation against frozen code (2e13fb6e) — ALL per-surface corrected clauses MATCH the code (Footnote/Code/Strike/Highlight/Link/Math/Xref deferrals accurate; InlineDepthExceeded 3-field+E-LAY-005 accurate). But end-to-end tracing of the AC-006 title-constraint diagnostic (BC→eval→strict gate) surfaced 3 NEW findings the per-surface focus missed.

## Findings

### F-P25-HIGH-001 [HIGH] — AC-006 strict-mode fatal-promotion NOT implemented (build did not fail on title markup)

- BC EC-011 + Title Constraint + AC-006 require strict (default) build to FAIL on inline markup in a title (exit non-zero, no output). Code emitted `EvalError::InlineMarkupInTitle` at `ParseSeverity::Warning` (for_eval.rs:351-360); strict gate (lib.rs:908-918) fires only on Error+Fatal → a bold title built SUCCESSFULLY with a non-blocking warning. AC-006 strict clause had NO load-bearing test (eval unit test asserted the OPPOSITE — non-fatal). Spec-vs-code gap; spec wins (precedence rule 7).
- REMEDIATED (code commit b97386a0): InlineMarkupInTitle now emitted at `ParseSeverity::Error` → strict default build fails (non-zero, no output); `--warn-only` (strict:false) skips the gate → succeeds with plain title + title_inlines shadow. Red-Gate CLI tests `test_fp25_high_001_strict_mode_title_markup_fails_build` (strict fails) + `test_fp25_high_001_warn_only_title_markup_succeeds`. Eval unit test updated to assert Error severity. Blast radius 0 (existing e2e use strict:false).

### F-P25-MED-001 [MEDIUM] — contract-type drift: BC named LayoutWarning::InlineMarkupInTitle but code emits EvalError::InlineMarkupInTitle (E-EVL-015); LayoutWarning variant was DEAD code

- BC Title Constraint/EC-011/test-vectors named `LayoutWarning::InlineMarkupInTitle` (2-field, layout stage); the variant existed (shape_types.rs:285) but was constructed NOWHERE. Live diagnostic is `EvalError::InlineMarkupInTitle` (E-EVL-015, 3-field span, eval stage). Test comments mislabeled.
- REMEDIATED: code (b97386a0) removed the dead LayoutWarning variant + fixed test comments; BC (v1.4.3) corrected all occurrences to EvalError::InlineMarkupInTitle / E-EVL-015 / Error severity.

### F-P25-LOW-001 [LOW] — cross-surface unsafe-scheme inconsistency: DOCX hard-errors (ExportError::ValidationError, SEC-001/CWE-601) while PPTX/HTML silently degrade; BC PC-3 was silent

- REMEDIATED (BC v1.4.3): PC-3 codifies the DOCX unsafe-scheme fatal policy; EC-013 expanded to a 3-surface inconsistency table (PPTX/HTML degrade vs DOCX fail → all-formats build fails on unsafe-scheme link, driven by DOCX). FU-LINK-SCHEME-CONSISTENCY registered (human/architect adjudication — should surfaces be aligned? NOT resolved in-burst).

## Observations

### OBS-P25-001 — v1.4.2 per-surface reconciliation confirmed accurate

BC v1.4.2 12-form×5-surface corrected postconditions all MATCH frozen code at 2e13fb6e. Footnote/Code/Strike/Highlight/Link/Math/Xref deferral cites accurate. InlineDepthExceeded 3-field+E-LAY-005 accurate. Carried as confirmed.

### OBS-P25-002 — Inv-9 unified engine confirmed

ADR-024 unified engine (`render_inline_nodes_to_runs`) confirmed operational in both body and notes paths at 2e13fb6e. No dual-generator divergence. Carried.

### OBS-P25-003 — title_inlines parser-gated confirmed

title_inlines shadow-field mechanism confirmed parser-gated at depth 64. No live vuln. Carried.

### OBS-P25-004 [process-gap] — dead diagnostic variant co-existing with live variant: BC-accuracy audit gap

Dead `LayoutWarning::InlineMarkupInTitle` co-existed with live `EvalError::InlineMarkupInTitle` undetected through 24 passes. Root cause: BC accuracy audit (FU-BC-ACCURACY-AUDIT, source F-P17-002+F-P24-MED-001) checked render postconditions but NOT diagnostic-type liveness. A "BC names a type → grep its construction sites (not just doc comments)" check must be added to the BC-accuracy audit protocol. Extends FU-BC-ACCURACY-AUDIT scope.

### OBS-P25-005 — FU-BC-ACCURACY-AUDIT scope extension required

FU-BC-ACCURACY-AUDIT scope must be extended to include: (1) diagnostic-type naming — grep every BC-named LayoutWarning/EvalError/LayoutError type against ALL callsites confirming it is constructed (not merely declared); (2) title-constraint section audit — verify struct field counts, error codes, and severity labels against the live error.rs definition; not just render postconditions. Source: OBS-P25-004.

## Trajectory

...→P23(0→1/3 strict-CLEAN)→P24(1MED BC-content F-P24-MED-001 remediated BC v1.4.2; streak reset 0/3)→P25(1HIGH+1MED+1LOW all remediated; code b97386a0 + BC v1.4.3; streak reset 0/3)

## Next Step

All 3 findings remediated (code b97386a0: strict-fatal title + dead-variant removal; BC v1.4.3: type-name + DOCX scheme policy). Restart cascade — adversary Pass 26 fresh at code HEAD b97386a0 against BC v1.4.3.
