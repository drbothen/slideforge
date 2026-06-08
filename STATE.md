---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-08
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
total_stories: 89
total_points: 553
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
wave_5_status: "5 of 22 Wave-5 stories MERGED (STORY-089 PR#68, STORY-046 PR#70, STORY-055 PR#71, STORY-079 PR#72, STORY-080 PR#73). 17 stories / 92 pts remain. In-flight: STORY-074 (pass 13/3-CLEAN, in-review PR#74), STORY-047 (adversary 1/3), STORY-072 (adversary 0/3 — gradient SVG linearGradient fix in progress), STORY-088 (adversary 0/3, rate-limited), STORY-081 (adversary pending), STORY-082 (full re-implementation)."
develop_sha: "e08f2f80"
develop_pr_count: 73
error_taxonomy_version: "v2.26"
workspace_tests: "~3690+ pass / 18 skip (develop e08f2f80); cold_budget flake PERMANENTLY FIXED (STORY-080 PR#73 squash-merged)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge | **Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop`. Canonical SHA: `e08f2f80` (73 merged PRs, 1 open PR: PR #74 STORY-074 in security+pr review).

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04). Fresh machines: `git clone <repo> && git worktree add .factory factory-artifacts`.

**Current position:** Phase 3, **Wave 5 IN PROGRESS**. STORY-080 MERGED PR #73 (e08f2f80). 5 of 22 done. 17 stories / 92 pts remain.

**Wave 5 — 5 of 22 done; 17 stories / 92 pts remaining (develop e08f2f80, 73 PRs):**

- **STORY-089 MERGED** PR #68 (c722c28b): field-value type validation. FieldSchemaValidator live. error-taxonomy v2.24. ADR-020. STORY-082 spec reconciled (BC-4.01.003 v1.3, export-arch v1.1).
- **STORY-046 MERGED** PR #70 (fa85d113): Static HTML exporter (slideforge-html crate). P4 Composite Rendering Model. BC-4.03.003 v1.4. STORY-047 + STORY-081 UNLOCKED (←046).
- **STORY-055 MERGED** PR #71 (cbebfd57): `slideforge build` CLI + miette diagnostics; unified compile pipeline. STORY-057/058/060/064 UNLOCKED (←055). STORY-056 UNLOCKED (←047+055).
- **STORY-079 MERGED** PR #72 (2f6d5da4): slideforge-diagrams SVG DoS hardening. SEC-001 nesting-depth guard (CWE-674, cap 64, iterative), SEC-002 byte-size guard (CWE-400, cap 50 MiB), error E-EXP-004. BC-1.12.003 v1.3, error-taxonomy v2.26. LOCAL adversary 3/3 strict-CLEAN (passes 3-4-5; 5 total). Security CLEAN, pr-reviewer APPROVE, CI green.
- **STORY-080 MERGED** PR #73 (e08f2f80): deflake cross-platform tests. Deterministic blocking-accept+mpsc 4xx harness (eliminates Windows TCP keep-alive flake); cold_budget restructured into always-on catastrophic NFR-003 gate (1s/2s, catches 10x EC-005) + #[ignore]'d precision test + always-on correctness companion. PERMANENTLY FIXES cold_budget CI flake. Error-taxonomy edge-case anchors corrected. LOCAL 3/3 strict-CLEAN (passes 5-6-7). Security CLEAN, pr-reviewer APPROVE, CI green.
- **DEP-PREP MERGED** PR #69 (3e3a978f): [workspace.dependencies] centralized + ADR-022 major-version migrations + INERT Wave-5 catalog. ADR-022 migration tasks DONE.
- **Uncertainty pass COMPLETE:** all stories spec-accurate. Ready to deliver.
- **In-flight:** STORY-074 (LOCAL 3/3 strict-CLEAN passes 11-13; 13 total; PR #74 OPEN in security+pr review; HEAD c84976b2), STORY-047 (adversary 1/3), STORY-072 (adversary 0/3 — HTML gradient SVG linearGradient fix in progress), STORY-088 (adversary 0/3, rate-limited retries), STORY-081 (adversary pending), STORY-082 (full re-implementation).
- **Dependency levels:** Full table in BACKLOG.md WAVE5-DELIVERY. **L0 remaining:** 082/088/072/074. Both unlock keys merged → L1 active: 047/081. Critical path: 047→056→059. **CAUTION:** resolve FU-SEC-001-GIT2-OPENSSL before STORY-060/061.
- **SEC-001-HARDENING (OPEN, non-blocking):** residual string-layer bypass vectors in ImagePathValidator. ANCHORED to image-loading story.
- **FU-SEC-001-GIT2-OPENSSL (OPEN, MEDIUM):** inert `git2` catalog entry with `features=["https"]` will pull `openssl-sys` (banned by deny.toml) when first consumed. MUST resolve before STORY-060/061. Anchored to STORY-060. See BACKLOG.md.

**Startup procedure:** (1) run `vsdd-factory:factory-worktree-health` (2) verify `develop == origin/develop` (3) confirm workspace tests green (`cargo nextest run --workspace --no-fail-fast` — expect ~3690+ pass, 18 skip; cold_budget STORY-080 converged, PR pending) (4) read `.factory/BACKLOG.md` and TaskCreate one task per OPEN item (5) read NEXT ACTIONS below (6) await human go-ahead before picking a story.

**Durable task source:** `.factory/BACKLOG.md` — rebuild in-session tasks from OPEN items there on every session start. State-manager mirrors it alongside STATE.md at every milestone.

**LESSON-18 (MANDATORY after every merge):** After `gh pr merge --squash`, run `git fetch && git merge --ff-only origin/develop` (or `git restore --source=HEAD --staged --worktree .`) to sync the working tree. `git update-ref refs/heads/develop origin/develop` alone moves the branch pointer but leaves the working tree STALE — gate agents will review stale files. Include a disk-presence check before dispatching any gate agent. Discovered: Wave 4 re-gate first attempt (2026-06-07). Lessons file: `.factory/cycles/wave-4-gate/lessons.md`.

**Phases 4–7 (holdout / adversarial / formal hardening / convergence) remain ahead before v1.0.**

---

## CURRENT POSITION

Phase 3, **Wave 5 IN PROGRESS** (develop `e08f2f80`, 73 merged PRs). STORY-089 PR #68; STORY-046 PR #70; STORY-055 PR #71; STORY-079 PR #72; STORY-080 PR #73; dep-prep PR #69. 5 of 22 done. 17 stories / 92 pts remain. 89 stories / 553 pts total.

- Active worktrees: 6 — STORY-047/072/074/081/082/088 in `.worktrees/` on `feature/STORY-<NNN>`. STORY-079/080 worktrees cleaned up post-merge. Open PRs: 1 (PR #74 STORY-074 in security+pr review).
- Workspace: ~3690+ pass, 18 skip (cold_budget PERMANENTLY FIXED by STORY-080 PR#73).
- Uncertainty pass: COMPLETE (all stories spec-accurate).
- ADR-022 dep-centralization migrations: DONE. ADR-008 P4 amendment: DONE. ADR-021 async runtime: DONE (wired in STORY-055 otel feature).

---

## NEXT ACTIONS

**STATUS: Wave 5, STORY-080 merged (PR #73, develop e08f2f80). 5 of 22 done. 6 worktrees active.**

**RESUME PROCEDURE (zero context):** (1) run `vsdd-factory:factory-worktree-health`; (2) verify `git rev-parse develop` == origin/develop == `e08f2f80`; (3) confirm workspace tests (`cargo nextest run --workspace` — expect green; cold_budget PERMANENTLY FIXED by STORY-080 PR#73); (4) read BACKLOG.md WAVE5-DELIVERY for in-flight status; (5) CONTINUE per-story delivery for active worktrees (STORY-074 PR#74 in review, STORY-047/088/072/082/081 in varying adversary stages).

**IN-FLIGHT (6 active worktrees):** STORY-047 (adversary 1/3), STORY-072 (adversary 0/3 — gradient SVG fix), STORY-074 (PR #74 in security+pr review; HEAD c84976b2, based on 2f6d5da4), STORY-081 (adversary pending), STORY-082 (full re-implementation), STORY-088 (adversary 0/3, rate-limited). **HELD:** STORY-057/058/064 (slideforge-cli same-crate conflict — deliver after this batch); STORY-056 (←047 not yet merged); STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must be resolved first).

**CRITICAL — APPLY LESSON-21 to EVERY story's exit gate** (this prevented 4 CI-fix cycles on STORY-055): the implementer/pre-push gate MUST run, in addition to nextest: `cargo test --workspace --all-features` (SHARED-PROCESS — catches global-state/test-isolation bugs nextest masks), `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` (catches cross-crate intra-doc links), and tests touching filesystem/global-state MUST be cross-platform + isolation-robust (no `/nonexistent` Unix-root paths — use a blocker-file parent; no virgin-global-subscriber assumptions; Windows can't run on the dev mac, so expect the Windows CI job as validator). APPLY LESSON-22: verify any implementer "upstream gap" claim + that any cited story ID exists before routing a deferral.

**STANDING MERGE AUTH applies:** squash-merge any PR that is CI-green (all-checks-pass) + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking.

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
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 89 stories, 21 epics, 6 waves, 553 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 4: 23/23 COMPLETE + follow-ups CLOSED. Wave 5: IN PROGRESS — 5/22 done (STORY-089 PR #68, STORY-046 PR #70, STORY-055 PR #71, STORY-079 PR #72, STORY-080 PR #73); 17 stories/92 pts remain. Both unlock keys (046+055) merged → L1 unlocked. | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

## Wave 4 Story Status (summary)

- **Batch A (10/10 MERGED):** PRs #39-#50 — STORY-035/036/043/044/073/075/076/078/045/077 (+ STORY-077 follow-up PR #50).
- **Batch B (6/6 MERGED):** PRs #51-#56 — STORY-041/042/037/038/039/040; pptx chain + docx chain complete.
- **Batch C (5/5 MERGED):** PR #57 STORY-083, PR #58 STORY-084, PR #59 STORY-085, PR #60 STORY-049, PR #61 STORY-050.
- **Remediation (3 MERGED):** PR #62 STORY-086 (content threading + TextTag), PR #63 STORY-087 (color-coded slide types), PR #64 image-alt fix (NEW-INT-001).
- **Wave 4 = 23/23 COMPLETE. Re-gate FULLY PASSED on develop 02d484cf.**

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. develop e08f2f80 (73 merged PRs). STORY-080 merged. 6 worktrees active. 17 stories / 92 pts remain.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-08 |
| **Position** | Wave 5 IN PROGRESS. 5 stories merged (STORY-089 PR #68, STORY-046 PR #70, STORY-055 PR #71, STORY-079 PR #72, STORY-080 PR #73). Active: STORY-074 (PR #74 OPEN in security+pr review; 3/3 strict-CLEAN 13-pass), STORY-047 (adversary 1/3), STORY-072 (adversary 0/3 — HTML gradient SVG fix), STORY-088 (adversary 0/3, rate-limited), STORY-081 (adversary pending), STORY-082 (full re-implementation). 17 stories / 92 pts remain. |
| **develop SHA** | `e08f2f80` (73 merged PRs; origin/develop confirmed; 1 open PR: #74 STORY-074) |
| **Active worktrees** | 6 — STORY-047/072/074/081/082/088, each in `.worktrees/STORY-<NNN>` on `feature/STORY-<NNN>`. STORY-079/080 cleaned up post-merge. |
| **Merge model** | Parallel development → SERIAL merge w/ rebase + re-gate. Serialize 072+074 (shared types+layout). Crate overlaps: layout (082/088/072/074), pdf (072/081), types (072/074), syntax (082/088), eval (088/081), html (047/081). |
| **Workspace tests** | ~3690+ pass / 18 skip (cold_budget PERMANENTLY FIXED by STORY-080 PR#73) |
| **factory-artifacts** | PUSHED to origin. Fresh sessions: clone + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | 6 worktrees + branches in varying stages. STORY-074: PR #74 in review (HEAD c84976b2, based on 2f6d5da4) — await security+pr-reviewer APPROVE then STANDING MERGE AUTH applies. Others: check each branch's commits to see which stage reached. Per-story: implementer TDD → LOCAL adversary 3-CLEAN (BC-5.39.001) → demo-recorder → pr-manager 9-step → squash-merge → state-manager post-merge burst → worktree cleanup. Apply LESSON-21 exit gate to EVERY story. HELD (next batch after this batch merges): STORY-057/058/064 (cli same-crate conflict). BLOCKED until FU-SEC-001-GIT2-OPENSSL resolved: STORY-060/061. HELD (←047 unmerged): STORY-056. Dep-graph L0–L3 in BACKLOG.md WAVE5-DELIVERY. Phases 4-7 remain for v1.0. |

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
| LESSON-19 (SIBLING-SWEEP) | When a fix-burst changes a canonical VALUE, TEXT, COUNT, or ANCHOR (error code, message template, site count, ADR/BC/PC citation, stale-state comment), it MUST sweep ALL sibling artifacts in ONE burst: production code + every consuming spec (BC/ADR/taxonomy/story) + test files + doc-comments + fixtures — verified by exhaustive grep BEFORE re-running the adversary. Partial propagation repeatedly reset STORY-089 3-CLEAN streak (~11 passes). Orchestrator must grep-map every occurrence itself and dispatch ONE coordinated exhaustive fix, not per-file partial fixes. Full narrative: `.factory/cycles/STORY-089/lessons.md`. REINFORCED by STORY-046 cascade (passes 14/17/20 all partial-propagation recurrences). |
| LESSON-20 (ADVERSARY-SPEC-PATHS) | Per-story LOCAL adversary dispatches MUST include ABSOLUTE `.factory/` spec paths (story file, traced BCs, traced ADRs, export-architecture or equivalent spec) in the task. The per-story worktree (`.worktrees/STORY-NNN`) does NOT contain the `.factory/` mount — adversary cannot read specs without absolute paths. Discovered: STORY-046 Pass-4 OBS-1. Full narrative: `.factory/cycles/STORY-046/convergence-summary.md`. |
| LESSON-21 (LOCAL-GATE-MIRRORS-CI-MATRIX) | The per-story LOCAL exit gate and the pre-push gate MUST mirror CI's FULL invocation matrix, not a narrower subset. STORY-055 incurred FOUR post-convergence CI-fix cycles, ALL because the local gate used `cargo nextest` (process-isolated) on macOS only with `-p`-targeted doc, while CI runs: (a) `cargo insta test`/`cargo test` SHARED-PROCESS (caught a global-tracing-subscriber test-isolation panic nextest masked); (b) `--all-features` (caught the otel no-Tokio-runtime panic); (c) `RUSTDOCFLAGS="-D warnings" cargo doc --workspace` (caught a cross-crate intra-doc link); (d) the Windows test matrix (caught Unix-only failure-injection in filesystem tests). Going forward, implementer exit-gate + pre-push instructions MUST include: `cargo test --workspace --all-features` (shared-process) in addition to nextest; `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`; and tests touching filesystem/global-state must use cross-platform/process-isolation-robust techniques (no `/nonexistent` Unix-root paths; no virgin-global-state assumptions). Windows can't run on the dev mac — expect the Windows CI job as the validator and write portable tests. Discovered: STORY-055 post-convergence CI-fix cycles B1-B4. |
| LESSON-22 (ORCHESTRATOR-VERIFY-UPSTREAM-CLAIMS) | Before routing a deferral/"not supported" based on an implementer's claim of an upstream gap, VERIFY the claim against the actual upstream code, and confirm any cited story ID EXISTS. STORY-055 initially shipped a false "variant selection not supported (STORY-091)" — STORY-091 was fabricated and `eval_deck_with_variant` WAS implemented; the adversary caught it. Reinforces SID-1 real-story-citation rule. Implementer self-disclosure of capability gaps is NOT authoritative — orchestrator independently verifies before accepting a deferral. Discovered: STORY-055 Pass-1 C-1 finding. |

---

## Blocking Issues

| ID | Description | Status |
|----|-------------|--------|
| BLK-001 | Alt-text enforcement non-functional end-to-end. | RESOLVED — STORY-050 ADR-018 post-layout validation. |
| BLK-002 | Wave 4 gate (content threading + color-coded types + image-alt fix). | CLOSED 2026-06-07 — Gate 5 mean 1.00, min_critical 1.00. |

---

## Decisions Log

_Entries before STORY-050-MERGE archived to `.factory/cycles/wave-4-gate/decisions-archive.md`._

| Date | ID | Decision |
|------|-----|---------|
| 2026-06-08 | STORY-080-MERGE | PR #73 squash-merged → develop `e08f2f80` (73 merged PRs). Deflake cross-platform tests: deterministic blocking-accept+mpsc 4xx harness (eliminates Windows TCP keep-alive flake); cold_budget restructured into always-on catastrophic NFR-003 gate (1s/2s, catches 10x EC-005) + #[ignore]'d precision test + always-on correctness companion. PERMANENTLY FIXES cold_budget CI flake (no more re-runs needed on future PRs). Spec touches: error-taxonomy edge-case anchors corrected. LOCAL 3/3 strict-CLEAN (passes 5-6-7). Security CLEAN, pr-reviewer APPROVE, CI green. STORY-074 LOCAL CONVERGED (3/3 strict-CLEAN passes 11-13; 13 total passes — long doc-precision tail, code converged since pass 3); PR #74 OPEN (https://github.com/drbothen/slideforge/pull/74, HEAD c84976b2, based on 2f6d5da4), in security+pr review. |
| 2026-06-08 | STORY-079-MERGE | PR #72 squash-merged → develop `2f6d5da4` (72 merged PRs). slideforge-diagrams SVG DoS hardening: SEC-001 nesting-depth guard (CWE-674, iterative DFS cap 64), SEC-002 byte-size guard (CWE-400, cap 50 MiB), E-EXP-004 added. Specs evolved during convergence: BC-1.12.003 v1.3 (inv-4 ordering correction, PC-8/9/10, EC-005..008); error-taxonomy v2.26 (E-EXP-004 sibling-accuracy); export-architecture v1.4 (root-at-1 iterative). LOCAL adversary 3/3 strict-CLEAN (passes 3-4-5; 5 total). Security CLEAN, pr-reviewer APPROVE, CI green (cold_budget flake cleared on re-run). Follow-ups anchored Phase 6: FU-079-SEC-001 (compile-time exhaustiveness on usvg Node variants), FU-079-SEC-LOW (slide_title truncation + shared constants + distinct sentinel spans), FU-079-DOC-NITS (normalize.rs AC-003 doc + evidence-report edge-case IDs). See BACKLOG.md. |
| 2026-06-08 | WAVE5-FULLWIDTH-LAUNCH | Human confirmed "full 8 parallel now". Orchestrator created 8 per-story worktrees off develop cbebfd57 and launched Stage-1 (test-writer Red Gate: compilable stubs + failing tests) for all 8 stories in parallel. Active worktrees + branches: STORY-082 (feature/STORY-082), STORY-088, STORY-072, STORY-074, STORY-079, STORY-080, STORY-047, STORY-081 — each in `.worktrees/STORY-<NNN>` on branch `feature/STORY-<NNN>`. Merge model: parallel development, SERIAL merge with rebase + re-gate. Known crate overlaps to manage at merge time: slideforge-layout (082/088/072/074), slideforge-pdf (072/079/081), slideforge-types (072/074), slideforge-syntax (082/088), slideforge-eval (088/081), slideforge-diagrams (079/080), slideforge-html (047/081). Serialize 072 and 074 merges (shared types+layout). Per-story flow per BC-5.39.001 + LESSON-21 exit gate applies to each. RESUME NOTE: if session dies mid-flight, 8 worktrees + feature branches exist at base cbebfd57; check each branch's commits to see which stage each story reached. |
| 2026-06-08 | WAVE5-FULLWIDTH-FANOUT-AUTH | Human authorized FULL-WIDTH fan-out (up to 8 parallel) and selected the next delivery batch. APPROVED NEXT BATCH (8 stories, launch in parallel): STORY-082, STORY-088, STORY-072, STORY-074, STORY-079, STORY-080, STORY-047, STORY-081. RATIONALE: 6 ready L0 (082/088/072/074/079/080) + the two 046-unlocked L1 (047,081). STORY-080 included to fix the recurring cold_budget timing flake (re-run-cleared on every PR this session). HELD (do NOT launch in this batch): STORY-057/058/064 — all extend slideforge-cli and would collide in parallel worktrees; deliver them AFTER this batch (or serialized) to avoid same-crate merge conflicts. STORY-056 — depends on STORY-047 (not yet merged). STORY-060/061 — blocked by FU-SEC-001-GIT2-OPENSSL (must resolve git2 https/openssl-sys before launching). NOTE within-batch minor same-crate risk: STORY-072+074 are both EPIC-07 (verify crate before parallelizing those two). |
| 2026-06-08 | STORY-055-MERGE | PR #71 squash-merged → develop cbebfd57. slideforge build CLI command + miette diagnostics; UNIFIED compile pipeline (compile_core single source; build_inner = compile_core + export_format; new public API compile()/export_format()/CompileOptions/CompiledDeck; BuildError::MultistageFailed for cross-stage eval+validator accumulation). 7-pass LOCAL 3-CLEAN + 4 post-convergence CI-fix cycles (B1 cross-crate intra-doc link; B2 otel rt-tokio needed a Tokio runtime — wired TracingGuard owning a runtime gated on `otel` feature, ADR-021; snapshots-job shared-process test-isolation — init_tracing already-set→Ok(noop) + moved traced_test to own binary; Windows cross-platform export-error test injection — portable blocker-file technique). Security CLEAN (2 LOW follow-ups SEC-001 tmp-file symlink, SEC-002 otel endpoint → Phase 6). pr-reviewer APPROVE. adversary 3-CLEAN. CI all-green incl windows+macos+snapshots+docs+bench. Convergence: `.factory/cycles/STORY-055/convergence-summary.md`. |
| 2026-06-08 | STORY-055-LOCAL-CONVERGED | STORY-055 (CLI: `slideforge build` + miette error rendering, EPIC-15, slideforge-cli) LOCAL adversarial cascade CONVERGED: 3/3 strict-CLEAN (passes 5, 6, 7); 7 total passes. Feature branch `feature/STORY-055` HEAD `fcdd1c55`, rebased onto develop `fa85d1136`. NOT YET MERGED — demo + PR pending. Key findings: CRIT-HTML (resolved by rebasing onto fa85d1136 — HtmlExporter now registered, 4-format default restored); C-1 (fabricated "STORY-091" deferral — adversary caught that `eval_deck_with_variant` IS implemented; `--variant` threaded through `CompileOptions`); HIGH-001 (validator span threading); HIGH-002 (multi-format duplicate diagnostics — unified pipeline); HIGH-003 (`--json total:1` hardcoded); HIGH-004 (`exit_code_to_u8` silent fallback); F-P2-MED-001 (non-fatal eval errors swallowed — `MultistageFailed` variant); MED-001 (all-or-nothing output atomicity); HIGH-P3-001/MED-P3-002 (source-order interleave — `diag_util` sort/dedup helper); OBS-P4-001..004 (eval dedup symmetry, JSON severity fidelity, pure `render_build_error_to_json_value`); LOW-001 (watch/init/extract-brand graceful not-implemented citing STORY-056/057). STRUCTURAL IMPROVEMENT: unified pipeline — `compile_core`/`compile_inner` is single canonical pipeline (parse→eval→brand→validate→layout); `build()`/`build_inner()` is thin adapter = `compile_core` + `export_format`. New public API: `compile()`, `export_format()`, `CompileOptions`, `CompiledDeck`. PROCESS LESSON (S-7.02 candidate, [process-gap]): orchestrator MUST independently verify implementer upstream-gap claims before accepting a deferral — fabricated "STORY-091" caught only by adversary absolute-path grep. Implementer self-disclosure of capability gaps is NOT authoritative. Open follow-ups (non-blocking): FU-055-OFFLINE-FLAG (--offline flag not threaded; anchor STORY-021), FU-055-VARIANT-NOTE (traceability), FU-055-JSON-HASFATAL-DOC (non-binding prose). Full detail: `.factory/cycles/STORY-055/convergence-summary.md`. |
| 2026-06-08 | STORY-046-MERGE | PR #70 squash-merged → develop fa85d1136. Static HTML exporter (slideforge-html crate) via P4 Composite Rendering Model. 23-pass LOCAL adversarial convergence incl. rendering-model pivot: ADR-008 P4 amendment (slide text as positioned HTML; graphical frames in sibling `<svg role="presentation">`; Pass-3 ARIA correction — outer svg NOT aria-hidden), BC-4.03.003 v1.4 (Inv2, PC-6, PC-7 4-step heading chain + synthetic visually-hidden h1), STORY-046 v1.3. HtmlExporter REGISTERED in root registry (human-authorized HTML-seam decision). Fixed CI axe-core gate module-resolution bug. Security CLEAN (2 LOW follow-ups → STORY-047/048: SEC-001 data: scheme in SVG href, SEC-002 npm pin). pr-reviewer APPROVE. CI all-green (cold_budget flake re-run passed). Convergence detail: `.factory/cycles/STORY-046/convergence-summary.md`. |
| 2026-06-08 | STORY-046-LOCAL-CONVERGED | STORY-046 (Static HTML Exporter, P4 Composite Rendering Model) LOCAL adversarial cascade CONVERGED: 3/3 strict-CLEAN (passes 21, 22, 23); 23 total passes. Feature branch `feature/STORY-046` HEAD `93cef9fb`. Key resolutions: ADR-008 amended (P4 model); BC-4.03.003 → v1.4; STORY-046 → v1.3; export-architecture updated; HtmlExporter registered in registry.rs (human-authorized HTML-seam decision 2026-06-08). LESSON-20 added (adversary spec paths). See `.factory/cycles/STORY-046/convergence-summary.md`. |
| 2026-06-08 | WAVE5-DEP-PREP-MERGE | PR #69 squash-merged → develop 3e3a978f (69 merged PRs). Wave-5 dependency prep: [workspace.dependencies] centralized (ADR-022). Major-version migrations DONE: toml 0.8.23→1.1.2 (brand/data/math), sha2 0.10.9→0.11.0 (pptx/math; hybrid_array→[u8;32] boundary), criterion 0.5→0.8.2 (std::hint::black_box), notify 6.1.1→8.2.0, indexmap 2.10→2.14. INERT Wave-5 catalog entries added (tokio 1.52.3, reqwest 0.13.4, axum 0.8.9, crossterm 0.29.0, otel 0.32.0/otlp/sdk, tracing-opentelemetry 0.33.0, git2 0.21.0, tar 0.4.46, flate2 1.1.9, dirs 6.0.0, globset 0.4.18, hex 0.4.3, toml_edit 0.25.12, notify-debouncer-full 0.7.0). Security CLEAN; pr-reviewer APPROVE; CI green (cold_budget flake re-run passed). ADR-022 merged-crate migration tasks DONE. FU-SEC-001-GIT2-OPENSSL identified (NEW, MEDIUM): inert git2 https feature will pull openssl-sys (banned) when consumed — resolve before STORY-060/061. |
| 2026-06-08 | WAVE5-UNCERTAINTY-COMPLETE | Wave-5 remove-uncertainty pass COMPLETE. All 21 Wave-5 stories spec-accurate. factory-artifacts commits: 7df72268 (STORY-082 recon: p14 sectionLst + E-PAR-023/W-PAR-002 + BC-4.01.003 v1.3), 277fc481 (ADR-021/022 + export-arch v1.2 + dep-graph v1.1 + NFR-002 deferral), b353613a (17 stories realigned + NFR-002 sweep across 5 spec files). STORY-074/088/082 needed no version changes. Wave-5 delivery may begin at full-width fan-out (L0: up to 8 parallel). Dep-graph levels + parallelism plan recorded in BACKLOG.md WAVE5-DELIVERY. |
| 2026-06-07 | WAVE5-UNCERTAINTY-S2 | Wave-5 remove-uncertainty pass Stage 2 COMPLETE (story propagation + NFR-002 sweep). 17 stories realigned to registry-verified pins + ADR-021/022 architecture: async stack (tokio 1.52.3/reqwest 0.13.4-rustls/axum 0.8.9 built-in ws/crossterm 0.29/otel 0.32-0.33) into STORY-047/055/056; notify 8.2.0+debouncer-full 0.7.0 (048/056); sha2 0.11 + toml 1.1.2 + toml_edit 0.25.12 + git2 0.21(no-shallow) + tar/flate2 reproducible + dirs 6.0 + globset (060-065); krilla gradients not pdf-writer + ooxmlsdk typed builders (072/081); usvg preserves-nesting design confirmed + stale-pin fixes (079); usvg strips-aria→string-wrap + scraper Selector + minijinja 2.20 (046); criterion 0.8.2 std::hint::black_box + critcmp 0.1.8 + AC-003 incremental deferred (059); NO_COLOR is_some_and fix (058); HashMap→BTreeMap determinism (065); Vec::dedup sort fix (064). Workspace-pin conflicts resolved to {workspace=true}. NFR-002 <50ms incremental gate deferred to v1.x across system-overview/cicd-setup/ux-spec(FLOW-003,SCR-002,SCR-008,UX-INDEX)/epics/wave-schedule. STORY-074/088/082 needed no changes. |
| 2026-06-07 | WAVE5-UNCERTAINTY-S1 | Wave-5 remove-uncertainty pass Stage 1 (foundation). Scanned all 21 Wave-5 stories; research-validated all version/API uncertainties (registry-verified). Human decisions: introduce async runtime (tokio), bump notify→8.x, adopt sha2 0.11 + toml 1.x majors, defer NFR-002 <50ms incremental gate to v1.x. ADR-021 (async runtime: tokio 1.52.3, reqwest 0.13.4 rustls, axum 0.8.9 built-in ws, crossterm 0.29, otel 0.32/0.33). ADR-022 (workspace dep centralization + migration: sha2 0.11 in pptx, toml 1.1.2 in data/brand/math, notify 8.2.0 at root; git2 0.21 no-shallow; reproducible tar.gz; globset over glob). export-arch v1.2 (krilla gradients not pdf-writer; usvg strips aria→string-wrap; usvg preserves <g> nesting; ooxmlsdk typed builders OK; quick-xml auto-escapes). NFR-002 deferred (nfr-catalog v1.3, prd v1.1, BC-3.06.002 v1.3). Story propagation to follow in Stages 2-4. |
| 2026-06-07 | STORY-082-SPEC-RECON | Spec-first reconciliation before STORY-082 implementation. BC-4.01.003 v1.3 (EC-010/EC-011, PC 7/8, Inv 5). error-taxonomy v2.24 (E-PAR-023 empty section name; W-PAR-002 duplicate name). export-architecture v1.1 (authoritative p14 ext sectionLst structure — bare p:sectionLst was wrong; ooxmlsdk can't emit p14 → raw quick-xml injection; sha2+quick-xml promoted to prod deps). STORY-082 body realigned (AC-010/AC-011 added). |
| 2026-06-07 | STORY-089-MERGE | PR #68 merged → develop c722c28b. Field-value type validation: FieldType enum + type_matches() + E-VAL-104 (T1 type-mismatch, T2 OneOf violation); FieldSchemaValidator wired into Stage-5 (closes validate_fields dead-letter — latent bug; ADR-020 Decision 8); 8 Priority-1 annotations; chart.data optional; BC-1.18.001 v1.5; BC-1.17.002 v1.3 (ValueRangeValidator range-only); error-taxonomy v2.23 (E-VAL-104 formal + E-VAL-101/102/W-VAL-103). LOCAL 3-CLEAN converged (~11 passes — sibling-artifact propagation gaps drove cascade). Security CLEAN (2 LOW). pr-reviewer APPROVE. CI green. |
| 2026-06-07 | WAVE-5-START | Human authorized Wave 5 start. (d) folded as STORY-089 (BC-1.18.001, ADR-020, E-VAL-104, error-taxonomy v2.20). Wave-4 follow-up fix-burst cycle CLOSED. factory-artifacts advanced: def8bb74 (PO BC/taxonomy) → 32fddb9a (ADR-020) → 9f01b8bf (STORY-089 + indexes). Wave 5 = 22 stories / 130 pts. STORY-089 delivery starting. |
| 2026-06-07 | PR-C-MERGE | PR #67 merged → develop df207b84. OBS-P6-001 (status slide RegionRole::Title frame) + OBS-P6-002 (canonical ColorBar percent:u8 in FrameContent; DOCX reads directly — fixes double-floor off-by-one) CLOSED. Both confirmed real defects. +5 tests (3 status-title + 2 docx-percent). Security CLEAN, pr-reviewer APPROVE, CI green. Wave-4 follow-up fix-bursts 5/6 done; (d) feature story remains. |
| 2026-06-07 | PR-A-MERGE | PR #66 merged → develop 23f09c62. SEC-001 (E-VAL-012, error-taxonomy v2.19, BC-1.16.001 EC-012) + diag-span CLOSED. +15 tests (11 ImagePathValidator + 4 diag-span). Security CLEAN (1 defense-in-depth suggestion anchored to image-I/O story as SEC-001-HARDENING). pr-reviewer APPROVE; CI green. |
| 2026-06-07 | WAVE4-FOLLOWUP-DISPOSITION | Human dispositioned all 6 Wave-4 follow-ups to fix NOW pre-Wave-5. (c) SEC-002 CLOSED PR #65 (0d0113a2): split a11y arm, AltText::Unspecified → tracing::warn!, Security CLEAN + pr-reviewer APPROVE. (d) validate_fields reframed: complete for current schema; type/enum/format checks need E-VAL-104 (new BC) + FieldDef extension = authorized feature story (PO/architect/story-writer). (a) diag-span + (b) SEC-001 CWE-22: IN PROGRESS PR-A. (e) OBS-P6-001 + (f) OBS-P6-002: IN PROGRESS PR-C. develop 0d0113a2. |
| 2026-06-07 | WAVE4-COMPLETE | Wave 4 COMPLETE. image-alt fix (PR #64) squash-merged -> develop 02d484cf (64 PRs; LESSON-12+18 applied). Re-gate FULLY PASSED: Gate 1 PASS; Gate 2 SKIP; Gate 3 PASS (all 4 original findings closed; NEW-INT-001 image-alt resolved — image.rs/known_fields.rs/field_to_block.rs aligned; validate_fields wired); Gate 5 PASS (mean 1.00; min_critical 1.00; trajectory 0.56->0.86->1.00; all 5 critical scenarios pass). BLK-002 CLOSED. Wave 5 (STORY-082/081/088) PENDING human go-ahead. |
| 2026-06-07 | WAVE4-REGATE-V2 | Wave 4 re-gate on correct tree (develop 54b8d3b1; LESSON-WORKTREE-SYNC applied — first attempt reviewed stale tree). Gate 3 PASS (all 4 original findings closed; 1 MED NEW-INT-001 image-alt remaining). Gate 5 FAIL (mean 0.860 PASS; min_critical 0.500 FAIL). BLK-002 remained open pending image-alt fix. |
| 2026-06-06 | STORY-087-MERGED | STORY-087 MERGED PR #63 (54b8d3b1). Color-coded slide types (status/progress_bar/weighted_composite). CI all-green. SEC-100 MED XML-control-char-strip fixed in-scope. pr-reviewer APPROVE. 3/3 strict-CLEAN (10 passes). Wave 4 = 23/23. All 4 original Gate-3 findings closed. |
| 2026-06-06 | STORY-086-MERGED | STORY-086 MERGED PR #62 (298ae518). Stage 2b field-to-block content threading + TextTag. CI 22 checks all-green. Security APPROVE/CLEAN (SEC-001/002/003/004 as open follow-ups). pr-reviewer APPROVE. 3/3 strict-CLEAN (16 passes). |
| 2026-06-06 | WAVE4-REMEDIATION-SETUP | Wave 4 gate FAILED — remediation scoped + human-approved. ADR-019 (Stage 2b threading) authored. STORY-086 + STORY-087 added as Wave 4 pull-ins. |
| 2026-06-05 | STORY-050-MERGE | STORY-050 MERGED PR #61 (030dec6c). E2E integration suite + PDF/a11y/observability pipeline fixes. ADR-018 v1.1; BC-5.02.001 v1.5; error-taxonomy v2.15. DRIFT-CRITICAL-1 + DRIFT-CRITICAL-2 RESOLVED. 5-pass LOCAL cascade, 3/3 strict-CLEAN (passes 3-4-5). |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
