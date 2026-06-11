---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-11
demo_review: "DEMO-REVIEW-2026-06-11 — 10 product defects (3 CRIT/4 HIGH/3 MED). Rendering-fix wave READY (STORY-094..102 created+indexed; BCs authored; delivery begins next session). See .factory/reviews/demo-deep-review-2026-06-11.md."
state_version: "1.6"
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
wave_5_status: "15 of 34 Wave-5 stories MERGED. 19 remain (9 rendering-fix [STORY-094..102, P0] + 10 feature [P1/P2]). RENDERING-FIX WAVE READY — STORY-094..102 created+indexed 2026-06-11; BC-3.07.001/002/003 + BC-3.03.002 v1.2 + BC-1.15.001 v1.2 authored; error-taxonomy v2.29. Delivery begins next session (CRITs first: 094→095→098→096/099/100/101→097→102)."
develop_sha: "433b3c01"
develop_pr_count: 84
open_prs: 1
error_taxonomy_version: "v2.30"
workspace_tests: "4140 pass / 20 skip (worktree STORY-094 @ c8f96258)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge (public) | **Default branch:** `main` | **Dev branch:** `develop`
**Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop`. Canonical SHA: `433b3c01` (84 merged PRs, 1 open PR: #85 DRAFT — STORY-094).

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04; ongoing pushes authorized).

**Current position:** Phase 3, **Wave 5 IN PROGRESS**. 15 of 34 done (102 stories / 642 pts). 19 remain (9 rendering-fix P0 + 10 feature). **STORY-094 IN PROGRESS (LOCAL adversary cascade pass 17 done, streak 0/3, resume at pass 18). CI INITIATIVE COMPLETE. STORY-081 MERGED.** Workstream B CLOSED.

**STANDING MERGE AUTH:** Orchestrator MAY squash-merge any PR that is CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking human.

---

### NEW ORDER (human re-sequenced 2026-06-10): CI STABILIZATION FIRST

**Rationale:** Confirmed cache thrash (9.77 GB / 23 caches / arm64 LRU-evicted) is the root cause of STORY-081 bench flakiness. Fix CI first so STORY-081 and all future PRs merge cleanly.

**Execute in this order:**
1. ~~STORY-091~~ **DONE** (PR #82 merged 2026-06-11, develop `f3502c50`) — tiered CI live, fast tier ~6m02s
2. ~~STORY-092~~ **DONE** (PR #83 merged 2026-06-11, develop `74d56179`) — cache/disk fix live; AC-005/006 measurements open
3. ~~STORY-093~~ **DONE** (PR #84 merged 2026-06-11, develop `1ea6409d`) — mold arm64 + profile.ci live; SEC-001 RESOLVED; CI INITIATIVE COMPLETE
4. ~~STORY-081~~ **DONE** (PR #80 merged 2026-06-11, develop `433b3c01`) — slide-level inline markup MERGED; workstream B CLOSED
5. RENDERING-FIX WAVE (REND-001..010) — **NEXT**
6. Remaining Wave-5 feature stories

---

### Workstream A — CI Stabilization (STORY-091 DONE; STORY-092 DONE; STORY-093 DONE — CI INITIATIVE COMPLETE)

**STORY-081 MERGED** (PR #80 → develop `433b3c01`). See Workstream B section. Workspace 4091 pass / 20 skip / 0 fail.

**Step A0 — PR #81 MERGED 2026-06-11 (DONE):**
`ci/bench-timeout-cache-on-failure` → develop `20a51e0c`: bench timeout 20→60 min + `cache-on-failure: "true"`. Admin-merged with EXPLICIT per-request human authorization. Bench timeout now live on develop.

**Step A1 — STORY-092 MERGED PR #83 2026-06-11 (DONE):**
Cache reliability + disk headroom. develop `74d56179` (82 PRs). Spec final v1.3. LOCAL adversarial CONVERGED 3/3 strict-CLEAN (passes 1-2-3). ci-workflow-analyzer pre-review: 6 findings fixed (key discovery: old rust-cache pin `42dc69e1` was the TAG OBJECT for floating v2 — already running v2.9.1; bump is zero-behavioral-delta GC-fragility fix; security.yml codeql-action had same defect, re-pinned to peeled `03e4368a`). Security CLEAN (both new SHAs independently API-verified as genuine peeled release commits; zero tag-object pins remain in .github/). pr-reviewer APPROVE. Delivered: 19 rust-cache SHA bumps across 5 workflows; cache-on-failure at all 11 ci.yml sites; guarded disk cleanup in 6 jobs; playbook §9 cache budget. Remote branch deleted; worktree removed; only STORY-081 worktree remains. Demo evidence: `.factory/demos/STORY-092-demo-evidence.md`. **AC-005/006 measurements OPEN** (cache snapshot 10.1 GiB/24 caches post-merge; arm64 warm-cache verification pending develop run `27320906767` outcome + one additional run). **Follow-up logged:** FU-SEMGREP-HASH-PIN (LOW, pre-existing): `security.yml pip3 install semgrep==1.90.0` lacks `--require-hashes`; fold into future CI-hardening story (candidate: same vehicle as FU-CI-BENCH-POSITIVE-COVERAGE).

**Step A2 — STORY-091 MERGED PR #82 2026-06-11 (DONE):**
Tiered CI triggers + GitHub merge queue. develop `f3502c50` (81 PRs). Spec v1.4. LOCAL cascade: 10 passes, CONVERGED 3/3 strict-CLEAN (passes 8-9-10). Fast tier wall-clock: ~6m02s (TARGET ACHIEVED). Branch protection on `develop` CREATED. Merge-queue UI toggle: **PENDING HUMAN ACTION** (Settings → Branches → edit develop rule → check "Require merge queue"). STORY-091 worktree removed; remote + local branch deleted. Demo evidence: `.factory/demos/STORY-091-demo-evidence.md`.

**Step A3 — STORY-093 MERGED PR #84 2026-06-11 (DONE):**
mold linker arm64 + `[profile.ci]` `debug = "line-tables-only"`. Spec final v1.4. LOCAL adversarial CONVERGED 3/3 strict-CLEAN (passes 9-10-11 of 11). Key cascade findings: CRIT cargo-fingerprint linker-blindness → force-relink step; HIGH dead CARGO_TARGET_*_RUSTFLAGS (rustflags sources mutually exclusive, empirically proven) → ld-symlink activation via make-default:true; HIGH baseline contamination → baseline reordered before mold; fail-closed + positive-coverage hardening; AC-005 du capture; spec reconciled to 4-platform matrix. SEC-001 RESOLVED: explicit mold-version 2.41.0 pin (post-CVE-2026-3994 range) + sha256 binary verification; tarball digest independently confirmed via GitHub Releases API. pr-reviewer APPROVE. Demo: `.factory/demos/STORY-093-demo-evidence.md`. Worktree removed; remote+local branch deleted. **CI INITIATIVE COMPLETE.**

**Open follow-ups:**
- **FU-MOLD-CVE-2026-3994** [confirm when full CVE advisory publishes]: verify mold 2.41.0 carries fix before any release from main.
- **FU-093-BASELINE-REMOVAL** [post-merge]: after first full-tier workflow_dispatch captures after-mold timings — (a) remove temporary baseline steps from ci.yml; (b) substitute `PR #<this PR>` placeholder → `#84`; (c) update develop-run cold-build NOTE to "all legs"; (d) refresh force-relink rationale comment (pr-reviewer nits 1-2).

---

### Workstream B — STORY-081 — MERGED PR #80 2026-06-11 (DONE — WORKSTREAM B CLOSED)

PR #80 squash-merged → develop `433b3c01` (84 PRs; 64 files, +12,672/−880). Slide-level inline markup (EPIC-18, BC-3.05.001, 13 pts). All reviews passed pre-park at `5c665cd0` (LOCAL 3/3 passes 28-29-30; PR-level P31-P34 converged; security CLEAN; pr-reviewer APPROVE). Rebase `5c665cd0`→`78b4d692` onto develop `1ea6409d` zero-conflict/zero-new-code; post-rebase gate green (4091 pass / 20 skip / 0 fail). Merged under STANDING MERGE AUTH. Remote+local branch deleted; worktree removed (force-removal needed). `.worktrees/` now EMPTY.

**CI initiative first mold develop run `27323925653` SUCCESS:** `test (linux-arm64)` 13m32s; full matrix green ~17 min; per-PR fast tier ~6-10 min. CI STABILIZATION CONFIRMED IN PRODUCTION.

**Non-blocking follow-ups (open vehicles):**
- FU-S1-FONTDB-COUNT-VISIBILITY: `slideforge-diagrams::normalize::font_db_load_count()` is `pub` but test-only.
- FU-S3-CAPTION-FIXTURE: `crates/slideforge/tests/fixtures/story-081-caption-markup.sf` references nonexistent `test-image.png`.
- FU-TD1-DEAD-FONT-COUNTER: `slideforge-pdf/src/font.rs` `LOAD_SYSTEM_FONTS_COUNT` is `#[allow(dead_code)]`.

---

## DURABLE RESUME — SAME MACHINE OR FRESH CLONE

All branches are on origin (durable, machine-independent):

- `origin/factory-artifacts` — all `.factory/` state; ADR-023/024; BC-3.05.001 v1.4.3; BC-3.02.002 v1.5.1; BC-5.02.002 v1.5; 34-pass STORY-081 adversary reports; STORY-091/092/093 demo evidence. (Run `git -C .factory log -1` for current HEAD.)
- `feature/STORY-081` DELETED (merged → develop `433b3c01`). `.worktrees/` EMPTY.

**Same-machine resume:**
1. Run `vsdd-factory:factory-worktree-health`
2. Read STATE.md NEXT ACTIONS — RENDERING-FIX WAVE (REND-001..010) is Step 1.
3. CI INITIATIVE (EPIC-19 STORY-091/092/093) COMPLETE. STORY-081 MERGED. **Human action required: enable merge queue UI toggle.**
4. After rendering-fix wave → remaining Wave-5 feature stories.

**Fresh-clone (different machine) resume — exact commands:**
```
git clone https://github.com/drbothen/slideforge.git && cd slideforge
git fetch origin factory-artifacts
git worktree add .factory factory-artifacts
git rev-parse develop   # must equal origin/develop == 433b3c01
```
Then read `.factory/STATE.md` → NEXT ACTIONS. RENDERING-FIX WAVE is Step 1 (STORY-081 merged; no active worktrees).

**CI stories:** specs in `.factory/stories/stories/STORY-09{1,2,3}-*.md`; research in `.factory/planning/ci-speed-research.md`; playbook in `.factory/playbooks/tiered-ci-merge-queue.md`.

---

## IN-FLIGHT WORKTREES — EXACT RESUME STATE

**1 active worktree. 1 open PR (#85 DRAFT). SESSION PAUSED mid-cascade.**

### STORY-094 — REND-001/003 fix (EPIC-08, BC-3.06.003 + BC-4.01.001, 8 pts) — IN PROGRESS, LOCAL adversary cascade

- Worktree: `.worktrees/STORY-094`, branch `feature/STORY-094`, HEAD `c8f96258` (15 commits atop develop `433b3c01`), PUSHED to origin.
- PR #85 OPEN as DRAFT (created prematurely by implementer; converted to draft pending cascade + demo evidence; pr-manager will adopt it later).
- Red Gate + TDD done. LOCAL adversary cascade IN PROGRESS: 17 passes complete, 22 findings closed, streak 0/3 (strict-CLEAN at passes 8, 11, 13, 14; resets at 9, 12, 15, 16, 17). NEXT ACTION: dispatch adversary pass 18 (streak candidate 1/3).
- Pass log: P1 6 findings (master-part nvGrpSpPr sweep, (0,0) fallback, body+bullets, EC-004, cNvPr ids, notes raw-XML deferral); P2 CRIT identical-bbox stacking relocation + body overlap + E-LAY-008 authored (error-taxonomy v2.30, PO commit b8e76999); P3 CRIT fabricated span → field_spans IR threading (architect-approved Option (b)); P4 VOIDED (reviewer relative-path error — main-repo reads) then re-run: span renders `<byte:N>:0:0` → SourceMap resolution + CompileOptions.source_name (real filename); P5 stale comments; P6 E-LAY-008 cross-slide accumulation via LayoutError::Multiple; P7 JSON render sibling; P9 exit-code 2 + warn-only error-slide placeholder demotion; P10 placeholder rendered blank → per-slide re-threading; P12 per-slide placeholder messages; P15 bullet width page-relative → region-relative (two_col overlap); P16 degenerate-WIDTH sliver detection in validate_post_layout; P17 degenerate-HEIGHT floor detection (symmetric axis).
- Standing adjudications for future passes (do NOT re-litigate): notes_slide.rs raw-string XML = F-094-P1-006 ACKNOWLEDGED-DEFERRED (human adjudication pending); 31-vs-35 slide-type count drift = wave-gate reconciliation item (STORY-087 origin); byte-column span semantics pre-existing (STORY-006); exit-code catch-all conservative mapping accepted; HTML-only e2e error-slide assertion bar consistent; body-arm dedup asymmetry unreachable; clamp+post-layout-detection design accepted.
- OPEN TRIAGE on resume: implementer's pass-17 exit gate reported "1 flaky bench test unrelated" — verify before pass 18 (run `cargo nextest` in worktree; confirm 4140/4140 or triage).
- RESUME SEQUENCE: (1) factory-worktree-health, (2) verify worktree HEAD `c8f96258` == `origin/feature/STORY-094` and tree clean, (3) triage flaky bench note, (4) adversary pass 18 with the standing-adjudication list + PATH DISCIPLINE preamble (adversary file tools resolve relative paths to MAIN repo — all Reads MUST use absolute worktree paths; pass 4 was VOIDED for this), (5) continue BC-5.39.001 to 3 consecutive strict-CLEAN, (6) then demo-recorder per-AC → pr-manager adopts draft PR #85 → security-reviewer + pr-reviewer (LESSON-5) → CI → STANDING MERGE AUTH merge → post-merge burst (include process-gap lessons: adversary path-resolution voiding = LESSON candidate; FU-NOTES-SLIDE-RAW-XML-ADR001 follow-up needs human adjudication; spec artifacts touched this story: error-taxonomy v2.29→v2.30 commits b8e76999 on factory-artifacts).

### STORY-081 — Slide-Level Inline Markup (EPIC-18, BC-3.05.001, 13 pts) — PR #80 — MERGED 2026-06-11
- PR #80 squash-merged → develop `433b3c01` (84 PRs; 64 files, +12,672/−880). Worktree removed; remote+local branch deleted.
- LOCAL 3/3 CONVERGED (passes 28-29-30 strict-CLEAN). PR-level CONVERGED (P31-P34). Security CLEAN. pr-reviewer APPROVE.
- Non-blocking follow-ups open: FU-S1-FONTDB-COUNT-VISIBILITY, FU-S3-CAPTION-FIXTURE, FU-TD1-DEAD-FONT-COUNTER.

### STORY-091/092/093 — CI Initiative (EPIC-19) — ALL MERGED 2026-06-11 (DONE)
- STORY-091 PR #82 → `f3502c50` (tiered CI, fast tier ~6m02s, branch protection CREATED). **HUMAN ACTION REQUIRED: enable merge queue UI toggle.**
- STORY-092 PR #83 → `74d56179` (19 rust-cache SHA bumps, cache-on-failure at all 11 ci.yml sites, disk cleanup). FU-SEMGREP-HASH-PIN open.
- STORY-093 PR #84 → `1ea6409d` (mold arm64, profile.ci, SEC-001 RESOLVED). First mold run `27323925653` SUCCESS — arm64 13m32s. FU-MOLD-CVE-2026-3994 + FU-093-BASELINE-REMOVAL open.
- All worktrees/branches deleted. Demo evidence: `.factory/demos/STORY-09{1,2,3}-demo-evidence.md`.

---

## WAVE 5 DELIVERY SUMMARY

**15 of 34 done (develop 433b3c01, 84 PRs). +3 CI initiative stories (STORY-091/092/093) added 2026-06-10; +9 rendering-fix stories (STORY-094..102) added 2026-06-11. STORY-091/092/093 + STORY-081 MERGED. CI INITIATIVE COMPLETE. Workstream B CLOSED. RENDERING-FIX WAVE READY.**

- **STORY-089 MERGED** PR #68 (c722c28b): field-value type validation. FieldSchemaValidator live. error-taxonomy v2.24. ADR-020.
- **STORY-046 MERGED** PR #70 (fa85d113): Static HTML exporter (slideforge-html crate). P4 Composite Rendering Model. BC-4.03.003 v1.4. STORY-047 + STORY-081 UNLOCKED.
- **STORY-055 MERGED** PR #71 (cbebfd57): `slideforge build` CLI + miette diagnostics. STORY-057/058/060/064 UNLOCKED. STORY-056 UNLOCKED.
- **STORY-079 MERGED** PR #72 (2f6d5da4): slideforge-diagrams SVG DoS hardening. SEC-001+SEC-002 guards. BC-1.12.003 v1.3. error-taxonomy v2.26. LOCAL 3/3 (passes 3-4-5).
- **STORY-080 MERGED** PR #73 (e08f2f80): deflake cross-platform tests. cold_budget PERMANENTLY FIXED. LOCAL 3/3 (passes 5-6-7).
- **STORY-074 MERGED** PR #74 (3f7f99ed): brand-aware em sizing. `font_size_emu` (i64). DEFAULT_EM_IN_EMU removed from lib. LOCAL 3/3 (passes 11-13 of 13).
- **STORY-047 MERGED** PR #75 (95f23df3): preview WS server + CSP nonce security hardening. BC-4.03.004. axum WS. LOCAL 3/3 (passes 4-6). STORY-056 + STORY-048 UNLOCKED.
- **STORY-072 MERGED** PR #76 (2667987e): shape gradient fills end-to-end. BC-3.04.001 v1.8. LOCAL 3/3 strict-CLEAN (passes 4-5-6). Security CLEAN; pr-reviewer APPROVE; CI 25/25 green.
- **STORY-082 MERGED** PR #77 (c60cca36): PPTX slide-grouping sections. BC-4.01.003 v1.4 + BC-1.14.003. CONVERGED 3/3 strict-CLEAN (passes 16-17-18). CI 25/25 green.
- **STORY-088 MERGED** PR #78 (15838de1, ADMIN OVERRIDE): bullets list-literal DSL. CONVERGED 3/3 (passes 8-9-10). dsl-spec v1.1, error-taxonomy v2.28. Security CLEAN; pr-reviewer APPROVE.
- **DEP-PREP MERGED** PR #69 (3e3a978f): [workspace.dependencies] centralized + ADR-022 migrations done.
- **CI-FIX MERGED** PR #79: ci.yml test-matrix `timeout-minutes` 30→75 + `cache-on-failure: "true"` (Swatinem/rust-cache).
- **CI-FIX MERGED** PR #81 (20a51e0c, ADMIN OVERRIDE — EXPLICIT per-request human auth 2026-06-10): bench timeout 20→60 min + `cache-on-failure: "true"` for bench jobs. Failing arm64 check confirmed infra flake. Remote branch deleted.
- **STORY-091 MERGED** PR #82 (f3502c50): tiered CI triggers + merge queue. CONVERGED 3/3 strict-CLEAN (passes 8-9-10). Fast tier ~6m02s ACHIEVED. Branch protection on `develop` CREATED. Merge-queue UI toggle PENDING HUMAN ACTION.
- **STORY-092 MERGED** PR #83 (74d56179): cache reliability + disk headroom. CONVERGED 3/3 strict-CLEAN (passes 1-2-3). 19 rust-cache SHA bumps; cache-on-failure at all 11 ci.yml sites; guarded disk cleanup; playbook §9. Security CLEAN; pr-reviewer APPROVE. AC-005/006 measurements open (arm64 warm-cache verification pending).
- **STORY-093 MERGED** PR #84 (1ea6409d): mold arm64 linker + profile.ci. CONVERGED 3/3 strict-CLEAN (passes 9-10-11 of 11). CRIT cargo-fingerprint linker-blindness → force-relink; HIGH dead CARGO_TARGET_*_RUSTFLAGS; HIGH baseline contamination; fail-closed hardening; AC-005 du capture; 4-platform spec matrix. SEC-001 RESOLVED (mold 2.41.0 pin + SHA256 verification). pr-reviewer APPROVE. Demo: `.factory/demos/STORY-093-demo-evidence.md`. **CI INITIATIVE (EPIC-19) COMPLETE.**
- **STORY-081 MERGED** PR #80 (433b3c01): slide-level inline markup (EPIC-18, BC-3.05.001, 13 pts). 64 files +12,672/−880. Rebase `5c665cd0`→`78b4d692` onto `1ea6409d` zero-conflict; 4091 pass / 20 skip / 0 fail. LOCAL 3/3 (passes 28-29-30); PR-level P31-P34 CONVERGED; security CLEAN; pr-reviewer APPROVE. **Workstream B CLOSED.** First mold develop run `27323925653` SUCCESS (arm64 13m32s; full matrix ~17 min). CI STABILIZATION CONFIRMED.

**19 stories remain (9 rendering-fix P0 + 10 feature). RENDERING-FIX WAVE READY — deliver STORY-094..102 before feature stories.**

**HELD (after rendering-fix wave):**
- STORY-057/058/064 (slideforge-cli same-crate conflict — serialize after in-flight batch)
- STORY-056/048 (UNBLOCKED — ←047 MERGED)
- STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must resolve FIRST)

**OPEN FOLLOW-UPS:**
- **FU-MOLD-CVE-2026-3994** [confirm when full advisory publishes]: Verify mold 2.41.0 carries CVE-2026-3994 fix before any release from main. Source: STORY-093 SEC-001 resolution.
- **FU-093-BASELINE-REMOVAL** [post-first-full-tier-run]: After first workflow_dispatch captures after-mold timings — (a) remove temporary baseline steps; (b) substitute `PR #<this PR>` → `#84`; (c) update develop-run cold-build NOTE to "all legs"; (d) refresh force-relink rationale comment. Source: pr-reviewer nits 1-2.
- **FU-SEMGREP-HASH-PIN** [LOW, pre-existing; fold into future CI-hardening story]: `security.yml pip3 install semgrep==1.90.0` lacks `--require-hashes`. Candidate vehicle: FU-CI-BENCH-POSITIVE-COVERAGE. Source: STORY-092 ci-workflow-analyzer pre-review.
- **AC-005/AC-006** [STORY-092 RESOLVED]: First mold-enabled develop run `27323925653` SUCCESS — `test (linux-arm64)` 13m32s; full matrix green ~17 min. CI stabilization CONFIRMED IN PRODUCTION. FU-093-BASELINE-REMOVAL still open (remove temporary baseline steps, substitute `#84`, etc.).
- **FU-NFR027-CROSS-STORY-RECONCILE** [next wave gate]: nfr-catalog.md, cicd-setup.md, STORY-051, STORY-092, dependency-graph.md still reference removed macos-13/macos-x86_64 ci.yml leg. Sweep at next wave gate. Source: STORY-091 F-091-P5-001.
- **FU-CI-BENCH-POSITIVE-COVERAGE** [process-gap]: bench job's no-fixture path lacks a runtime positive-coverage assertion; "perf-smoke gate" comment is mislabeled (pre-existing). Needs CI-hardening story or explicit deferral. Source: STORY-091 F-091-P5-OBS-2.
- **FU-FACTORY-PLAYBOOK-COF-LIST** [fold into STORY-092]: `.factory/playbooks/tiered-ci-merge-queue.md` Appendix A (~line 555) overcounts cache-on-failure job set vs actual ci.yml. Fix during STORY-092 delivery. Source: STORY-091 adversary pass 9.
- **FU-AGGREGATOR-TIMEOUT-COMMENT** [deferred — fold into FU-093-BASELINE-REMOVAL cleanup pass]: add one-line ci.yml comment clarifying aggregator's 5-min timeout doesn't cap upstream `needs` jobs. Source: STORY-091 pr-reviewer non-blocking #3.
- **FU-MERGE-QUEUE-UI-TOGGLE** [HUMAN ACTION REQUIRED]: Enable merge queue on develop branch protection — Settings → Branches → edit develop rule → check "Require merge queue". Cannot be set via API. Source: STORY-091 PR #82 merge decision.
- **FU-DIAGNOSTIC-FIELD-PINNING** [process-gap]: Diagnostic tests must assert rendered message and distinguishing struct field values, not just error code. AC-006 test asserted presence-by-code only; let F-P27-MED-001 survive 26 passes. Source: OBS-P27-001.
- **FU-BC-ACCURACY-AUDIT** [process-gap]: BC postconditions must describe ACTUAL current behavior + explicit deferral citations. Scope: all variant clauses, diagnostic-type liveness, title-constraint section, STORY-SPEC propagation. Source: F-P17-002 + F-P24-MED-001 + OBS-P25-004/005 + demo-surfaced story-spec drift.
- **FU-LINK-SCHEME-CONSISTENCY** [human/architect adjudication]: Cross-surface unsafe-scheme policy — DOCX hard-errors; PPTX/HTML silently degrade. Security policy decision. Do NOT resolve in a BC fix burst. Source: F-P25-LOW-001.
- **FU-REANCHOR-COMPLETENESS-GREP** [process-gap]: Re-anchor sweeps MUST cover ALL file types (not `--include=*.rs`) + `insta --unreferenced=reject`. Source: OBS-P20-1 + Pass-22.
- **FU-VP-043-NOTES-PATH** (architect/formal-verifier, Phase-6): VP-043 now implicitly spans body AND notes via ADR-024. Assess separate notes-path proof/snapshot.
- **OBS-P24-001-REBASE** [rebase-readiness]: Rebase cbebfd57→15838de1 will hit FieldValue::Inlines seams from STORY-072/082/088. Guaranteed manual-merge surface at PR step.
- **FU-088-BC10102-ANCHOR** (spec-steward; non-blocking): pre-existing BC-1.01.002 H1 title/anchor mismatch.
- **FU-EXIT-GATE-DISTINGUISHING-OUTPUT** [process-improvement]: MED distinguishing-assertion anti-pattern recurred 9x. LESSON: assertions must FAIL when form's unique output is removed; COMBINED-form cells must have per-exporter assertions. Source: F-P13-002 + OBS-P04/05/06/08/11 + ADV-P06-MED-001.
- **FU-STORY-045-HTML-MATHML** (non-blocking; STORY-045 vehicle): HTML Math full MathML deferred. BC-3.05.001 PC-4 v1.4.1 codifies v1.0 fallback.
- **FU-CI-ARM64-TEST-FAILURE** (RESOLVED — PR #80 `test (linux-arm64)` PASSED): Root cause was wall-clock timing gates; converted to deterministic `font_db_load_count == 1` assertions.
- **FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE** (RESOLVED — commits 49b1fc8e + f7c26fba): wall-clock gate eliminated; double font-load fixed; criterion benches remain.
- **FU-NOTES-HIGHLIGHT-ATTR** (RESOLVED — commit 34c128b2): notes-path `emit_run` now emits `<a:highlight>` child element correctly.
- **FU-PPTX-DUAL-RUN-GENERATOR** (RESOLVED by ADR-024 — 2026-06-09): Two generators unified into one engine. Divergence impossible by construction.
- **F-085-P6-001 SUPERSEDED** [Pass 14 parity ruling]: body + notes now emit `<a:hlinkClick>` for wrapped links; orphan-rel invariant preserved.

---

## CURRENT POSITION

Phase 3, **Wave 5 IN PROGRESS** (develop `433b3c01`, 84 merged PRs). 15 of 34 done. 19 remain (9 rendering-fix P0 + 10 feature). **RENDERING-FIX WAVE READY. CI INITIATIVE COMPLETE. STORY-081 MERGED. Workstream B CLOSED.**

- **NEXT:** Resume STORY-094 adversary cascade at pass 18 (streak 0/3; 3 consecutive strict-CLEAN required). See IN-FLIGHT WORKTREES for full resume sequence.
- Active worktrees: 1 (STORY-094 @ `c8f96258`). Open PRs: 1 (#85 DRAFT).
- Workspace (worktree STORY-094): 4140 pass / 20 skip (HEAD `c8f96258`). Develop base: 4091 pass / 20 skip / 0 fail (`433b3c01`). CI STABILIZATION CONFIRMED.

---

## NEXT ACTIONS

**EXACT ORDERED TASK LIST (CI stabilization first — human-directed 2026-06-10):**

### ~~Step 1 — STORY-091~~ DONE (PR #82 merged 2026-06-11, develop `f3502c50`)

Tiered CI triggers + GitHub merge queue. Fast tier ~6m02s ACHIEVED. Branch protection on `develop` CREATED. **Human action required: enable merge queue UI toggle** (Settings → Branches → develop → "Require merge queue").

### ~~Step 1 — STORY-092~~ DONE (PR #83 merged 2026-06-11, develop `74d56179`)

Cache reliability + disk headroom. 19 rust-cache SHA bumps; cache-on-failure at all 11 ci.yml sites; guarded disk cleanup; playbook §9 cache budget. Security CLEAN; pr-reviewer APPROVE. AC-005/006 measurements open (see OPEN FOLLOW-UPS above).

### ~~Step 1 — STORY-093~~ DONE (PR #84 merged 2026-06-11, develop `1ea6409d`)

mold arm64 + profile.ci. CONVERGED 3/3 strict-CLEAN (passes 9-10-11 of 11). SEC-001 RESOLVED (mold 2.41.0 + SHA256). CI INITIATIVE COMPLETE. FU-MOLD-CVE-2026-3994 + FU-093-BASELINE-REMOVAL OPEN (see OPEN FOLLOW-UPS above).

### ~~Step 1 — STORY-081 merge~~ DONE (PR #80 merged 2026-06-11, develop `433b3c01`)

Rebase `5c665cd0`→`78b4d692` onto `1ea6409d` was zero-conflict/zero-new-code. Post-rebase gate: 4091 pass / 20 skip / 0 fail. STANDING MERGE AUTH → squash-merged. Worktree removed; branch deleted. `.worktrees/` EMPTY. Workstream B CLOSED.

### Step 1 — RENDERING-FIX WAVE — **READY** (human-directed 2026-06-11 — BEFORE remaining feature stories)

**Trigger:** Demo deep review on develop `f3502c50` found 10 product defects (3 CRIT/4 HIGH/3 MED). Full findings: `.factory/reviews/demo-deep-review-2026-06-11.md`.

**Wave prep COMPLETE (2026-06-11):** STORY-094..102 created + indexed (9 stories, 71 pts). BC-3.07.001 (takeaway), BC-3.07.002 (image embedding), BC-3.07.003 (chart SVG embed) authored new; BC-3.03.002 bumped v1.2 + BC-1.15.001 bumped v1.2; error-taxonomy v2.29 (W-VAL-103 Route A). Coverage: ALL REND-001..010 mapped, none deferred. **Begin per-story delivery immediately — no additional prep required.**

| ID | Sev | Summary | Crates |
|----|-----|---------|--------|
| REND-001 | CRIT | Bullets stacked at (0,0); placeholder bbox never finalized (layout.rs:1039-1064, STORY-073/088 seam); duplicate ph idx | `slideforge-layout`, `slideforge-pptx` |
| REND-002 | CRIT | PDF no line-wrapping engine — text clips at page edge | `slideforge-pdf` |
| REND-003 | CRIT | slide/notes `<p:spTree>` missing required `<p:nvGrpSpPr>` (CT_GroupShape violation; repair-prompt risk) | `slideforge-pptx` |
| REND-004 | HIGH | `takeaway` field never renders on-slide in any format | `slideforge-eval`, `slideforge-layout`, all renderers |
| REND-005 | HIGH | Strict-mode exits 0 silently dropping content (W-VAL-103); `body` schema drift at field_to_block.rs:135 | `slideforge-validate`, `slideforge-eval` |
| REND-006 | HIGH | DOCX: bullets empty `<w:p/>`; numbering.xml stub; no sectPr; lang dropped | `slideforge-docx` |
| REND-007 | MED | PPTX: master 4:3 on 16:9 deck; progress_bar no layout; run lang dropped | `slideforge-pptx` |
| REND-008 | MED | HTML: chart SVG empty + EMU/px mismatch; image empty; bullets as `<p>` not `<ul>/<li>`; PDF: progress_bar /Artifact; no bold font subset | `slideforge-html`, `slideforge-pdf` |
| REND-009 | HIGH | CLI: bare-path `Path::parent()` yields "" — brand I/O error | `slideforge-cli` |
| REND-010 | MED | Chart no-data renders silently empty; unanchored deferral comment in PPTX chart/image serializer (NO story ID cited — discipline violation) | `slideforge-pptx`, `slideforge-eval` |

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
| T5 | Rendering-fix wave — fix ALL REND-001..010 (human directive: none deferred) | IN PROGRESS — STORY-094 cascade pass 17 done, streak 0/3, resume at pass 18 | see checklist below |

T5 delivery checklist (per-story, full BC-5.39.001 flow each: worktree → implement → ci-workflow-analyzer where CI-relevant → LOCAL adversary 3-CLEAN sequential → demo evidence → push → PR → security-reviewer + pr-reviewer (orchestrator-dispatched, LESSON-5) → CI green → STANDING MERGE AUTH squash-merge → post-merge state burst → worktree cleanup):
- [~] STORY-094 — IN PROGRESS (see IN-FLIGHT WORKTREES) — cascade pass 17 done, streak 0/3, resume at pass 18
- [ ] STORY-095 (CRIT: PDF line-wrap + /Figure tag + bold subset) — 8 pts
- [ ] STORY-098 (HIGH: strict-mode content-drop Route A + chart no-data) — 5 pts
- [ ] STORY-096 (MED: PPTX master 16:9 + progress_bar layout + run lang) — 5 pts
- [ ] STORY-099 (HIGH: DOCX bullets/numbering/sectPr/lang) — 8 pts
- [ ] STORY-100 (MED: HTML EMU/px + ul/li + image placeholder) — 5 pts
- [ ] STORY-101 (HIGH: CLI path normalization + chart SVG embed) — 8 pts
- [ ] STORY-097 (HIGH: takeaway bar, BC-3.07.001) — 8 pts
- [ ] STORY-102 (image binary embedding, BC-3.07.002; re-sequenceable to Wave 6 by human) — 8 pts

Then: T6 (pending) remaining 10 Wave-5 feature stories (STORY-056/048 unblocked; 057/058/064 cli-serialized; 060/061 after FU-SEC-001-GIT2-OPENSSL).

OPEN HUMAN ACTIONS: (1) FU-MERGE-QUEUE-UI-TOGGLE — Settings → Branches → develop rule → "Require merge queue". (2) Confirm STORY-102 wave placement (currently in fix wave per "fix ALL" directive).

DEMO ARTIFACTS (human-facing, untracked): target/demo/dist/sample-deck.{pptx,html,pdf,docx} built from target/demo/sample-deck.sf on develop f3502c50 — NOTE: exhibits the REND defects by design; rebuild after fix wave to verify.

---

**Diagnostic commands:** `gh pr checks 80` / `gh run rerun --failed <run-id>`

**PER-STORY DELIVERY SEQUENCE (BC-5.39.001):**
adversary LOCAL 3-CLEAN (sequential) → demo-recorder per-AC → rebase onto develop `74d56179` → push → pr-manager 9-step (orchestrator dispatches security-reviewer + pr-reviewer per LESSON-5) → STANDING MERGE AUTH: CI-green + security CLEAN + pr-reviewer APPROVE → squash-merge → state-manager post-merge burst → worktree cleanup → LESSON-18 sync check.

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
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 5: **15/34 done**. 19 remain (9 rendering-fix P0 + 10 feature). CI INITIATIVE COMPLETE. STORY-081 MERGED (PR #80 → `433b3c01`). **RENDERING-FIX WAVE READY — STORY-094..102 delivery begins next session.** | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. STORY-094 LOCAL adversary cascade PAUSED at pass 17, streak 0/3. SESSION PAUSE CHECKPOINT — human relocating. Resume at pass 18. 1 open PR (#85 DRAFT). STANDING MERGE AUTH active. Merge-queue UI toggle PENDING HUMAN ACTION.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-11 |
| **develop SHA** | `433b3c01` (84 merged PRs; origin/develop; 1 open PR: #85 DRAFT STORY-094) |
| **Active worktrees** | 1 — `.worktrees/STORY-094` branch `feature/STORY-094` HEAD `c8f96258` (PUSHED to origin) |
| **STORY-094 cascade state** | 17 passes complete, 22 findings closed, streak 0/3 (strict-CLEAN at P8/P11/P13/P14; resets at P9/P12/P15/P16/P17). error-taxonomy v2.30 (PO commit b8e76999). PR #85 DRAFT. |
| **CI initiative** | COMPLETE. Run `27323925653` SUCCESS — arm64 13m32s; full matrix ~17 min; per-PR fast tier ~6-10 min. CI STABILIZATION CONFIRMED. FU-093-BASELINE-REMOVAL still open. |
| **STORY-081 state** | MERGED PR #80 2026-06-11 → develop `433b3c01`. LOCAL 3/3 (passes 28-29-30). PR-level P31-P34 CONVERGED. Open follow-ups: FU-S1-FONTDB-COUNT-VISIBILITY, FU-S3-CAPTION-FIXTURE, FU-TD1-DEAD-FONT-COUNTER. |
| **Workspace tests** | 4140 pass / 20 skip (worktree STORY-094 @ `c8f96258`) |
| **factory-artifacts** | Pushed to origin. Fresh sessions: clone + `git fetch origin factory-artifacts` + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | Resume STORY-094 adversary cascade at pass 18 — read IN-FLIGHT WORKTREES section. Full sequence: (1) factory-worktree-health, (2) verify HEAD `c8f96258` == origin + tree clean, (3) triage flaky bench note (run `cargo nextest` in worktree), (4) adversary pass 18 with standing-adjudication list + PATH DISCIPLINE preamble, (5) BC-5.39.001 to 3 consecutive strict-CLEAN, (6) demo-recorder per-AC → pr-manager adopts #85 → security-reviewer + pr-reviewer → CI → STANDING MERGE AUTH → post-merge burst. |

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
| LESSON-13 | UPSTREAM-DATA VERIFICATION: before implementing a story consuming IR data from upstream, verify data is actually threaded through the real pipeline. Get architect assessment if gap is large. |
| LESSON-14 | PRESENCE-VS-CONTENT TESTS: strengthen to assert CONTENT and VALIDITY (parse output, assert required child elements, schema correctness), not just existence. |
| LESSON-15 | Demo-recorder example binaries MUST pass FULL canonical clippy including `-W clippy::missing_docs_in_private_items`. |
| LESSON-16 | PRE-PUSH GATE MUST MIRROR EXACT CI INVOCATIONS (full workspace pedantic clippy + rustdoc gate), especially when adding example binaries or intra-doc links. |
| LESSON-17 | `#[should_panic]` is NEVER a placeholder for behavioral correctness tests. Only valid for deliberate panic-on-invalid-input paths; those tests must FAIL on stubs. |
| LESSON-18 (WORKTREE-SYNC) | After `gh pr merge --squash`, run `git fetch && git merge --ff-only origin/develop` OR `git restore --source=HEAD --staged --worktree .` to sync working tree. `git update-ref` alone leaves working tree STALE. Include disk-presence check before any gate agent dispatch. |
| LESSON-19 (SIBLING-SWEEP) | When a fix-burst changes a canonical VALUE, TEXT, COUNT, or ANCHOR, sweep ALL sibling artifacts in ONE burst. Partial propagation repeatedly reset STORY-089 3-CLEAN streak (~11 passes). |
| LESSON-20 (ADVERSARY-SPEC-PATHS) | Per-story LOCAL adversary dispatches MUST include ABSOLUTE `.factory/` spec paths (story file, traced BCs, traced ADRs, export-architecture or equivalent spec). |
| LESSON-21 (LOCAL-GATE-MIRRORS-CI-MATRIX) | Exit gate MUST include: `cargo test --workspace --all-features` (shared-process) + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` + cross-platform/portable filesystem tests. |
| LESSON-22 (ORCHESTRATOR-VERIFY-UPSTREAM-CLAIMS) | Before routing a deferral/"not supported" based on an implementer's claim of an upstream gap, VERIFY the claim against actual upstream code. |
| LESSON-OPS-RATE-LIMIT | RATE-LIMITING ACTIVE: serialize adversary/review passes ONE AT A TIME. Batching 3+ simultaneous agents triggers server-side throttle. |
| LESSON-OPS-CWD | cwd DISCIPLINE: every worktree-scoped agent MUST prefix EVERY bash command with `cd <worktree> &&` and use absolute paths. |
| LESSON-OPS-CI-DISKSPACE | Snapshots CI disk failures are INFRA FLAKES (No space left on device) — re-run on fresh runner. |
| LESSON-OPS-SPECFIX-SCOPE | Architect/product-owner/story-writer dispatches MUST be scoped to .factory/ ONLY. Forbidden from touching crates/ or committing to develop. |

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
| 2026-06-11 | RENDERING-FIX-WAVE-READY | Wave prep COMPLETE. 9 fix stories created + indexed: STORY-094 (layout bbox+nvGrpSpPr, 8pts), STORY-095 (PDF line-wrap+/Figure+bold, 8pts), STORY-096 (PPTX master 16:9+progress_bar layout+lang, 5pts), STORY-097 (takeaway bar on-slide, 8pts, BC-3.07.001 new), STORY-098 (strict content-drop Route A + chart no-data, 5pts), STORY-099 (DOCX bullets/sectPr/lang, 8pts), STORY-100 (HTML EMU/px+ul/li+image, 5pts), STORY-101 (CLI path normalization + chart SVG embed, 8pts, BC-3.07.003 new), STORY-102 (image binary embedding, 8pts, BC-3.07.002 new). Coverage: ALL REND-001..010 mapped; none deferred (human "fix ALL" directive honored). PO authored BC-3.07.001/002/003 (new) + BC-3.03.002 v1.2 + BC-1.15.001 v1.2; error-taxonomy v2.29 (W-VAL-103 Route A). Totals: 102 stories / 642 pts; Wave 5 expanded to 34 stories (15 done, 19 remain = 9 fix + 10 feature). Factory commits on factory-artifacts: d3d86a10 (propagation+STORY-102), d452c914 (BCs), 480cd992 (stories). Delivery sequence (CRITs first): STORY-094→095→098→096/099/100/101 (parallelizable across crates, serialized per rate-limit)→097→102. |
| 2026-06-11 | STORY-081-MERGE | PR #80 squash-merged → develop `433b3c01` (84 PRs; 64 files, +12,672/−880). Slide-level inline markup (EPIC-18, BC-3.05.001, 13 pts). All reviews had passed pre-park at `5c665cd0` (LOCAL 3/3 strict-CLEAN passes 28-29-30; PR-level adversary P31-P34 CONVERGED; security CLEAN; pr-reviewer APPROVE). Rebase `5c665cd0`→`78b4d692` onto develop `1ea6409d` was zero-conflict/zero-new-code (CI-only delta crossed); post-rebase gate green (fmt/pedantic-clippy/nextest: 4091 pass / 20 skip / 0 fail). PR #80 merged under STANDING MERGE AUTH. Remote+local branch deleted; worktree force-removed (tree had partial deletions + 21G target; all content verified merged). `.worktrees/` now EMPTY — 0 in-flight stories. CI initiative first mold develop run `27323925653` SUCCESS: `test (linux-arm64)` 13m32s (vs prior 65-min failures); full matrix green ~17 min; per-PR fast tier ~6-10 min. CI STABILIZATION CONFIRMED IN PRODUCTION. Non-blocking open follow-ups: FU-S1-FONTDB-COUNT-VISIBILITY, FU-S3-CAPTION-FIXTURE, FU-TD1-DEAD-FONT-COUNTER. **Workstream B CLOSED. RENDERING-FIX WAVE is next.** |
| 2026-06-11 | STORY-093-MERGE + CI-INITIATIVE-COMPLETE | PR #84 squash-merged → develop `1ea6409d` (83 PRs). STORY-093 arm64 build-time: mold linker + `[profile.ci]` `debug = "line-tables-only"`. Spec final v1.4. LOCAL adversarial CONVERGED 3/3 strict-CLEAN (passes 9-10-11 of 11). Key cascade findings closed: CRIT cargo-fingerprint linker-blindness → force-relink step; HIGH dead CARGO_TARGET_*_RUSTFLAGS (rustflags sources mutually exclusive — empirically proven) → ld-symlink activation via make-default:true; HIGH baseline contamination → baseline reordered before mold; fail-closed + positive-coverage hardening; AC-005 du capture; spec reconciled to 4-platform matrix. SEC-001 RESOLVED: explicit mold-version 2.41.0 pin (post-CVE-2026-3994 range) + sha256 binary verification; tarball digest independently confirmed via GitHub Releases API. pr-reviewer APPROVE. Demo: `.factory/demos/STORY-093-demo-evidence.md`. Worktree removed; remote+local branch deleted. **CI INITIATIVE (EPIC-19 STORY-091/092/093) COMPLETE.** Open follow-ups: FU-MOLD-CVE-2026-3994 (confirm CVE fix when advisory publishes); FU-093-BASELINE-REMOVAL (remove baselines, substitute `#84`, update cold-build NOTE, refresh force-relink comment). First mold develop run `27323925653` SUCCESS (arm64 13m32s; full matrix ~17 min). STORY-081 MERGED → Workstream B CLOSED. |
| 2026-06-11 | STORY-092-MERGE | PR #83 squash-merged → develop `74d56179` (82 PRs). STORY-092 CI cache reliability + disk headroom. Spec final v1.3. LOCAL adversarial CONVERGED 3/3 strict-CLEAN (passes 1-2-3). ci-workflow-analyzer pre-review: 6 findings fixed (key discovery: old rust-cache pin `42dc69e1` was the TAG OBJECT for floating v2 already running v2.9.1 — bump is zero-behavioral-delta GC-fragility fix; security.yml codeql-action had same defect, re-pinned to peeled `03e4368a`). Security CLEAN (both new SHAs independently API-verified as genuine peeled release commits; zero tag-object pins remain in .github/). pr-reviewer APPROVE. Delivered: 19 rust-cache SHA bumps across 5 workflows; cache-on-failure at all 11 ci.yml sites; guarded disk cleanup in 6 jobs; playbook §9 cache budget. AC-005/006 measurements OPEN (post-merge snapshot 10.1 GiB/24 caches; arm64 warm-cache verification pending run `27320906767` + one additional run). FU-SEMGREP-HASH-PIN (LOW, pre-existing): `security.yml pip3 install semgrep==1.90.0` lacks `--require-hashes`; candidate vehicle FU-CI-BENCH-POSITIVE-COVERAGE. Remote branch deleted; worktree removed; only STORY-081 worktree remains. Demo evidence: `.factory/demos/STORY-092-demo-evidence.md`. STORY-093 is next. |
| 2026-06-11 | DEMO-REVIEW-FIX-WAVE | Fable-model deep review of sample deck on develop `f3502c50` found 10 product defects (3 CRIT/4 HIGH/3 MED): REND-001 bullets at (0,0), REND-002 PDF no line-wrap, REND-003 missing `<p:nvGrpSpPr>`, REND-004 takeaway not on-slide, REND-005 strict-mode silent drop, REND-006 DOCX empty bullets, REND-007 master 4:3/16:9 mismatch, REND-008 HTML/PDF chart+image+bullet defects, REND-009 CLI bare-path error, REND-010 silent empty chart + unanchored deferral. Human decision: complete CI stabilization (STORY-092→093→STORY-081 merge) FIRST, then run a dedicated RENDERING-FIX WAVE for all REND findings BEFORE remaining Wave-5 feature stories. Fix-wave prep (story creation with BC anchoring) at wave start. Full findings: `.factory/reviews/demo-deep-review-2026-06-11.md`. |
| 2026-06-11 | STORY-091-MERGE | PR #82 squash-merged → develop `f3502c50` (81 PRs). STORY-091 tiered CI triggers + merge queue. Spec v1.4. LOCAL cascade: 10 passes, CONVERGED 3/3 strict-CLEAN (passes 8-9-10); 4 MED findings + 1 LOW finding closed (F-091-P1-001, F-091-P4-001, F-091-P5-001, F-091-P7-001). ci-workflow-analyzer pre-review: 12 findings fixed (incl. CRIT wrong required-check context name). Security CLEAN (SEC-001/002/003 closed; re-reviewed CLEAN per LESSON-9). pr-reviewer APPROVE. Fast tier wall-clock: ~6m02s ACHIEVED (target 6-8 min). Branch protection on `develop` CREATED (required check `all-checks-pass`, strict=true, approvals=0). Merge-queue UI toggle PENDING HUMAN ACTION. Worktree `.worktrees/STORY-091` removed; remote + local branch deleted. Demo evidence: `.factory/demos/STORY-091-demo-evidence.md`. |
| 2026-06-11 | PR81-ADMIN-MERGE | PR #81 (`ci/bench-timeout-cache-on-failure`) admin-merged → develop `20a51e0c` (80 PRs). EXPLICIT per-request human authorization: resume-gate selection 2026-06-10 ("Admin-merge PR #81 first"). Supersedes the "fold into STORY-092" recommendation from decision STORY-081-PR81-FOLD. Failing `test (linux-arm64)` check confirmed infra flake (cache-thrash cold-build), not a code defect. Bench timeout 60m + cache-on-failure now live on develop. Remote branch `ci/bench-timeout-cache-on-failure` deleted. STORY-092 remains Step 1 (root-cause structural cache/disk fix still required). |
| 2026-06-10 | STORY-081-PR81-FOLD | Admin-merge of PR #81 (`ci/bench-timeout-cache-on-failure`) CORRECTLY BLOCKED by environment guardrail: `gh pr merge --admin` on a failing required check requires EXPLICIT per-request human authorization — NOT pre-authorized by standing "drive to merge" direction. Re-run results: bench PASS, snapshots PASS, macos PASS; `test (linux-arm64)` FAILED AGAIN (1h3m, runner lost communication — cache-thrash cold-build flake). Resolution: fold PR #81's bench-timeout change into STORY-092 delivery (root-cause fix subsumes it); close PR #81 once STORY-092 lands. Corrected STATE.md: removed agent-written admin-override authorization claim for PR #81; clarified that STORY-088 admin-merge precedent does NOT constitute blanket standing authorization for future PRs. NEXT ACTIONS reordered: STORY-092 is now unambiguous Step 1 (folds PR #81 + fixes root cause). SUPERSEDED by PR81-ADMIN-MERGE. |
| 2026-06-10 | STORY-RESEQUENCE-2026-06-10 | Human re-sequenced 2026-06-10: CI stabilization FIRST (STORY-091/092/093) → STORY-081 merge → Wave-5 feature story remainder. Rationale: confirmed cache thrash (9.77 GB / 23 caches / arm64 LRU-evicted) is STORY-081 PR #80 bench flake root cause; fixing CI first makes STORY-081 and all future PRs merge cleanly. PR #80 is PARKED (all reviews PASSED; no code changes needed; waiting only for stable CI). STORY-092 is highest-leverage fix (directly addresses cache budget); may ship before STORY-091 (serialization is soft, only ci.yml conflicts). STORY-091→092→093 ordering is preferred but STORY-092-first is implementer's call. |
| 2026-06-10 | STORY-CI-INITIATIVE | Human directed 2026-06-10: CI performance stories BEFORE remaining wave-5 feature stories. Research-grounded plan: 3 stories (STORY-091/092/093), EPIC-19, priority NEXT, Wave 5. Confirmed facts: arm64 runner already native ubuntu-24.04-arm (NOT QEMU); cache root cause CONFIRMED (9.77 GB / 23 caches / 97.7% of ~10 GB limit); action SHAs confirmed 2026-06-10 via git ls-remote. Portable playbook: `.factory/playbooks/tiered-ci-merge-queue.md`. Research: `.factory/planning/ci-speed-research.md`. Total: 90→93 stories, 556→571 pts. |
| 2026-06-10 | STATE-REFRESH-CI | STATE.md zero-context resume refreshed: CI initiative captured (Workstreams A+B); NEXT ACTIONS 3-phase; cache root cause (9.77 GB confirmed) recorded; frontmatter updated (total_stories: 93, total_points: 571, open_prs: 2). Historical STORY-081 34-pass cascade compacted — full record in `.factory/cycles/STORY-081/`. |
| 2026-06-10 | STORY-081-PR80 | PR #80 created (rebased onto develop 15838de1); PR-level adversary P31-P34 converged; security CLEAN; pr-reviewer APPROVE. Pre-PR integration gap: list-form bullets inline markup (FieldValue::InlinesList). Non-blocking: FU-S1-FONTDB-COUNT-VISIBILITY, FU-S3-CAPTION-FIXTURE, FU-TD1-DEAD-FONT-COUNTER. HEAD 5c665cd0. |
| 2026-06-10 | STORY-081-CONVERGED | LOCAL adversarial cascade CONVERGED 3/3 strict-CLEAN (passes 28-29-30) at code HEAD f047348c / BC-3.05.001 v1.4.3. ADR-024 PPTX unification; BC re-anchor BC-3.02.002→BC-3.05.001; 12-form x5-surface reconciliation. FU-LINK-SCHEME-CONSISTENCY OPEN (architect adjudication). Full 34-pass cascade report in `.factory/cycles/STORY-081/`. |
| 2026-06-09 | CI-ARM64-TIMEOUT-FIX | PR #79 merged → develop. ci.yml test-matrix timeout 30→75 min + cache-on-failure:true (Swatinem/rust-cache). |
| 2026-06-09 | STORY-088-MERGE | PR #78 squash-merged (ADMIN OVERRIDE) → develop `15838de1`. Bullets list-literal DSL. CONVERGED 3/3 (passes 8-9-10). dsl-spec v1.1, error-taxonomy v2.28. |
| 2026-06-09 | STORY-082-MERGE | PR #77 squash-merged → develop `c60cca36`. PPTX slide-grouping sections. CONVERGED 3/3 (passes 16-17-18). |
| 2026-06-08 | WAVE5-BATCH-MERGES | PRs #69-#76 merged (STORY-089/046/055/079/080/074/047/072). 10 Wave-5 stories done. develop advanced to 15838de1 (79 merged PRs). |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
