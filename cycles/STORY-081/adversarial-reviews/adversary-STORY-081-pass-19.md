---
document_type: adversary-pass-report
story_id: STORY-081
pass: 19
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: d185badb
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

# Adversary Pass 19 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (reset from 1/3).**

Fresh re-derivation at HEAD d185badb, extra scrutiny on under-tested axes (eval edge cases, determinism, error recovery, security). Determinism (deterministic rId allocation via ordered Vec+HashSet dedup; OoxmlRun Hash+Eq+Clone), depth-bound (single layout enforcement + engine defense, 65 errors no panic), HTML XSS escaping (html_escape crate all positions + is_safe_link_scheme allowlist), unsafe-scheme rejection across exporters, error recovery (engine Result, Math/unsafe degrade + warn, no panic) — ALL re-derived CLEAN. One MED finding.

## Findings

### F-P19-MED-001 [MEDIUM] — full-export hyperlink tests + 2 production doc-comments assert the count-equality form BC-3.05.001 HI-1 explicitly retired/forbids

**Severity:** MEDIUM
**BC clause:** BC-3.05.001 HI-1 (lines 267-278)
**TD-VSDD-059 classification:** Paper-correctness (test passed only because single-leaf inputs never exercised the multi-leaf case at export boundary)

BC-3.05.001 HI-1 (267-278) retired `external_rel_count == hlinkclick_count` as FALSE (1 rel backs N clicks for multi-leaf display text) and FORBIDS it as a test assertion/invariant; EC-012 canonical case `[click **here** now](url)` → 1 rel / 3 clicks.

**Violations found:**
- `slide_serializer.rs:942-944` + `slide_serializer.rs:1026-1028` — doc-comments stated the `∀` count-equality form as invariant (false per HI-1)
- `inline_markup_tests.rs` (5 sites) — `assert_eq!(external_rel_count, hlinkclick_count)` count-equality assertions
- `notes_tests.rs` (4 sites) — `assert_eq!(external_rel_count, hlinkclick_count)` count-equality assertions

**Root cause (TD-VSDD-059):** Export tests exclusively used single-leaf display text inputs; the multi-leaf HI-1 case (`[click **here** now](url)` → 1 rel / 3 clicks same rId) was exercised only at engine level, never at the export boundary. Count-equality assertions were structurally green because inputs never triggered the 1-rel/N-clicks case. Would false-fail (expected 1≠received 3) if a multi-leaf export test were added.

**REMEDIATED** (commit 3b76a9ba):
- Extracted `assert_hyperlink_reference_set_invariant(slide_xml, rels_xml)` helper: every `hlinkClick` rId → registered External rel; every registered rel referenced ≥1×; set/containment semantics, not counts.
- Converted all 9 export assertion sites (5 in inline_markup_tests.rs, 4 in notes_tests.rs) from count-equality to reference-set helper.
- Added 2 NEW multi-leaf export tests:
  - `test_f_p16_m1_body_multi_leaf_link_display_text` — body `[click **here** now](url)`: 1 rel / 3 clicks same rId / middle span bold; reference-set helper passes; count-equality would false-fail (1≠3). Load-bearing.
  - Enhanced notes multi-leaf export test encoding the same reference-set form.
- Corrected 2 `∀` doc-comments in `slide_serializer.rs` to HI-1 wording (set/containment).
- Production hyperlink behavior unchanged (was already correct — only test assertions and doc-comments were wrong).

## Observations

### OBS-P19-1 — stale retired-anchor references in test names + module header (BC_3_02_002 / PC8)

Post re-anchor to BC-3.05.001 (human ruling 2026-06-09), 6 test names still cited `BC_3_02_002` and the module traceability table cited `PC8` (retired anchor).

**REMEDIATED** (commit 3b76a9ba):
- 6 test names: `BC_3_02_002` → `BC_3_05_001`
- Module traceability table: `PC8` → `BC-3.05.001 PC-1`
- Zero stale refs remain after sweep.

### OBS-P19-2 — PDF Link text-only within carried PC-5 scope

PDF Link text-only fallback remains within the BC-3.05.001 PC-5 / STORY-045 deferral scope. Non-finding; carried OBS.

## Trajectory

...→P17(2MED remediated)→P18(0, 1/3)→P19(1MED, remediated, reset 0/3)

## Next Step

Fix landed at commit 3b76a9ba (test-assertion + doc correctness; no production behavior change). Restart cascade — adversary Pass 20 fresh at HEAD 3b76a9ba.
