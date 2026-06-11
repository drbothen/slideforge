# STORY-094 — LOCAL Adversary Cascade Summary

**Story:** REND-001 + REND-003 fix (EPIC-08, BC-3.06.003 + BC-4.01.001, 8 pts)
**PR:** #85 squash-merged → develop `c72bd2f6` (85 merged PRs) 2026-06-11
**Cascade:** 20 passes, 24 findings closed, CONVERGED 3/3 strict-CLEAN (passes 18-19-20)
**Post-convergence:** SEC-001 LOW fixed in-scope + fmt fix; LESSON-9 re-reviews CLEAN/APPROVE

---

## Pass Log

### Pass 1 — 6 findings
- F-094-P1-001: `nvGrpSpPr` missing from master parts (slide/notesSlide/notesMaster/handoutMaster) — master sweep required
- F-094-P1-002: `(0,0)` fallback for unresolved placeholder bbox — fixed
- F-094-P1-003: body+bullets region split — separate body ph from bullet ph (finalized)
- F-094-P1-004: EC-004 unreachable variant — cleaned
- F-094-P1-005: `cNvPr` ids starting at 0 — bumped to ≥2 for shape ids
- F-094-P1-006: `notes_slide.rs` + `notes_master.rs` raw-string XML builders — ACKNOWLEDGED-DEFERRED (pre-existing ADR-001 deviation; no ooxmlsdk CT_NotesBody type; human/architect adjudication pending — see FU-NOTES-SLIDE-RAW-XML-ADR001)

### Pass 2 — 3 findings
- CRIT: Identical-bbox stacking detection — post-layout geometric validator relocated to enforce stacking invariant
- HIGH: Body + bullets overlap in two_col layout — fixed
- MED: E-LAY-008 `BulletsOnContentlessSlideType` authored (error-taxonomy v2.30, PO commit `b8e76999` on factory-artifacts); SourceMap-resolved spans, cross-slide Multiple accumulation, taxonomy-faithful exit codes

### Pass 3 — 1 finding (CRIT)
- CRIT: Fabricated span in E-LAY-008 — `field_spans` IR threading required; architect-assessed; Option (b) wholesale Stage-2b span fidelity approved; `Slide.field_spans` threaded through IR

### Pass 4 — VOIDED then re-run
- VOIDED: Reviewer's `Read` calls resolved relative paths against main repo while Greps hit the worktree → false "work missing + files mutating" halt report. Per PATH DISCIPLINE mandate, pass was voided.
- RE-RUN (Pass 4): HIGH `<byte:N>:0:0` span renders — SourceMap resolution + `CompileOptions.source_name` threading (real filename); cross-slide accumulation; taxonomy-faithful exit codes (strict 2 / warn-only 0); warn-only per-slide error-placeholder rendering

### Pass 5 — 1 finding
- MED: Stale comments at 13 sites — swept + updated

### Pass 6 — 1 finding
- MED: E-LAY-008 cross-slide accumulation not using taxonomy-canonical `LayoutError::Multiple` — fixed

### Pass 7 — 1 finding
- MED: JSON render sibling missing — fixed

### Pass 8 — CLEAN (strict)
- Streak: 1/3

### Pass 9 — 2 findings (streak reset to 0/3)
- HIGH: Exit code non-compliance under strict mode — exit code 2 enforced
- HIGH: warn-only mode not demoting error-placeholder to slide-level render

### Pass 10 — 1 finding (streak reset to 0/3)
- HIGH: Placeholder rendered blank in warn-only mode — per-slide re-threading required

### Pass 11 — CLEAN (strict)
- Streak: 1/3

### Pass 12 — 1 finding (streak reset to 0/3)
- LOW: Aggregate placeholder message not per-slide — fixed to per-slide messages

### Pass 13 — CLEAN (strict)
- Streak: 1/3

### Pass 14 — CLEAN (strict)
- Streak: 2/3

### Pass 15 — 1 finding (streak reset to 0/3)
- HIGH: Bullet width page-relative in `two_col` layout → region-relative (containment fix); width sliver detection needed

### Pass 16 — 1 finding (streak reset to 0/3)
- MED: Degenerate-WIDTH sliver detection missing in `validate_post_layout` — floor added

### Pass 17 — 1 finding (streak reset to 0/3)
- HIGH: Degenerate-HEIGHT floor detection absent (symmetric axis case) — added

### Pass 18 — CLEAN (strict)
- Streak: 1/3

### Pass 19 — CLEAN (strict)
- Streak: 2/3

### Pass 20 — CLEAN (strict)
- Streak: 3/3 — **CONVERGED**

---

## Post-Convergence

- **SEC-001 RESOLVED in-scope:** `sanitize_source_name` added (CWE-116); path-traversal + null-byte + overlong input sanitization. Security re-review CLEAN (0 findings).
- **fmt fix:** formatting pass after SEC-001 changes.
- **LESSON-9 re-reviews:** Security-reviewer CLEAN; pr-reviewer APPROVE (re-review confirmed F1/F2/F3 body nits fixed).

---

## Standing Adjudications (do NOT re-litigate in future passes)

- `notes_slide.rs` raw-string XML = F-094-P1-006 ACKNOWLEDGED-DEFERRED (human adjudication pending)
- 31-vs-35 slide-type count drift = wave-gate reconciliation item (STORY-087 origin)
- Byte-column span semantics = pre-existing (STORY-006)
- Exit-code catch-all conservative mapping accepted
- HTML-only e2e error-slide assertion bar consistent
- Body-arm dedup asymmetry unreachable by construction
- Clamp + post-layout-detection design accepted

---

## Delivery Facts

- Demo evidence: `.factory/demos/STORY-094-demo-evidence.md` (all 5 ACs PASS)
- Workspace at PR head: 4142 pass / 20 skip / 0 fail (`a5384c92`)
- CI green on `a5384c92`
- Worktree/branches deleted; `.worktrees/` EMPTY post-merge
- develop == origin/develop == `c72bd2f6`; 0 open PRs

---

## Closed Defect Registrations

| ID | Surface | Severity | Status |
|----|---------|----------|--------|
| REND-001 | layout.rs bbox finalization + two_col containment | CRIT | CLOSED |
| REND-003 | nvGrpSpPr first-child in slide/notesSlide/notesMaster/handoutMaster | CRIT | CLOSED |
| SEC-001 (layout) | sanitize_source_name CWE-116 | LOW | RESOLVED in-scope |
