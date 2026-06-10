---
document_type: adversary-pass-report
story_id: STORY-081
pass: 15
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: d73bbc64
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
findings_obs: 2
findings_open: 0
findings_remediated: 1
---

# Adversary Pass 15 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (held).**

Fresh-context re-derivation at HEAD d73bbc64. Verified Pass-14 wrapper-descent fix sound for all normal shapes; verified the 5 changed F-085-P6-001 notes tests are non-vacuous (real rel+click, would fail if link dropped). Surfaced a new HIGH on the exotic link-in-link-display-text axis.

## Findings

### F-P15-HIGH-001 [HIGH] — body path emits an ORPHAN External rel for a Link nested in another Link's display text (notes path correct; body/notes asymmetry)
- Shape `Link{ url:U1, text:[Link{url:U2}] }` (and `Link{ url:U1, text:[Bold([Link{U2}])] }`), parser-reachable from `[[inner](https://inner.test)](https://outer.test)` (template.rs:1102-1117).
- Body collector registers only U1 (1 rel, no recurse into Link display text — F-040-P3-001 guard). Body dispatcher inner Link arm (slide_serializer.rs:1149-1164) does `rid=lookup(U2)=None` and builds `RunContext{ hyperlink_rid: None, ..ctx.clone() }`, OVERWRITING inherited Some(rId_U1) → inner text renders plain, no hlinkClick → 1 rel / 0 clicks → ORPHAN rel. Violates external_rel_count == hlinkclick_count. Notes path avoids it (flattens display text); only body is wrong — a consequence of the dual-generator design (OBS-P15-001).
- REMEDIATED (commit 5048a987): body Link arm now `let inherited_rid = rid.or_else(|| ctx.hyperlink_rid.clone()); RunContext{ hyperlink_rid: inherited_rid, .. }` — registered rid still wins; inherits outer rid only when inner lookup None. 1 rel / 1 click for the degenerate shape. NO flatten (Link([Bold([Plain])]) formatting preserved). Red-Gate tests: body link-in-link (1/1), body link-in-bold-link (1/1) — both failed pre-fix (1/0); notes guard test (1/1, passes via flatten). Added cross-path equivalence guard `test_obs_p15_001_body_notes_cross_path_rel_click_parity` (8-shape battery asserting body and notes produce identical (rel_count, hlinkclick_count) — makes future dual-generator divergence visible). Sibling check: exactly 2 hyperlink paths (body + notes), no third.

## Observations
### OBS-P15-001 [process-gap] FU-PPTX-DUAL-RUN-GENERATOR — THIRD defect rooted in the two independent PPTX inline-OOXML generators (typed body inline_node_to_ooxml_runs_inner vs raw-string notes render_ooxml_accumulate): ADV-P11-HIGH-001 (highlight attr), and now F-P15-HIGH-001 (orphan-rel divergence — body recurses node-by-node, notes flattens). Cross-path equivalence guard added this pass mitigates detection; the architectural unification remains a dedicated follow-up STORY (not in STORY-081 scope). Non-blocking OBS.
### OBS-P15-002 PDF link clickability — text-only, within AC-004 scope, consistent (no rel concept, orphan class impossible). NOT a finding.

## Trajectory
...->P12(0,1/3)->P13(2MED,reset)->P14(1MED,reset)->P15(1HIGH,reset 0/3)

## Next Step
Remediated at 5048a987. Restart cascade — adversary Pass 16 fresh.
