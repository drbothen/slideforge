---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-10
state_version: "1.3"
phase_1_approved: 2026-05-25
phase_2_approved: 2026-05-25
phase_1_convergence: "17 passes, 69 findings, 3/3 clean (passes 15-16-17)"
phase_2_convergence: "22 passes, 96+ findings, 3/3 clean (passes 20-21-22)"
prd_bcs: 117
prd_hs: 15
prd_vps: 15
prd_supplements: 4
spikes_resolved: 7
spikes_total: 7
total_stories: 93
total_points: 571
total_waves: 6
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
wave_5_status: "10 of 25 Wave-5 stories MERGED. 15 remain (12 feature + 3 CI initiative STORY-091/092/093). RESEQUENCED 2026-06-10: CI stabilization FIRST (091/092/093), THEN STORY-081 merge, THEN remaining feature stories. PARKED: STORY-081 PR #80 (OPEN, HEAD 5c665cd0, MERGEABLE — all reviews PASSED; waiting for CI stabilization). PR #81 (ci/bench-timeout-cache-on-failure): bench PASSED; infra flakes on snapshots/arm64 — rerun or admin-merge. ROOT CAUSE confirmed: 9.77 GB / 23 caches / 97.7% of ~10 GB limit — arm64 LRU-evicted — STORY-081 bench flake."
develop_sha: "15838de1"
develop_pr_count: 79
open_prs: 2
error_taxonomy_version: "v2.28"
workspace_tests: "4083 pass / 20 skip / 0 fail (STORY-081 worktree HEAD 5c665cd0; post list-bullets x088 fix; includes STORY-072/082/088 tests)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge (public) | **Default branch:** `main` | **Dev branch:** `develop`
**Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop`. Canonical SHA: `15838de1` (79 merged PRs, 2 open PRs: #80, #81).

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04; ongoing pushes authorized).

**Current position:** Phase 3, **Wave 5 IN PROGRESS**. 10 of 25 done (93 stories / 571 pts). 15 stories remain (12 feature + 3 CI initiative).

**STANDING MERGE AUTH:** Orchestrator MAY squash-merge any PR that is CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking human.

---

### NEW ORDER (human re-sequenced 2026-06-10): CI STABILIZATION FIRST

**Rationale:** Confirmed cache thrash (9.77 GB / 23 caches / arm64 LRU-evicted) is the root cause of STORY-081 bench flakiness. Fix CI first so STORY-081 and all future PRs merge cleanly.

**Execute in this order:**
1. CI stabilization: deliver STORY-091/092/093 (and optionally merge PR #81)
2. STORY-081: rebase PR #80 onto stabilized develop → merge
3. Remaining Wave-5 feature stories

---

### Workstream A — CI Stabilization (DO FIRST)

**STORY-081's PR #80 is PARKED here.** All reviews PASSED (PR-level adversary P31-P34 converged; security CLEAN; pr-reviewer APPROVE; workspace 4083 pass / 20 skip / 0 fail). Waiting only for CI to stabilize.

**Step A0 — PR #81 (optional quick win or fold into STORY-092):**
`ci/bench-timeout-cache-on-failure` → develop: bench timeout 20→60 min + `cache-on-failure: "true"`. bench PASSED (23m43s); snapshots + `test (linux-arm64)` failed infra flakes (re-runnable). Merge when green, OR admin-merge past diagnosed arm64 flake (STANDING AUTH; STORY-088 precedent). Alternative: fold PR #81 into STORY-092 delivery and close it — implementer's choice.

**Step A1 — STORY-092 (HIGHEST-LEVERAGE — consider delivering first):**
Cache reliability + disk headroom. Fixes the CONFIRMED 9.77 GB/23-cache thrash. Brings arm64 cache under budget; eliminates cold-build LRU-eviction. Bumps rust-cache → v2.9.1 SHA `c19371144df3bb44fab255c43d04cbc2ab54d1c4` at all 11 ci.yml sites (grep-zero on old `42dc69e`); cache-on-failure on all build-heavy jobs; disk cleanup (`jlumbroso/free-disk-space` SHA `54081f13...`). Story spec: `.factory/stories/stories/STORY-092-ci-cache-reliability-disk.md`. NOTE: the 091→092 ordering is soft (ci.yml-conflict-serialization only); 092 MAY ship first.

**Step A2 — STORY-091 (biggest per-PR wall-clock win):**
Tiered CI triggers + GitHub merge queue. Gates slow legs (arm64/windows/macos/bench) off the per-PR path via `merge_group` + job-level `if:`; single required check = `all-checks-pass`. Verify at impl: enable merge queue on `develop` branch protection (non-default; default is `main`); `schedule:` nightly fires from `main` only. Story spec: `.factory/stories/stories/STORY-091-ci-tiered-triggers-merge-queue.md`. Playbook: `.factory/playbooks/tiered-ci-merge-queue.md`.

**Step A3 — STORY-093 (arm64 build-time):**
mold linker (arm64 only, `setup-mold` SHA `9c9c13bf...`) + `[profile.ci]` `debug = "line-tables-only"` via `--cargo-profile ci`. Story spec: `.factory/stories/stories/STORY-093-ci-arm64-build-time.md`.

**Confirmed facts (do not re-research):**
- arm64 runner is ALREADY native `ubuntu-24.04-arm` (NOT QEMU). Slowness is cache-eviction, not emulation.
- Action SHAs confirmed 2026-06-10 via git ls-remote: rust-cache `c19371144...`, free-disk-space `54081f13...`, setup-mold `9c9c13bf...`, install-action `fd2f5e3d...`. Verify at impl time (pinned SHAs may rotate).
- Research sidecar: `.factory/planning/ci-speed-research.md`.

---

### Workstream B — STORY-081 (PARKED — merge AFTER CI stabilized)

PR #80 (`feature/STORY-081` HEAD `5c665cd0` → develop). PARKED pending CI stabilization.

**State:** LOCAL adversarial CONVERGED 3/3 (passes 28-29-30). PR-level CONVERGED (P31-P34, all CRIT/HIGH/MED resolved). Security CLEAN. pr-reviewer APPROVE. Workspace 4083 pass / 20 skip / 0 fail.

**ROOT CAUSE of PR #80 bench failure:** cache thrash (STORY-092 fixes). Not a code defect.

**After 091/092/093 land on develop:**
1. Rebase `feature/STORY-081` onto stabilized develop → `git push --force-with-lease`
2. Re-run PR #80 CI (bench now passes with warm cache + 60-min budget)
3. STANDING MERGE AUTH → squash-merge PR #80 → post-merge state burst → worktree cleanup (`.worktrees/STORY-081`)

**Non-blocking follow-ups (post-merge; do NOT block PR #80):**
- FU-S1-FONTDB-COUNT-VISIBILITY: `slideforge-diagrams::normalize::font_db_load_count()` is `pub` but test-only.
- FU-S3-CAPTION-FIXTURE: `crates/slideforge/tests/fixtures/story-081-caption-markup.sf` references nonexistent `test-image.png`.
- FU-TD1-DEAD-FONT-COUNTER: `slideforge-pdf/src/font.rs` `LOAD_SYSTEM_FONTS_COUNT` is `#[allow(dead_code)]`.

---

## DURABLE RESUME — SAME MACHINE OR FRESH CLONE

All branches are on origin (durable, machine-independent):

- `origin/factory-artifacts` — all `.factory/` state; ADR-023/024; BC-3.05.001 v1.4.3; BC-3.02.002 v1.5.1; BC-5.02.002 v1.5; 34-pass STORY-081 adversary reports. (Run `git -C .factory log -1` for current HEAD.)
- `origin/feature/STORY-081` @ `5c665cd0` — PR #80 OPEN (→ develop, PARKED pending CI stabilization). LOCAL adversarial CONVERGED 3/3 (passes 28-29-30). PR-level CONVERGED (P31-P34). Security CLEAN. pr-reviewer APPROVE.
- `origin/ci/bench-timeout-cache-on-failure` — PR #81 OPEN (→ develop, MERGEABLE). bench timeout 20→60 min + cache-on-failure. Merge or fold into STORY-092.

**Same-machine resume:**
1. Run `vsdd-factory:factory-worktree-health`
2. Read STATE.md NEXT ACTIONS — CI stabilization is the first task.
3. Check `gh pr checks 81` — rerun infra flakes or admin-merge; then deliver STORY-092 → 091 → 093.
4. After CI stories land → rebase STORY-081 → merge PR #80.

**Fresh-clone (different machine) resume — exact commands:**
```
git clone https://github.com/drbothen/slideforge.git && cd slideforge
git fetch origin factory-artifacts feature/STORY-081 ci/bench-timeout-cache-on-failure
git worktree add .factory factory-artifacts
git worktree add .worktrees/STORY-081 feature/STORY-081
git rev-parse develop   # must equal origin/develop == 15838de1
```
Then read `.factory/STATE.md` → NEXT ACTIONS. CI stories first; STORY-081 is parked.

**CI stories:** specs in `.factory/stories/stories/STORY-09{1,2,3}-*.md`; research in `.factory/planning/ci-speed-research.md`; playbook in `.factory/playbooks/tiered-ci-merge-queue.md`.

---

## IN-FLIGHT WORKTREES — EXACT RESUME STATE

**1 active worktree. 2 open PRs. NEW ORDER: CI stabilization before STORY-081 merge.**

### STORY-081 — Slide-Level Inline Markup (EPIC-18, BC-3.05.001, 13 pts) — PR #80 — PARKED
- **Worktree:** `.worktrees/STORY-081` | **Branch:** `feature/STORY-081` | **HEAD:** `5c665cd0`. PR #80 OPEN → develop, MERGEABLE, **PARKED pending CI stabilization.**
- **All reviews DONE:** LOCAL 3/3 CONVERGED (passes 28-29-30 strict-CLEAN). PR-level CONVERGED (passes P31-P34, all CRIT/HIGH/MED resolved). Security CLEAN. pr-reviewer APPROVE. Workspace 4083 pass / 20 skip / 0 fail.
- **Bench failure cause:** CI cache thrash (9.77 GB/23 caches, arm64 LRU-evicted). STORY-092 fixes this. Not a code defect.
- **Merge sequence (after CI stabilized):** rebase onto new develop → force-push → re-run PR #80 CI → squash-merge → post-merge state burst → worktree cleanup.

### CI-FIX — PR #81 (ci/bench-timeout-cache-on-failure → develop) — MERGE FIRST OR FOLD
- bench timeout 20→60 min + `cache-on-failure: "true"` for all build-heavy CI jobs.
- bench PASSED (23m43s). snapshots FAILED (transient disk flake). linux-arm64 pending.
- Option A: `gh run rerun --failed <run-id>` → green → squash-merge.
- Option B: fold into STORY-092 delivery and close PR #81.

---

## WAVE 5 DELIVERY SUMMARY

**10 of 25 done (develop 15838de1, 79 PRs). +3 CI initiative stories (STORY-091/092/093) added 2026-06-10.**

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
- **CI-FIX OPEN** PR #81 (ci/bench-timeout-cache-on-failure → develop): bench timeout 20→60 + cache-on-failure. bench PASSED; snapshots/arm64 infra flakes. Merge or fold into STORY-092.

**15 stories remain (12 feature + 3 CI initiative). NEW ORDER: STORY-091/092/093 first, then STORY-081, then feature stories.**

**HELD (after CI initiative + STORY-081 land):**
- STORY-057/058/064 (slideforge-cli same-crate conflict — serialize after in-flight batch)
- STORY-056/048 (UNBLOCKED — ←047 MERGED)
- STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must resolve FIRST)

**OPEN FOLLOW-UPS:**
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

Phase 3, **Wave 5 IN PROGRESS** (develop `15838de1`, 79 merged PRs). 10 of 25 done. 15 stories remain (12 feature + 3 CI initiative STORY-091/092/093).

- **NEW ORDER (human re-sequenced 2026-06-10):** CI stabilization (STORY-091/092/093) FIRST → STORY-081 merge → remaining wave-5 feature stories.
- Active worktrees: 1 — STORY-081 in `.worktrees/STORY-081` on `feature/STORY-081` HEAD `5c665cd0`. PR #80 OPEN (PARKED — all reviews PASSED, waiting for CI stabilization). Open PRs: 2 (#80 parked, #81 to-merge/fold).
- Workspace: 4083 pass / 20 skip / 0 fail (FU-CI-ARM64-TEST-FAILURE CONFIRMED RESOLVED; cargo deny PASS).

---

## NEXT ACTIONS

**EXACT ORDERED TASK LIST (CI stabilization first — human-directed 2026-06-10):**

### Step 1 — PR #81 (optional quick win, do before or alongside STORY-092)

Merge PR #81 (`ci/bench-timeout-cache-on-failure`) OR fold into STORY-092 and close it:
- Option A: `gh run rerun --failed <run-id>` (from `gh pr checks 81`) → if green → squash-merge. If infra flakes persist after 2 reruns → admin-merge (STANDING AUTH; STORY-088 precedent).
- Option B: implementer delivers STORY-092 and folds the bench-timeout change in; close PR #81 as superseded.

### Step 2 — STORY-092 (HIGHEST-LEVERAGE CI fix — deliver first within CI batch)

Cache reliability + disk headroom. Stops the 9.77 GB/23-cache thrash. STORY-081 bench will pass once arm64 cache survives LRU. Dispatch devops-engineer + ci-workflow-analyzer. Spec: `.factory/stories/stories/STORY-092-ci-cache-reliability-disk.md`. Research: `.factory/planning/ci-speed-research.md`.

### Step 3 — STORY-091 (tiered triggers + merge queue)

Biggest per-PR wall-clock win (~6-8 min feedback loop). Spec: `.factory/stories/stories/STORY-091-ci-tiered-triggers-merge-queue.md`. Playbook: `.factory/playbooks/tiered-ci-merge-queue.md`. Verify at impl: enable merge queue on `develop` branch protection; `schedule:` nightly from `main` only.

### Step 4 — STORY-093 (arm64 build-time)

mold linker (arm64 only, `setup-mold` SHA `9c9c13bf...`) + `[profile.ci]` via `--cargo-profile ci`. Spec: `.factory/stories/stories/STORY-093-ci-arm64-build-time.md`.

### Step 5 — STORY-081 merge (AFTER CI stabilized)

All reviews already DONE. PR #80 OPEN, HEAD `5c665cd0`, PARKED.
1. Rebase `feature/STORY-081` onto stabilized develop: `git -C .worktrees/STORY-081 rebase origin/develop` → resolve mechanical conflicts → `git push --force-with-lease origin feature/STORY-081`.
2. Re-run PR #80 CI (bench now warm + 60-min budget; arm64 cache survives → passes).
3. STANDING MERGE AUTH → squash-merge PR #80 → develop.
4. Post-merge state burst: STORY-INDEX / dependency-graph / STATE.md → MERGED; record new develop SHA. LESSON-18: `git fetch && git merge --ff-only origin/develop` in any active worktrees.
5. Worktree cleanup: remove `.worktrees/STORY-081`; prune `feature/STORY-081`.

### Step 6 — Remaining Wave-5 feature stories

STORY-056/048 (UNBLOCKED), STORY-057/058/064 (cli serialized), STORY-060/061 (FU-SEC-001-GIT2-OPENSSL first).

**Diagnostic commands:** `gh pr checks 80` / `gh pr checks 81` / `gh run rerun --failed <run-id>`

**PER-STORY DELIVERY SEQUENCE (BC-5.39.001):**
adversary LOCAL 3-CLEAN (sequential) → demo-recorder per-AC → rebase onto develop `15838de1` → push → pr-manager 9-step (orchestrator dispatches security-reviewer + pr-reviewer per LESSON-5) → STANDING MERGE AUTH: CI-green + security CLEAN + pr-reviewer APPROVE → squash-merge → state-manager post-merge burst → worktree cleanup → LESSON-18 sync check.

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
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 89 stories, 21 epics, 6 waves, 553 pts (baseline). Now 93 stories / 571 pts after CI initiative +3. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 5: **10/25 done**. 15 remain (12 feature + 3 CI: STORY-091/092/093 NEXT per 2026-06-10 resequencing). STORY-081 PARKED (all reviews done; awaiting CI stabilization). | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. develop 15838de1 (79 merged PRs, 2 open PRs). RESEQUENCED: CI stabilization (STORY-091/092/093) FIRST, then STORY-081. STANDING MERGE AUTH active.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-10 |
| **develop SHA** | `15838de1` (79 merged PRs; origin/develop confirmed; 2 open PRs: #80 parked, #81 to-merge) |
| **Active worktrees** | 1 — STORY-081 in `.worktrees/STORY-081` on `feature/STORY-081` HEAD `5c665cd0`. **PARKED pending CI stabilization.** |
| **PR #81 state** | OPEN (ci/bench-timeout-cache-on-failure, MERGEABLE). bench PASSED. snapshots/arm64 infra flakes. Merge or fold into STORY-092. |
| **PR #80 / STORY-081 state** | OPEN (PARKED). HEAD `5c665cd0`. All reviews DONE (LOCAL 3/3, PR-level P31-P34 converged, security CLEAN, pr-reviewer APPROVE). Bench failed: cache thrash (STORY-092 fixes). |
| **Workspace tests** | 4083 pass / 20 skip / 0 fail (HEAD 5c665cd0; cargo deny PASS; FU-CI-ARM64-TEST-FAILURE CONFIRMED RESOLVED) |
| **Cache situation** | CONFIRMED ROOT CAUSE: 9.77 GB / 23 active caches / 97.7% of ~10 GB limit. arm64 LRU-evicted. STORY-092 is the structural fix. |
| **CI initiative** | STORY-091/092/093 (EPIC-19, NEXT). Specs written; uncertainty resolved; playbook + research sidecar ready. Deliver NOW (before STORY-081 merge). Consider STORY-092 first (highest-leverage). |
| **factory-artifacts** | Pushed to origin. Fresh sessions: clone + `git fetch origin factory-artifacts feature/STORY-081 ci/bench-timeout-cache-on-failure` + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | **STEP 1:** Handle PR #81 (merge or fold into STORY-092). **STEP 2:** Deliver STORY-092 (cache+disk). **STEP 3:** Deliver STORY-091 (tiered triggers). **STEP 4:** Deliver STORY-093 (arm64 build-time). **STEP 5:** Rebase STORY-081 onto stabilized develop → merge PR #80 → post-merge burst + worktree cleanup. **STEP 6:** Remaining wave-5 feature stories. STORY-081 PR #80 is PARKED — do NOT merge it before CI stories land. |

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
