---
document_type: adversary-pass-report
story_id: STORY-081
pass: 16
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: 5048a987
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 3
findings_crit: 0
findings_high: 0
findings_med: 2
findings_low: 1
findings_obs: 2
findings_open: 0
findings_remediated: 3
---

# Adversary Pass 16 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (held).**

Fresh-context re-derivation at HEAD 5048a987. Verified the Pass-15 rid-inheritance fix
SOUND (both asymmetric safe/unsafe link-nesting corners secure: no unsafe-scheme smuggling,
no orphan, no dangling rid; cross-path test non-vacuous, would have caught F-P15-HIGH-001).
Surfaced 2 MED + 1 LOW on the link-with-formatted-display-text axis.

## Findings

### M1 [MEDIUM] — PPTX notes path SILENTLY DROPS formatting inside link display text (cross-format parity break)

`[click **here**](url)` (parser-reachable, template.rs:1103-1117 →
eval register_routing.rs:267-273 → `Link{text:[Plain, Bold([Plain])]})`). Notes path
(`default_formatter.rs` flatten via `extract_plain_text_depth_limited`) drops the bold;
body path + HTML preserve it. Silent drop (no warn). HUMAN RULED THIS A DEFECT
(preserve formatting). Also exposes body/notes count divergence (body 1 rel/2 clicks
vs notes 1 rel/1 click) the 8-shape parity battery never exercised (single-leaf only).

REMEDIATED via ADR-024 unification (see below): notes now preserves link-text formatting
through the unified engine.

### M2 [MEDIUM] — cross-path test asserts only body↔notes PARITY, never the absolute per-shape rel/click contract; a shared-mode regression passes silently

REMEDIATED: replaced by ADR-024 Step-7 single-engine structural invariant test
(reference-set INV-4 + resolver-call INV-5 contract).

### L1 [LOW] — notes link-formatting-loss intent question

RESOLVED by human ruling: it is a DEFECT; preserve formatting. No notes-plain carve-out.

## Observations

### OBS-1 [process-gap] OBS-P15-001 FU-PPTX-DUAL-RUN-GENERATOR — root cause of M1 (and ADV-P11-HIGH-001, F-P15-HIGH-001)

RESOLVED by ADR-024 (see below) — the two generators are now ONE engine; the FU is
closed by the unification, not deferred.

### OBS-2 PDF link clickability within AC-004 scope — NOT a finding.

## Resolution: ADR-024 unification (human-authorized root-cause fix)

Human (senior architect) ruled: UNIFY the two PPTX inline-run generators NOW in
STORY-081, and notes-link-formatting-loss is a DEFECT (preserve). Implemented across
HEADs c1401f33 (unification) + e5b1e92e (dead-code cleanup):

- New unified engine `render_inline_nodes_to_runs(nodes, hlink_resolver) -> Result<Vec<OoxmlRun>, InlineError>` +
  `OoxmlRun` neutral typed intermediate + `serialize_ooxml_run` (single `<a:rPr>`
  child-ordering site) in `slideforge-plugin-api/src/inline_formats/ooxml_runs.rs`.
- Body path consumes via `ooxml_run_to_ooxmlsdk` (typed ooxmlsdk `Run`); notes via
  `serialize_ooxml_run` (string). Both call the SAME engine — divergence impossible
  by construction.
- F-P16-M1 fixed (notes preserves link-text formatting); test
  `test_f_p16_m1_bold_in_link_display_text_formatting_preserved_notes`.
- Risk-4 anti-orphan: rels pre-pass derives from the SAME resolver-query set the
  engine uses (collector↔emitter symmetry by construction).
- Old body cluster (`inline_node_to_ooxml_runs` / `_inner` / `make_run` / `RunContext`,
  ~312 lines) DELETED; 3 tests retargeted to production path (Math→warn intent
  preserved); zero workspace refs to deleted symbols.
- Corrected hyperlink invariant: reference-set ("every `<a:hlinkClick>` references a
  registered rel AND every rel referenced ≥1×; no orphan, no dangling") — count-equality
  is false for multi-leaf link display text.
- Workspace tests 3820 pass / 20 skip / 0 fail; clippy + fmt clean; AC-005 grep-zero
  audit clean.

## Trajectory

...->P13(2MED,reset)->P14(1MED,reset)->P15(1HIGH,reset)->P16(2MED+1LOW,reset→ROOT-CAUSE FIX ADR-024)

## Next Step

ADR-024 unification implemented at e5b1e92e. NOTE: orchestrator surfaced a separate
BC-3.02.002 ANCHORING issue (STORY-081 slide-level inline markup is anchored to a
section-block BC whose PC8 excludes slide-level) — pending human adjudication before
cascade resumes. After anchor resolution, restart cascade — adversary Pass 17 fresh
on the unified engine.
