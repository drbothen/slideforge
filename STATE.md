---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-12T00:00:00
demo_review: "DEMO-REVIEW-2026-06-11 — 10 product defects (3 CRIT/4 HIGH/3 MED). Rendering-fix wave READY (STORY-094..102 created+indexed; BCs authored; delivery begins next session). See .factory/reviews/demo-deep-review-2026-06-11.md."
state_version: "1.8"
phase_1_approved: 2026-05-25
phase_2_approved: 2026-05-25
phase_1_convergence: "17 passes, 69 findings, 3/3 clean (passes 15-16-17)"
phase_2_convergence: "22 passes, 96+ findings, 3/3 clean (passes 20-21-22)"
prd_bcs: 120
prd_hs: 15
prd_vps: 15
prd_supplements: 4
spikes_resolved: 7
spikes_total: 7
total_stories: 102
total_points: 642
total_waves: 7
total_epics: 21
dtu_required: false
dtu_assessment: 2026-05-24
dtu_clones_built: n/a
dtu_services: []
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed"
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_gate: "PASSED 2026-06-07 — Gate 1 PASS; Gate 2 SKIP (no DTU); Gate 3 PASS (all 4 original findings closed; NEW-INT-001 image-alt RESOLVED PR #64); Gate 5 PASS (mean 1.00, min_critical 1.00; trajectory 0.56->0.86->1.00). BLK-002 CLOSED. develop 02d484cf (64 merged PRs)."
wave_4_merged: 23
wave_5_dep_prep: "MERGED PR #69 (3e3a978f) — [workspace.dependencies] centralized + ADR-022 major-version migrations: toml 1.1.2, sha2 0.11.0, criterion 0.8.2, notify 8.2.0, indexmap 2.14. INERT Wave-5 catalog entries added. Security CLEAN; CI green."
wave_5_status: "19 of 34 Wave-5 stories MERGED. 15 remain (5 rendering-fix [STORY-099..102+097, P0] + 10 feature [P1/P2]). STORY-096 MERGED PR #88 2a873a5d. NEXT: STORY-099."
develop_sha: "2a873a5d"
develop_pr_count: 88
open_prs: 0
error_taxonomy_version: "v2.33"
workspace_tests: "~4214+ pass / 20 skip / 0 fail (develop 2a873a5d)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge (public) | **Default branch:** `main` | **Dev branch:** `develop`
**Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop` must equal `2a873a5d65d22a2c1b59e3c2ff8dd440a76066f8` (88 merged PRs, 0 open PRs). If the short SHA `2a873a5d` does not match, STOP — do not create worktrees or dispatch agents.

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04; ongoing pushes authorized).

**Current position:** Phase 3, **Wave 5 IN PROGRESS**. 19 of 34 done (102 stories / 642 pts). 15 remain (5 rendering-fix P0 + 10 feature). **STORY-096 MERGED PR #88 → `2a873a5d` (REND-007 CLOSED). NEXT: STORY-099.**

**STANDING MERGE AUTH:** Orchestrator MAY squash-merge any PR that is CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking human.

---

### MANDATORY DISPATCH PREAMBLES (copy verbatim into every agent dispatch)

**PATH DISCIPLINE** (all reviewer / adversary / architect dispatches — include this text):
> Your file tools resolve RELATIVE paths against the MAIN repo root, not the worktree. EVERY Read/Grep/Glob MUST use ABSOLUTE paths. For worktree agents: use paths under `/Users/jmagady/Dev/slideforge/.worktrees/STORY-NNN/`. Sanity-check the worktree `git log` SHAs FIRST before reading any file; if Grep and Read disagree on content, you used a relative path. (STORY-094 adversary pass 4 was VOIDED for this; an architect dispatch read stale main-repo state for the same reason.)

**IMPLEMENTER GUARD** (all implementer dispatches — include this text):
> You MUST NOT create pull requests or push to develop. You MUST NOT run `gh pr create` or `git push origin develop`. Your scope ends at: all tests green, exit gate clean, feature branch pushed to origin/feature/STORY-NNN. The pr-manager and orchestrator own the PR lifecycle. (STORY-094's implementer opened PR #85 prematurely — orchestrator had to draft it.)

**ADVERSARY REPORT FORMAT** (all adversary dispatches — include this text):
> End your report with: (1) findings table with columns [ID | Severity | File:Line | Description | Fix]; (2) "CLEAN (strict): yes/no" — yes ONLY if zero findings of ANY severity; (3) "CLEAN (PR-merge): yes/no" — yes if zero CRIT/HIGH/MED findings; (4) Standing adjudications carried forward (list any previously settled items that are NOT re-litigated). First pass: no standing adjudications. Each subsequent pass: re-include the list from the prior pass.

**EXIT GATE** (all implementer dispatches — include this text):
> Before declaring work done, run ALL of the following in the worktree and confirm each exits 0:
> ```bash
> cargo fmt --all -- --check
> cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items
> cargo nextest run --workspace --no-fail-fast
> cargo test --workspace --all-features
> RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
> ```
> Verify fmt-clean ON the commit (run fmt --check after staging, not just in the working tree).

---

### NEW ORDER (human re-sequenced 2026-06-10): CI STABILIZATION FIRST

**Rationale:** Confirmed cache thrash (9.77 GB / 23 caches / arm64 LRU-evicted) is the root cause of STORY-081 bench flakiness. Fix CI first so STORY-081 and all future PRs merge cleanly.

**Execute in this order:**
1. ~~STORY-091~~ **DONE** (PR #82 merged 2026-06-11, develop `f3502c50`) — tiered CI live, fast tier ~6m02s
2. ~~STORY-092~~ **DONE** (PR #83 merged 2026-06-11, develop `74d56179`) — cache/disk fix live; AC-005/006 measurements open
3. ~~STORY-093~~ **DONE** (PR #84 merged 2026-06-11, develop `1ea6409d`) — mold arm64 + profile.ci live; SEC-001 RESOLVED; CI INITIATIVE COMPLETE
4. ~~STORY-081~~ **DONE** (PR #80 merged 2026-06-11, develop `433b3c01`) — slide-level inline markup MERGED; workstream B CLOSED
5. RENDERING-FIX WAVE (REND-001..010) — **IN PROGRESS (2/9)**
6. Remaining Wave-5 feature stories

---

### Workstream A — CI Stabilization (STORY-091/092/093 DONE — CI INITIATIVE COMPLETE)

**Step A3 — STORY-093 MERGED PR #84 2026-06-11 (DONE).**
**Step A2 — STORY-092 MERGED PR #83 2026-06-11 (DONE).** FU-SEMGREP-HASH-PIN open.
**Step A1 — STORY-091 MERGED PR #82 2026-06-11 (DONE).** **HUMAN ACTION REQUIRED: enable merge queue UI toggle.**
Full detail archived to wave-5-merges-archive.md.

**Open follow-ups:**
- **FU-MOLD-CVE-2026-3994** [confirm when full CVE advisory publishes]: verify mold 2.41.0 carries fix before any release from main.
- **FU-093-BASELINE-REMOVAL** [post-merge]: after first full-tier workflow_dispatch captures after-mold timings — (a) remove temporary baseline steps from ci.yml; (b) substitute `PR #<this PR>` placeholder → `#84`; (c) update develop-run cold-build NOTE to "all legs"; (d) refresh force-relink rationale comment.

---

### Workstream B — STORY-081 — MERGED PR #80 2026-06-11 (DONE — WORKSTREAM B CLOSED)

PR #80 squash-merged → develop `433b3c01` (84 PRs). Slide-level inline markup (EPIC-18, BC-3.05.001, 13 pts). Full detail archived to cycles/STORY-081/.

**Non-blocking follow-ups (open vehicles):**
- FU-S1-FONTDB-COUNT-VISIBILITY, FU-S3-CAPTION-FIXTURE, FU-TD1-DEAD-FONT-COUNTER.

---

## DURABLE RESUME — SAME MACHINE OR FRESH CLONE

All branches are on origin (durable, machine-independent):

- `origin/factory-artifacts` — all `.factory/` state; ADR-023/024; VP-054 v1.2.1; STORY-094+095+098+096 cascade summaries + lessons; demo evidence through STORY-096. (Run `git -C .factory log -1` for current HEAD.)
- develop `2a873a5d` (88 merged PRs, 0 open PRs). **0 active worktrees.**

**Same-machine resume:**
1. Run `vsdd-factory:factory-worktree-health`
2. Read STATE.md — STORY-096 MERGED. NEXT: create STORY-099 worktree + kick off delivery.
3. CI INITIATIVE COMPLETE. STORY-081 MERGED. STORY-094 + STORY-095 + STORY-098 + STORY-096 MERGED. **Human action required: enable merge queue UI toggle.**
4. Delivery order: STORY-099 → 100 → 101 → 097 → 102 → remaining Wave-5 feature stories.

**Fresh-clone (different machine) resume — exact commands:**
```
git clone https://github.com/drbothen/slideforge.git && cd slideforge
git fetch origin factory-artifacts
git worktree add .factory factory-artifacts
git rev-parse develop   # must equal origin/develop == 2a873a5d65d22a2c1b59e3c2ff8dd440a76066f8
git worktree add .worktrees/STORY-099 -b feature/STORY-099
```
Then read `.factory/STATE.md` → NEXT ACTIONS → kick off STORY-099.

---

## IN-FLIGHT WORKTREES — EXACT RESUME STATE

**0 active worktrees. 0 open PRs. STORY-096 MERGED — ready to kick off STORY-099.**

### STORY-096 — REND-007: PPTX slideMaster 16:9 geometry + progress_bar layout + lang — MERGED PR #88 → develop `2a873a5d` 2026-06-12

- PR #88 squash-merged → develop `2a873a5d` (88 merged PRs). REND-007 CLOSED.
- LOCAL adversary cascade: 9 passes, 11 findings + SEC-096-001 closed, CONVERGED 3/3 strict-CLEAN (passes 7-8-9).
- 2 PO adjudications: (a) no-lang default "en" ALL surfaces — BC-5.01.005 v1.3; (b) AC-001 rewritten — sldSz in presentation.xml ONLY, master asserts ABSENCE.
- Reviews: security CLEAN; pr-reviewer APPROVE (3 nits).
- Worktree/branches deleted; `.worktrees/` EMPTY. Cascade archived: `.factory/cycles/STORY-096/cascade-summary.md`.
- Workspace tests on develop `2a873a5d`: ~4214+ pass / 20 skip / 0 fail.

---

## WAVE 5 DELIVERY SUMMARY

**19 of 34 done (develop 2a873a5d, 88 PRs). CI INITIATIVE COMPLETE. Workstream B CLOSED. RENDERING-FIX WAVE IN PROGRESS (4/9).**

- **STORY-089 MERGED** PR #68 (c722c28b): field-value type validation.
- **STORY-046 MERGED** PR #70 (fa85d113): Static HTML exporter.
- **STORY-055 MERGED** PR #71 (cbebfd57): `slideforge build` CLI + miette diagnostics.
- **STORY-079 MERGED** PR #72 (2f6d5da4): slideforge-diagrams SVG DoS hardening.
- **STORY-080 MERGED** PR #73 (e08f2f80): deflake cross-platform tests.
- **STORY-074 MERGED** PR #74 (3f7f99ed): brand-aware em sizing.
- **STORY-047 MERGED** PR #75 (95f23df3): preview WS server + CSP nonce security hardening.
- **STORY-072 MERGED** PR #76 (2667987e): shape gradient fills end-to-end.
- **STORY-082 MERGED** PR #77 (c60cca36): PPTX slide-grouping sections.
- **STORY-088 MERGED** PR #78 (15838de1, ADMIN OVERRIDE): bullets list-literal DSL.
- **DEP-PREP MERGED** PR #69 (3e3a978f): [workspace.dependencies] centralized.
- **CI-FIX MERGED** PR #79: ci.yml test-matrix timeout + cache-on-failure.
- **CI-FIX MERGED** PR #81 (20a51e0c, ADMIN OVERRIDE — explicit per-request human auth): bench timeout.
- **STORY-091 MERGED** PR #82 (f3502c50): tiered CI triggers + merge queue. CONVERGED 3/3.
- **STORY-092 MERGED** PR #83 (74d56179): cache reliability + disk headroom. CONVERGED 3/3.
- **STORY-093 MERGED** PR #84 (1ea6409d): mold arm64 linker + profile.ci. CONVERGED 3/3. **CI INITIATIVE COMPLETE.**
- **STORY-081 MERGED** PR #80 (433b3c01): slide-level inline markup (EPIC-18, 13 pts). CONVERGED 3/3 (passes 28-29-30 + PR-level P31-P34). **Workstream B CLOSED.**
- **STORY-094 MERGED** PR #85 (c72bd2f6): REND-001+REND-003 (EPIC-08, 8 pts). CONVERGED 3/3 (passes 18-19-20). SEC-001 in-scope.
- **STORY-095 MERGED** PR #86 (6c27fcf3): REND-002+REND-008-pdf (EPIC-13, BC-4.03.001+BC-4.03.002, VP-054 NEW, 8 pts). CONVERGED 3/3 (passes 14-15-16). OBS-precedent at P11.
- **STORY-098 MERGED** PR #87 (14272e75): REND-005+REND-010a (EPIC-04, BC-3.03.002 v1.3+BC-1.11.002 v1.2+BC-4.01.001, 5 pts). CONVERGED 3/3 (passes 17-18-19). 2 PO adjudications: body-on-content REVERSED; missing data==empty-data for E-LAY-003.
- **STORY-096 MERGED** PR #88 (2a873a5d): REND-007 (EPIC-08, BC-4.01.001+BC-4.01.005+BC-5.01.005 v1.3, 5 pts). CONVERGED 3/3 (passes 7-8-9). 2 PO adjudications: (a) no-lang default "en" ALL surfaces; (b) AC-001 rewritten — sldSz in presentation.xml ONLY, master asserts ABSENCE (ECMA-376 §19.3.1.42).

**15 stories remain (5 rendering-fix P0 + 10 feature). NEXT: STORY-099.**

**HELD (after rendering-fix wave):**
- STORY-057/058/064 (slideforge-cli same-crate conflict — serialize after in-flight batch)
- STORY-056/048 (UNBLOCKED — ←047 MERGED)
- STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must resolve FIRST)

**OPEN FOLLOW-UPS:**
- **FU-096-43-DOC-NOTE** [nit]: `layout_xml.rs` — "4:3 same height" doc phrasing imprecise (pr-reviewer N-1, PR #88). Logged; non-blocking.
- **FU-096-DEAD-STATIC-CY** [nit]: `MASTER_PLACEHOLDER_DEFS` body entry idx==1 has a static `cy` value that is dead (not referenced in the geometry derivation path) — no compile-time signal it is unused (pr-reviewer N-2, PR #88). Candidate for follow-up cleanup story.
- **FU-096-PROCESS-GAP-OOXML-PLACEMENT** [process-gap]: OOXML-element-placement claims in specs/stories must be validated against ECMA-376 typed schema at story-authoring time. L-c in `.factory/cycles/STORY-096/lessons.md`. No current story vehicle — pending next spec-authoring cycle (story-writer/PO dispatch template update).
- **FU-098-SHARED-EMPTY-PREDICATE** [nit]: `chart_data_is_empty` duplicated in `collect_e_lay_003_chart_indices` — extract shared predicate. Source: PR #87 pr-reviewer nit 1.
- **FU-098-UNEVALUATED-DATA-DIAG** [SEC-003+nit]: `Expr`/`Interpolated` chart data silently skipped — add internal invariant diagnostic. Source: SEC-003 + pr-reviewer nit 2.
- **FU-098-ELAY003-HINT-SPECIFICITY** [nit]: hint lost binding-expression specificity. Source: PR #87 pr-reviewer nit 5.
- **FU-COLD-BUDGET-LOAD-FLAKE** [process-gap]: `cold_budget` test failed under full-parallel load 3+ times across STORY-095/098 bursts — needs load-robust gate or serial group. Source: STORY-098 cascade.
- **FU-095-NONPDF-COLORBAR-ALT** [a11y, follow-up candidate]: PPTX/DOCX/HTML exporters ignore new ColorBar alt field; ColorLabel-derived alt not threaded into PPTX shape descr / DOCX / HTML aria. Source: PR #86 pr-reviewer nit 5.
- **FU-095-WRAP-ENGINE-CONSOLIDATION** [maintenance]: two parallel wrap engines (text_layout plain vs exporter SpanWord) guarded by cross-engine agreement tests; future story could factor into one generic engine over a measurement trait. Source: pr-reviewer nit 2.
- **FU-095-ALT-LENGTH-CAP** [LOW security suggestion]: /Alt label uncapped; cap (e.g. 1024 chars) + diagnostic at layout extraction. Source: security review SEC-002 suggestion.
- **FU-095-WRAP-PRECONDITION-DOC** [LOW security suggestion]: document wrap_text bounded-input pre-condition / consider DSL-layer max-token cap. Source: security review SEC-001 suggestion.
- **FU-095-WHITESPACE-PATH-NOTE** [doc]: whole-string fast-path preserves whitespace verbatim while wrap path normalizes — add one-line doc note. Source: pr-reviewer nit 3.
- **FU-NOTES-SLIDE-RAW-XML-ADR001** [human/architect adjudication]: `notes_slide.rs` + `notes_master.rs` raw-string XML builders (pre-existing ADR-001 deviation). Human/architect decision required.
- **FU-INCLUDE-PATH-DIAG-SANITIZE** [LOW, pre-existing]: `include.rs:394` registers canonicalized include paths into SourceMap without SEC-001 sanitizer.
- **FU-SLIDE-TYPE-COUNT** [wave-gate reconciliation]: 31-vs-35 slide-type count drift.
- **FU-MOLD-CVE-2026-3994** [confirm when full advisory publishes].
- **FU-093-BASELINE-REMOVAL** [post-first-full-tier-run].
- **FU-SEMGREP-HASH-PIN** [LOW, pre-existing].
- **FU-MERGE-QUEUE-UI-TOGGLE** [HUMAN ACTION REQUIRED]: Enable merge queue on develop branch protection.
- **FU-DIAGNOSTIC-FIELD-PINNING**, **FU-BC-ACCURACY-AUDIT**, **FU-LINK-SCHEME-CONSISTENCY**, **FU-REANCHOR-COMPLETENESS-GREP**, **FU-VP-043-NOTES-PATH**, **FU-NFR027-CROSS-STORY-RECONCILE**, **FU-CI-BENCH-POSITIVE-COVERAGE**, **FU-088-BC10102-ANCHOR**, **FU-EXIT-GATE-DISTINGUISHING-OUTPUT** — see wave-5-merges-archive.md for full detail.

---

## CURRENT POSITION

Phase 3, **Wave 5 IN PROGRESS** (develop `2a873a5d`, 88 merged PRs). 19 of 34 done. 15 remain (5 rendering-fix P0 + 10 feature). **STORY-096 MERGED. CI INITIATIVE COMPLETE. STORY-081 MERGED. Workstream B CLOSED.**

- **NEXT: STORY-099** (HIGH: DOCX bullets/numbering/sectPr/lang, 8 pts). No active worktree yet. Create worktree + kick off delivery.
- Active worktrees: 0. Open PRs: 0.
- Workspace (develop `2a873a5d`): ~4214+ pass / 20 skip / 0 fail. CI STABILIZATION CONFIRMED.

---

## NEXT ACTIONS

**EXACT ORDERED TASK LIST:**

### Step 1 — RENDERING-FIX WAVE — IN PROGRESS (2/9 done)

**STORY-094 MERGED** PR #85 → develop `c72bd2f6`. REND-001 + REND-003 CLOSED.
**STORY-095 MERGED** PR #86 → develop `6c27fcf3`. REND-002 + REND-008-pdf CLOSED.
**STORY-098 MERGED** PR #87 → develop `14272e75`. REND-005 + REND-010a CLOSED.
**STORY-096 MERGED** PR #88 → develop `2a873a5d`. REND-007 CLOSED.

**NEXT: STORY-099** (HIGH: DOCX bullets/numbering/sectPr/lang, 8 pts). Spec: `.factory/stories/stories/STORY-099-rend-docx-bullets-secpr-lang.md` v1.1. Create worktree + kick off delivery (same runbook pattern as STORY-096).

| ID | Sev | Summary | Crates |
|----|-----|---------|--------|
| ~~REND-001~~ | CRIT | CLOSED (PR #85) | |
| ~~REND-002~~ | CRIT | CLOSED (PR #86) | |
| ~~REND-003~~ | CRIT | CLOSED (PR #85) | |
| REND-004 | HIGH | `takeaway` field never renders on-slide | `slideforge-eval`, `slideforge-layout`, all renderers |
| ~~REND-005~~ | HIGH | CLOSED (PR #87) — W-VAL-103 Route A strict-mode exit | |
| REND-006 | HIGH | DOCX: bullets empty `<w:p/>`; numbering.xml stub; no sectPr; lang dropped | `slideforge-docx` |
| ~~REND-007~~ | MED | CLOSED (PR #88) — master sldSz absence + 16:9 geometry + progress_bar layout + lang on rPr | |
| ~~REND-008-pdf~~ | (part) | CLOSED (PR #86) — bold font subset + progress_bar /Artifact | |
| REND-008-html | MED | HTML: chart SVG empty + EMU/px mismatch; image empty; bullets as `<p>` not `<ul>/<li>` | `slideforge-html` |
| REND-009 | HIGH | CLI: bare-path `Path::parent()` yields "" — brand I/O error | `slideforge-cli` |
| ~~REND-010a~~ | MED | CLOSED (PR #87) — ChartEmptyDataValidator registered (E-LAY-003) | |

**T5 checklist (4/9 done):**
- [x] STORY-094 — DONE (PR #85, c72bd2f6) — REND-001+REND-003 CLOSED
- [x] STORY-095 — DONE (PR #86, 6c27fcf3) — REND-002+REND-008-pdf CLOSED
- [x] STORY-098 — DONE (PR #87, 14272e75) — REND-005+REND-010a CLOSED
- [x] STORY-096 — DONE (PR #88, 2a873a5d) — REND-007 CLOSED
- [ ] STORY-099 (HIGH: DOCX bullets/numbering/sectPr/lang) — 8 pts
- [ ] STORY-100 (MED: HTML EMU/px + ul/li + image placeholder) — 5 pts
- [ ] STORY-101 (HIGH: CLI path normalization + chart SVG embed) — 8 pts
- [ ] STORY-097 (HIGH: takeaway bar, BC-3.07.001) — 8 pts
- [ ] STORY-102 (image binary embedding, BC-3.07.002; re-sequenceable to Wave 6 by human) — 8 pts

### Step 4 — Remaining Wave-5 feature stories

STORY-056/048 (UNBLOCKED), STORY-057/058/064 (cli serialized), STORY-060/061 (FU-SEC-001-GIT2-OPENSSL first).

---

## TASK LEDGER (durable mirror — reconstruct harness tasks from this)

| # | Task | Status | Notes |
|---|------|--------|-------|
| T1 | Deliver STORY-091 — CI tiered triggers + merge queue | DONE | PR #82 merged 2026-06-11 |
| T2 | Deliver STORY-092 — CI cache reliability + disk headroom | DONE | PR #83 merged 2026-06-11 |
| T3 | Deliver STORY-093 — CI arm64 build-time (mold + profile.ci) | DONE | PR #84 merged 2026-06-11 |
| T4 | Merge STORY-081 PR #80 | DONE | merged 2026-06-11, develop 433b3c01 |
| T5 | Rendering-fix wave — fix ALL REND-001..010 (human directive: none deferred) | IN PROGRESS — STORY-094 DONE (PR #85); STORY-095 DONE (PR #86, 6c27fcf3); STORY-098 DONE (PR #87, 14272e75); STORY-096 DONE (PR #88, 2a873a5d); 4/9. Next: STORY-099 | see checklist above |

OPEN HUMAN ACTIONS: (1) FU-MERGE-QUEUE-UI-TOGGLE — Settings → Branches → develop rule → "Require merge queue". (2) FU-NOTES-SLIDE-RAW-XML-ADR001 — adjudicate whether `notes_slide.rs` + `notes_master.rs` raw-string XML builders constitute an ADR-001 violation.

---

**Diagnostic commands:** `gh pr checks 86` / `gh run rerun --failed <run-id>`

**PER-STORY DELIVERY SEQUENCE (BC-5.39.001):**
adversary LOCAL 3-CLEAN (sequential) → demo-recorder per-AC → rebase onto develop → push → pr-manager 9-step (orchestrator dispatches security-reviewer + pr-reviewer per LESSON-5) → STANDING MERGE AUTH: CI-green + security CLEAN + pr-reviewer APPROVE → squash-merge → state-manager post-merge burst → worktree cleanup → LESSON-18 sync check.

**APPLY LESSON-21 to EVERY story exit gate:** nextest + `cargo test --workspace --all-features` (shared-process) + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` + cross-platform filesystem test discipline.

---

## STANDING AUTHORIZATIONS

**STANDING MERGE AUTH (granted 2026-06-02):** Orchestrator MAY squash-merge any PR that is CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking the human. Revocable by human.

**factory-artifacts PUSHED to remote (origin/factory-artifacts) — human-authorized 2026-06-04; ongoing pushes authorized.**

---

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1-q25 docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (116 BCs, 15 HS, 4 supplements) + arch (18 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 89 stories, 21 epics, 6 waves, 553 pts (baseline). Now 102 stories / 642 pts after CI initiative +3 + rendering-fix wave +9. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 5: **19/34 done**. 15 remain (5 rendering-fix P0 + 10 feature). CI INITIATIVE COMPLETE. STORY-081 MERGED. STORY-094/095/098/096 MERGED. **STORY-096 MERGED** (PR #88 → `2a873a5d`; REND-007 CLOSED). **NEXT: STORY-099.** | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. STORY-096 MERGED. 0 active worktrees. 0 open PRs. STANDING MERGE AUTH active. Merge-queue UI toggle PENDING HUMAN ACTION. NEXT: STORY-099.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-12 |
| **develop SHA** | `2a873a5d` (88 merged PRs; origin/develop; 0 open PRs) |
| **Active worktrees** | 0 — `.worktrees/` empty. |
| **STORY-096 state** | MERGED PR #88 → `2a873a5d`. REND-007 CLOSED. 9-pass LOCAL cascade CONVERGED 3/3 strict-CLEAN (passes 7-8-9). 2 PO adjudications: (a) no-lang default "en"; (b) AC-001 sldSz master absence. Demo 4/4 ACs PASS. Cascade + lessons archived. |
| **STORY-098 state** | MERGED PR #87 → `14272e75`. REND-005+REND-010a CLOSED. Cascade archived. |
| **CI initiative** | COMPLETE. Per-PR fast tier ~6-10 min confirmed. |
| **Workspace tests** | develop `2a873a5d`: ~4214+ pass / 20 skip / 0 fail. |
| **factory-artifacts** | Pushed to origin. Fresh sessions: clone + `git fetch origin factory-artifacts` + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | STORY-096 MERGED. NO active worktree. **Kick off STORY-099:** spec `.factory/stories/stories/STORY-099-rend-docx-bullets-secpr-lang.md` v1.1 (DOCX bullets/numbering/sectPr/lang, 8 pts, EPIC-08, REND-006). Run pre-flight: `git rev-parse develop` == `2a873a5d65d22a2c1b59e3c2ff8dd440a76066f8`; confirm 0 open PRs; create worktree `git worktree add .worktrees/STORY-099 -b feature/STORY-099`; dispatch test-writer (Red Gate). Delivery order: 099→100→101→097→102. |

---

## Standing Process Rules

| ID | Rule |
|----|------|
| LESSON-1 | Adversary dispatches MUST pass ABSOLUTE worktree path (`--cwd /Users/jmagady/Dev/slideforge/.worktrees/STORY-NNN`). |
| LESSON-2 | Canonical clippy: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`. Also run `RUSTDOCFLAGS="-D warnings" cargo doc`. |
| LESSON-3 | Re-run full pre-push gate (fmt + pedantic clippy + nextest) after EVERY commit to a feature branch before PR. |
| LESSON-5 | pr-manager CANNOT spawn sub-agents. Orchestrator dispatches security-reviewer + pr-reviewer independently. |
| LESSON-6 | RELAXED by STANDING MERGE AUTH: orchestrator may merge when CI-green + security-reviewer CLEAN + pr-reviewer APPROVE. |
| LESSON-7 | Per-story adversary convergence passes run SEQUENTIALLY. Never parallelize passes of one story. |
| LESSON-8 | Every pre-push gate AND implementer exit-gate MUST run FULL canonical clippy (LESSON-2) + `cargo fmt --all -- --check`. |
| LESSON-9 | When PR-level security/pr-reviewer findings are fixed AFTER per-story adversary convergence, RE-RUN both reviewers + wait for CI before merge. |
| LESSON-10 | For strict 3-CLEAN on compliance/extension stories: proactive doc-vs-code + sibling-site consistency audit BEFORE final convergence passes. |
| LESSON-11 | EXTENSION stories often have high GREEN-BY-DESIGN ratio. Stricter new tests can expose real bugs in already-merged code — treat as in-scope fixes. |
| LESSON-12 | Agents in feature worktrees MUST NEVER commit to `develop`. After every merge, verify `develop == origin/develop` before creating next worktree. |
| LESSON-13 | UPSTREAM-DATA VERIFICATION: before implementing a story consuming IR data from upstream, verify data is actually threaded through the real pipeline. |
| LESSON-14 | PRESENCE-VS-CONTENT TESTS: strengthen to assert CONTENT and VALIDITY, not just existence. |
| LESSON-15 | Demo-recorder example binaries MUST pass FULL canonical clippy including `-W clippy::missing_docs_in_private_items`. |
| LESSON-16 | PRE-PUSH GATE MUST MIRROR EXACT CI INVOCATIONS (full workspace pedantic clippy + rustdoc gate). |
| LESSON-17 | `#[should_panic]` is NEVER a placeholder for behavioral correctness tests. |
| LESSON-18 (WORKTREE-SYNC) | After `gh pr merge --squash`, run `git fetch && git merge --ff-only origin/develop` to sync working tree. |
| LESSON-19 (SIBLING-SWEEP) | When a fix-burst changes a canonical VALUE, TEXT, COUNT, or ANCHOR, sweep ALL sibling artifacts in ONE burst. |
| LESSON-20 (ADVERSARY-SPEC-PATHS) | Per-story LOCAL adversary dispatches MUST include ABSOLUTE `.factory/` spec paths (story file, traced BCs, traced ADRs). |
| LESSON-21 (LOCAL-GATE-MIRRORS-CI-MATRIX) | Exit gate MUST include: `cargo test --workspace --all-features` + `RUSTDOCFLAGS="-D warnings" cargo doc` + cross-platform filesystem tests. |
| LESSON-22 (ORCHESTRATOR-VERIFY-UPSTREAM-CLAIMS) | Before routing a deferral based on an implementer's claim, VERIFY the claim against actual upstream code. |
| LESSON-OPS-RATE-LIMIT | RATE-LIMITING ACTIVE: serialize adversary/review passes ONE AT A TIME. |
| LESSON-OPS-CWD | cwd DISCIPLINE: every worktree-scoped agent MUST prefix EVERY bash command with `cd <worktree> &&` and use absolute paths. |
| LESSON-OPS-CI-DISKSPACE | Snapshots CI disk failures are INFRA FLAKES — re-run on fresh runner. |
| LESSON-OPS-SPECFIX-SCOPE | Architect/product-owner/story-writer dispatches MUST be scoped to .factory/ ONLY. |

---

## Blocking Issues

| ID | Description | Status |
|----|-------------|--------|
| BLK-001 | Alt-text enforcement non-functional end-to-end. | RESOLVED — STORY-050 ADR-018 post-layout validation. |
| BLK-002 | Wave 4 gate (content threading + color-coded types + image-alt fix). | CLOSED 2026-06-07 — Gate 5 mean 1.00, min_critical 1.00. |

---

## Decisions Log

_Wave-4-gate and earlier archived to `.factory/cycles/wave-4-gate/decisions-archive.md`._
_STORY-081 full 34-pass cascade (passes 1-30 LOCAL + P31-P34 PR-level) archived to `.factory/cycles/STORY-081/`._
_Wave-5 per-story pass logs archived to `.factory/cycles/wave-5-merges-archive.md`._

| Date | ID | Decision |
|------|-----|---------|
| 2026-06-12 | STORY-096-MERGE | PR #88 squash-merged → develop `2a873a5d` (88 merged PRs). REND-007 CLOSED. EPIC-08, BC-4.01.001+BC-4.01.005+BC-5.01.005 v1.3, 5 pts. LOCAL adversary cascade: 9 passes, 11 findings + SEC-096-001 closed, CONVERGED 3/3 strict-CLEAN (passes 7-8-9). PO adjudications: (a) no-lang default "en" ALL surfaces — BC-5.01.005 v1.2→v1.3; story swept v1.0→v1.2; STORY-099 v1.1 + STORY-100 v1.1 swept (STORY-100 gained AC-006/EC-005 + T-008/T-009 + BC-5.01.005 frontmatter); (b) AC-001 rewritten — sldSz in presentation.xml ONLY, master asserts ABSENCE (ECMA-376 §19.3.1.42). Delivered: master placeholder geometry from brand.page_size + .max(0) clamps; schema-invalid master sldSz REMOVED; progress_bar named layout (CL-20 "SF Progress Bar" idx 30 → slideLayout31.xml, rels-verified); lang on every a:rPr + universality test; DEFAULT_DECK_LANG="en" cross-surface identity; SEC-096-001 fail-fast validate_lang_for_xml. Reviews: security CLEAN (re-verify 9c8e1e1e); pr-reviewer APPROVE (3 nits). Demo evidence: `.factory/demos/STORY-096-demo-evidence.md` (4/4 ACs PASS). Workspace tests: 4214 pass / 20 skip / 0 fail. Worktree/branches deleted. Cascade archived: `.factory/cycles/STORY-096/cascade-summary.md`. Lessons: `.factory/cycles/STORY-096/lessons.md` (L-a timeless prose, L-b traceability sweep, L-c [process-gap] OOXML-element-placement). Open follow-ups: FU-096-43-DOC-NOTE, FU-096-DEAD-STATIC-CY. NEXT: STORY-099. |
| 2026-06-11 | STORY-098-MERGE | PR #87 squash-merged → develop `14272e75` (87 merged PRs). REND-005 + REND-010a CLOSED. EPIC-04, BC-3.03.002 v1.3 + BC-1.11.002 v1.2 + BC-4.01.001, 5 pts. 26 files, +2989/−529. LOCAL adversary cascade: 19 passes, 24 findings closed, CONVERGED 3/3 strict-CLEAN (passes 17-18-19). PO adjudications: (1) BC-3.03.002 EC-007 REVERSED — `body` is VALID on `content` (known_fields() is the authority; ratified post-hoc); (2) missing `data:` == empty-evaluating `data:` for E-LAY-003. Delivered: CONTENT_DROP_KEYS W-VAL-103 context-sensitive severity (Route A); taxonomy v2.29→v2.33 incl. summary-row; body-on-content end-to-end; known_fields 31→34 + registry↔known_fields coherence test; ChartEmptyDataValidator (E-LAY-003); pre-layout warn-only placeholder + mixed-deck selectivity tests; e2e exit-code suite; charts dead-code module deleted. Spec: story v1.0→v1.5; BC-3.03.002 →v1.3; BC-1.11.002 →v1.2; error-taxonomy →v2.33; BC-INDEX title sync; STORY-INDEX de-versioned. Security CLEAN (3 LOW); pr-reviewer APPROVE (5 nits). CI green fast-tier. Demo evidence: `.factory/demos/STORY-098-demo-evidence.md` (5/5 ACs PASS). Worktree/branches deleted. Cascade archived: `.factory/cycles/STORY-098/cascade-summary.md`. Lessons archived: `.factory/cycles/STORY-098/lessons.md`. Open follow-ups: FU-098-SHARED-EMPTY-PREDICATE, FU-098-UNEVALUATED-DATA-DIAG, FU-098-ELAY003-HINT-SPECIFICITY, FU-COLD-BUDGET-LOAD-FLAKE. NEXT: STORY-096. |
| 2026-06-11 | STORY-095-MERGE | PR #86 squash-merged → develop `6c27fcf3` (86 merged PRs). REND-002 + REND-008-pdf CLOSED. EPIC-13, BC-4.03.001+BC-4.03.002, VP-054 (NEW v1.2.1), 8 pts. LOCAL adversary cascade: 16 passes, 25 findings closed, CONVERGED 3/3 strict-CLEAN (passes 14-15-16). OBS-strict-precedent enforced at P11 (streak reset; orchestrator ruling: OBS counts under BC-5.39.001). Delivered: pure word-wrap engine text_layout.rs (word-boundary + char-fallback + frag0 boundary space, VP-054 Kani-amenable); wrap wired into BOTH PDF draw paths (shared space_before_word helper; SpanWord.is_continuation; measure==draw by construction); frame-bottom clamp + tracing::warn; ColorBar /Figure+/Alt end-to-end from ColorLabel-derived label (layout.rs threading; decorative stays /Artifact); bold subset preserved across wrap; veraPDF fixtures extended (pdf-ua1-verapdf CI job PASSED on PR with new /Figure fixture); FontMetrics mock_char/space_width_pts. Security CLEAN (0 crit/0 important; 2 suggestions: FU-095-ALT-LENGTH-CAP, FU-095-WRAP-PRECONDITION-DOC logged). pr-reviewer APPROVE (5 non-blocking nits; nit-5 description inaccuracy fixed pre-merge). CI green fast tier ~6min. 4180/20/0 workspace tests on develop 6c27fcf3. Worktree/branches deleted; `.worktrees/` EMPTY. Cascade archived: `.factory/cycles/STORY-095/cascade-summary.md`. Lessons archived: `.factory/cycles/STORY-095/lessons.md`. Demo evidence: `.factory/demos/STORY-095-demo-evidence.md` (5/5 ACs PASS, commit ad983606). OPEN FOLLOW-UPS added: FU-095-NONPDF-COLORBAR-ALT, FU-095-WRAP-ENGINE-CONSOLIDATION, FU-095-ALT-LENGTH-CAP, FU-095-WRAP-PRECONDITION-DOC, FU-095-WHITESPACE-PATH-NOTE. NEXT: STORY-098. |
| 2026-06-11 | STORY-094-MERGE | PR #85 squash-merged → develop `c72bd2f6` (85 merged PRs). REND-001+REND-003 CLOSED. EPIC-08, BC-3.06.003+BC-4.01.001, 8 pts. LOCAL adversary cascade: 20 passes, 24 findings closed, CONVERGED 3/3 strict-CLEAN (passes 18-19-20). P4 VOIDED (reviewer path error) and re-run. Key deliverables: region-relative two_col bullet width, body+bullets region split, post-layout geometric validation, nvGrpSpPr first-child sweep, E-LAY-008 w/ SourceMap+exit-codes+warn-only, Slide.field_spans IR threading, SEC-001 RESOLVED in-scope (CWE-116). Security re-review CLEAN; pr-reviewer APPROVE. CI green `a5384c92`. 4142/20/0. Cascade archived `.factory/cycles/STORY-094/cascade-summary.md`. |
| 2026-06-11 | RENDERING-FIX-WAVE-READY | Wave prep COMPLETE. 9 fix stories created + indexed: STORY-094..102. Coverage: ALL REND-001..010 mapped; none deferred (human "fix ALL" directive honored). Totals: 102 stories / 642 pts; Wave 5 expanded to 34 stories. |
| 2026-06-11 | STORY-081-MERGE | PR #80 squash-merged → develop `433b3c01` (84 PRs). Slide-level inline markup (EPIC-18, BC-3.05.001, 13 pts). All reviews passed. Rebase zero-conflict; 4091 pass / 20 skip / 0 fail. Workstream B CLOSED. |
| 2026-06-11 | STORY-093-MERGE + CI-INITIATIVE-COMPLETE | PR #84 → `1ea6409d`. mold arm64 + profile.ci. CONVERGED 3/3 (passes 9-10-11). SEC-001 RESOLVED. **CI INITIATIVE COMPLETE.** |
| 2026-06-11 | STORY-092-MERGE | PR #83 → `74d56179`. Cache reliability + disk headroom. CONVERGED 3/3 (passes 1-2-3). |
| 2026-06-11 | STORY-091-MERGE | PR #82 → `f3502c50`. Tiered CI + merge queue. CONVERGED 3/3 (passes 8-9-10). Fast tier ~6m02s ACHIEVED. |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
