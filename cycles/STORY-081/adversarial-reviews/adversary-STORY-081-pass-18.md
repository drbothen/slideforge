---
document_type: adversary-pass-report
story_id: STORY-081
pass: 18
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: d185badb
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
findings_obs: 4
findings_open: 0
findings_remediated: 0
---

# Adversary Pass 18 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): yes / CLEAN (PR-merge): yes / Streak after this pass: 1/3.**

Fresh re-derivation at HEAD d185badb against BC-3.05.001 v1.4.1. Verified Pass-17 fixes (DOCX build_hyperlink_display_runs nested-link formatting; HTML Footnote `<span role="note">`; grep-zero guard cleanup still load-bearing). Full BC-3.05.001 compliance matrix PASS (PC-1..PC-5, Title Constraint/EC-011, HI-1..HI-5, Inv 9/10, EC-012/013/014/016, depth-bound 64). Cross-surface parity (formatting-inside-link) now consistent PPTX body/notes/DOCX/HTML; PDF text-only per PC-5. Invariant 9 holds (single PPTX generator). FU-PPTX-DUAL-RUN-GENERATOR confirmed resolved. 4 carried OBS (PDF PC-5 boundary; Inv-9 resolved; FU-STORY-045 HTML MathML; DOCX per-surface empty/unsafe link semantics) — all non-resetting. ZERO findings.

## Compliance Matrix (BC-3.05.001 v1.4.1)

| Clause | Status |
|--------|--------|
| PC-1 PPTX body | PASS |
| PC-2 PPTX notes | PASS |
| PC-3 DOCX | PASS |
| PC-4 HTML | PASS |
| PC-5 PDF text-only | PASS |
| EC-011 Title Constraint | PASS |
| HI-1 reference-set (not count-equality) | PASS |
| HI-2 safe-scheme guard | PASS |
| HI-3 empty-display-text guard | PASS |
| HI-4 rId namespace isolation | PASS |
| HI-5 nested/wrapped link behavior | PASS |
| Invariant 9 (single PPTX generator) | PASS |
| Invariant 10 (depth-bound 64) | PASS |
| EC-012/013/014/016 | PASS |

## Observations (non-resetting)

- **OBS-P18-1** — PDF PC-5 boundary: text-only fallback within STORY-045 deferral scope; carried from prior passes.
- **OBS-P18-2** — Invariant 9 (FU-PPTX-DUAL-RUN-GENERATOR) resolved by ADR-024; confirmed in this pass.
- **OBS-P18-3** — FU-STORY-045-HTML-MATHML: HTML `<code class="math">` codified as v1.0 per BC-3.05.001 v1.4.1 PC-4; full MathML deferred to STORY-045. Non-finding in STORY-081 scope.
- **OBS-P18-4** — DOCX per-surface empty/unsafe link semantics — consistent with HI-2/HI-3 guards; no deviation found.

## Trajectory

...→P16(2MED→ADR-024)→P17(2MED remediated)→P18(0, 1/3)

## Next Step

Pass 19 sequential at unchanged HEAD d185badb.
