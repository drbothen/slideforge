---
document_type: adversary-pass-report
story_id: STORY-081
pass: 14
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: eec4be32
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

# Adversary Pass 14 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (held).**

Fresh-context re-derivation at HEAD eec4be32. Verified Pass-13 fixes sound (DOCX apply_run_property Fn descends into WHyperlink inner runs preserving clickability; combined-form assertions load-bearing across PPTX/DOCX/HTML/PDF; e2e slides 3-4). MANDATED nested-link PPTX parity adjudication resolved to a real defect.

## ADJUDICATION RULING: cross-format parity DEFECT (not spec-sanctioned)

The PPTX body silently drops a NESTED link's URL (`Bold([Link])` from `**[click](url)**` → bold but non-clickable), while DOCX (post-F-P13-001) + HTML render it bold AND clickable. Grounds:

1. No spec sanction — EC-004 (story line 526) requires `<a:hlinkClick>` for `InlineNode::Link`, no top-level carve-out; EC-003 (line 525) "all exporters handle nested runs correctly"; DIR-077-002 §3 treats `Bold([Link])` as first-class.
2. F-085-P6-001 "top-level-only" was an orphan-rel expedient (notes_slide.rs:322-366, mirrored into body), never parity-ratified (STATE.md:133 "Pending adversary ruling").
3. Orphan-rel invariant does NOT force the drop — `make_run` (slide_serializer.rs:1192-1268) carries both `b="1"` AND `a_hlink_click` on one `<a:rPr>`; collector could descend into wrappers keeping `external_rel_count == hlinkclick_count`. CLAUDE.md Rule-5 cheap-path violation.
4. Silent drop, no warn — same function warns on dropped Math (1159-1163) citing no-silent-drop. Internal inconsistency.

Also corrected Pass-13's mistaken claim "PPTX-body preserves bold-on-link" (true for bold, FALSE for the link).

## Findings

### ADV-P14-MED-001 [MEDIUM] — PPTX silently drops a wrapped link's URL (cross-format parity break, no diagnostic)

**Location:** `slide_serializer.rs` body collector (983-993), `for_body` (894-913), inner Link arm (1079-1100); `story_081_inline_markup_e2e.rs` (311-368).

**Defect:** Body collector `no-ops` on wrappers → URL unregistered; `for_body` sets `hyperlink_rid` only for top-level `Link`; inner `Link` arm renders plain when `ctx.hyperlink_rid` is `None`, no diagnostic emitted. DOCX + HTML render clickable; PPTX does not. e2e test codified the omission.

**REMEDIATED** (commit d73bbc64): body + notes collectors now descend THROUGH formatting wrappers (`Bold/Italic/Strike/Super/Sub/Highlight/Footnote`) to register safe-scheme non-empty nested-link URLs, never recursing into a `Link`'s own display text (orphan-rel guard preserved); dispatcher threads `hyperlink_rid` through wrapper arms so inner `Link` arm emits `<a:hlinkClick>` on a run also carrying inherited `bold`/`italic`. `external_rel_count == hlinkclick_count` maintained on both paths.

**Red-Gate tests added:**
- body bold-link: `b="1"` + `hlinkClick` + rel registered + count==1
- body bold-italic-link: `b="1"` + `i="1"` + `hlinkClick`
- notes bold-link: same invariants on notes path
- body nested unsafe-scheme: no rel/click registered (orphan-rel guard preserved)
- body nested empty-text: no orphan rel

**e2e updated:** slide-3 now asserts `hlinkClick` present; F-085-P6-001 top-level-only carve-out docstring removed.

## Observations

### OBS-P13-001 [carried][process-gap] FU-PPTX-DUAL-RUN-GENERATOR

PPTX dual run-generators (make_run typed ooxmlsdk vs DefaultInlineFormat raw-string). Non-blocking. Already registered as follow-up story candidate.

### OBS-P13-002 [carried]

PDF link clickability — PDF drops link URLs for ALL links (top-level AND nested) CONSISTENTLY; link clickability is NOT an AC-004 requirement (Phase-5 task line 389 mentions it but no AC asserts it). Spec-compliant, consistent — NOT a finding. Separate deferred-feature question (PDF link annotations) for future scope, not STORY-081.

## Trajectory

...->P11(1HIGH+1MED+1LOW,0/3)->P12(0,1/3)->P13(2MED,reset)->P14(1MED,reset 0/3)

## Next Step

Remediated at d73bbc64. Restart cascade — adversary Pass 15 fresh (HEAD d73bbc64). Need 3 consecutive strict-CLEAN to converge 3/3.
