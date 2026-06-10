---
document_type: adversary-pass-report
story_id: STORY-081
pass: 17
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: e5b1e92e
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 2
findings_crit: 0
findings_high: 0
findings_med: 2
findings_low: 0
findings_obs: 2
findings_open: 0
findings_remediated: 2
---

# Adversary Pass 17 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (held).**

First pass after (1) ADR-024 PPTX generator UNIFICATION and (2) re-anchor BC-3.02.002 →
BC-3.05.001 v1.4.0. Axis-B verified the unification is REAL + correct: unified
`render_inline_nodes_to_runs` is the SOLE PPTX inline-run generator (Invariant 9 grep-zero
holds; dispatch_inline_nodes_to_ooxml gone); body (ooxml_run_to_ooxmlsdk) + notes
(serialize_ooxml_run) feed from the same Vec<OoxmlRun>; F-P16-M1 fix load-bearing.
**FU-PPTX-DUAL-RUN-GENERATOR confirmed RESOLVED.** Axis-A BC-3.05.001 compliance matrix:
PC-1/PC-2/PC-5, HI-1..HI-5, Inv 9/10, EC-011/EC-016, depth-bound all PASS. Two MEDIUM
findings in the non-PPTX legs ADR-024 did NOT touch but BC-3.05.001 PC-3/PC-4 now govern.

## Findings

### F-P17-001 [MEDIUM] — DOCX silently drops formatting INSIDE a link's display text (F-P16-M1 bug class, DOCX sibling)

- document_body.rs Link arm built the run from collect_plain_text(text) →
  `Link{text:[Bold]}` (`[**here**](url)`) dropped the bold. HTML + PPTX preserve it;
  DOCX flattened (lone divergent surface). Violates BC-3.05.001 PC-3 + Description
  (all 5 surfaces) + no-silent-drop. S-7.01(b) sibling-layer: ADR-024 unified PPTX
  only; DOCX sibling unswept.
- REMEDIATED (commit d185badb): DOCX Link arm now recurses display-text children via
  build_hyperlink_display_runs → structured WR runs (with `<w:rPr>`) wrapped in
  `<w:hyperlink>` (mirrors apply_run_property WHyperlink branch); clickability preserved.
  Red-Gate tests test_f_p17_001_bold_inside_link_display_text_preserved_in_docx +
  italic variant + regression guard for Bold([Link]).

### F-P17-002 [MEDIUM] — HTML Footnote + Math diverge from BC-3.05.001 PC-4

- Footnote → `<small>` (PC-4 says `<span role="note">`); Math → `<code class="math">`
  (PC-4 v1.4.0 said MathML `<math>` unconditionally).
- REMEDIATED two ways:
  - Footnote (impl fix, commit d185badb): render.rs now emits `<span role="note">`
    matching PC-4. Red-Gate tests test_f_p17_002a_footnote_renders_as_span_role_note_not_small
    + inner-formatting-preserved.
  - Math (spec reconciliation, BC-3.05.001 v1.4.1): PC-4 Math clause corrected to the
    ACTUAL v1.0 behavior `<code class="math">{escaped LaTeX}</code>` with full MathML
    `<math>` explicitly DEFERRED to STORY-045 (codified, not silent). EC-010 + escaping
    test vector updated. All 12 PC-4 forms re-verified to match render.rs.

## Observations

### OBS-P17-A [process-gap] — stale dispatch-site comments + dead exemption branch in the grep-zero guard (notes_tests.rs)

Test remained load-bearing (Invariant 9 grep-zero real). REMEDIATED (commit d185badb):
removed dead dispatch_site_marker branch; updated doc comments to reference
render_inline_nodes_to_runs/serialize_ooxml_run/ooxml_run_to_ooxmlsdk; guard verified
still catches a planted raw `<a:r>` code line.

### OBS-P17-B — EC-011 strict-mode fatal-promotion of E-EVL-015 not traced end-to-end (build-orchestration BC-1.15.003 concern, outside STORY-081 eval scope)

Eval-side obligations PASS. Non-finding.

## Trajectory

...->P15(1HIGH,reset)->P16(2MED+1LOW→ADR-024 unification)->P17(2MED, both remediated, reset 0/3)

## Next Step

Both findings remediated at HEAD d185badb (DOCX + HTML Footnote impl; BC-3.05.001 v1.4.1
Math deferral). Restart cascade — adversary Pass 18 fresh at d185badb against
BC-3.05.001 v1.4.1.
