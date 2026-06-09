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
total_stories: 90
total_points: 556
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
wave_5_status: "8 of 22 Wave-5 stories MERGED (STORY-089 PR#68, STORY-046 PR#70, STORY-055 PR#71, STORY-079 PR#72, STORY-080 PR#73, STORY-074 PR#74, STORY-047 PR#75, STORY-072 PR#76). 14 stories remain. In-flight (3 active worktrees): STORY-082 (adversary 0/3 — pass 9 next against spec v1.2/BC-4.01.003 v1.4/v2.28; HEAD 67530b17), STORY-088 (adversary 0/3 — pass 4 pending), STORY-081 (adversary 0/3 — pass 2 pending after full re-impl)."
develop_sha: "2667987e"
develop_pr_count: 76
error_taxonomy_version: "v2.28"
workspace_tests: "~3807+ pass / 21 skip (develop 2667987e)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge | **Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop`. Canonical SHA: `2667987e` (76 merged PRs, 0 open PRs).

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04; ongoing pushes authorized).

**Current position:** Phase 3, **Wave 5 IN PROGRESS**. STORY-072 MERGED PR #76 (2667987e). 8 of 22 done. 14 stories remain.

---

## IN-FLIGHT WORKTREES — EXACT RESUME STATE

**3 active worktrees. ALL based on cbebfd57 and MUST rebase onto develop 2667987e at PR step.**

### STORY-082 — PPTX Slide-Grouping Sections (EPIC-08, BC-4.01.003)
- **Worktree:** `.worktrees/STORY-082` | **Branch:** `feature/STORY-082` | **HEAD:** `67530b17` (Pass-4 fix-burst applied)
- **Adversary streak:** **0/3** (reset by Pass-8 F-P8-MED-1 spec drift — see Decisions Log STORY-082-F-P8-MED1)
- **History:** Pass-1 found feature NON-FUNCTIONAL end-to-end (dead wiring + silent slide-drop) → full re-implementation (genuine parse→eval→layout→export wiring + keystone e2e). Pass-2 found CRIT-A (section slide-IDs diverged from Deck.slides under @for/@if) → fixed via single-pass section membership. Pass-3 found HIGH-1 (dead "rescue" paper-fix contradicting error-taxonomy "E-PAR always fatal") → removed; message templates + AC-008 full-pipeline test + CRIT-A disjoint+complete assertions strengthened. Pass-4 fix-burst applied (HEAD 67530b17). Pass-5 IMP-1 (HIGH) was a spec defect — E-PAR-023 exit 2→1 corrected in BC-4.01.003 v1.4 + error-taxonomy v2.28 + STORY-082 spec_version 1.1; worktree code UNCHANGED. Passes 6/7 (presumed clean per context). Pass-8 found F-P8-MED-1 (MEDIUM, spec-text drift) — sha2 pin =0.10.9→=0.11.0 in 4 spec locations; spec_version 1.1→1.2; code UNCHANGED (67530b17).
- **NEXT ACTION:** Run adversary **Pass 9** against corrected spec (STORY-082 spec_version 1.2, BC-4.01.003 v1.4 + error-taxonomy v2.28) → need 3 consecutive strict-CLEAN for convergence.

### STORY-088 — Bullets List-Literal DSL (EPIC-02, BC-1.01.002)
- **Worktree:** `.worktrees/STORY-088` | **Branch:** `feature/STORY-088` | **HEAD:** `3b0c6295`
- **Adversary streak:** **0/3**
- **History:** Pass-1 CRIT E-PAR-015 mis-anchor→E-PAR-024 (new code, taxonomy v2.27, dsl-spec v1.1). Pass-2 `<type>` interpolation + FieldValue::Error substitution. Pass-3 MED-P3-001 nested-list silent-drop→errors + MED-P3-002 Float test. HAS a genuine end-to-end test (real .sf → slideforge::build() → assert `<a:r>` + bullet text in slide XML). 792 tests pass.
- **NEXT ACTION:** Run adversary **Pass 4** → need 3 consecutive strict-CLEAN for convergence.

### STORY-081 — Slide-Level Inline Markup (EPIC-18, BC-3.02.002, 13 pts)
- **Worktree:** `.worktrees/STORY-081` | **Branch:** `feature/STORY-081` | **HEAD:** `c11d6468`
- **Adversary streak:** **0/3**
- **History:** Pass-1 found 4 CRIT dead-wiring (dead PDF path, body/subtitle markup dropped at threading seam, PPTX Body flattens, no e2e test) → FULL re-implementation: eval threads FieldValue::Inlines; PPTX inline runs + Code/Super/Sub/Highlight; DOCX highlight + rich title; PDF ActualText via slide_to_krilla_runs; keystone e2e test (build() → b="1"/`<w:b/>`/`<strong>`/PDF ActualText, no literal `**`). 1190 tests pass.
- **NEXT ACTION:** Run adversary **Pass 2** (re-review the full rebuild).
- **OPEN RISK for adversary:** HTML/PDF *title*-inline rendering was DEFERRED as a follow-on (DOCX rich title done; AC-006(3) requires all 3 — HTML + PDF title inline). Adversary must judge if partial AC-006(3) is acceptable or needs a fix before convergence.

---

## WAVE 5 DELIVERY SUMMARY

**8 of 22 done (develop 2667987e, 76 PRs):**

- **STORY-089 MERGED** PR #68 (c722c28b): field-value type validation. FieldSchemaValidator live. error-taxonomy v2.24. ADR-020.
- **STORY-046 MERGED** PR #70 (fa85d113): Static HTML exporter (slideforge-html crate). P4 Composite Rendering Model. BC-4.03.003 v1.4. STORY-047 + STORY-081 UNLOCKED.
- **STORY-055 MERGED** PR #71 (cbebfd57): `slideforge build` CLI + miette diagnostics. STORY-057/058/060/064 UNLOCKED. STORY-056 UNLOCKED (←047+055 both now merged).
- **STORY-079 MERGED** PR #72 (2f6d5da4): slideforge-diagrams SVG DoS hardening. SEC-001+SEC-002 guards. BC-1.12.003 v1.3. error-taxonomy v2.26. LOCAL 3/3 (passes 3-4-5).
- **STORY-080 MERGED** PR #73 (e08f2f80): deflake cross-platform tests. cold_budget PERMANENTLY FIXED. LOCAL 3/3 (passes 5-6-7).
- **STORY-074 MERGED** PR #74 (3f7f99ed): brand-aware em sizing. `font_size_emu` (i64). DEFAULT_EM_IN_EMU removed from lib. LOCAL 3/3 (passes 11-13 of 13).
- **STORY-047 MERGED** PR #75 (95f23df3): preview WS server + CSP nonce security hardening. BC-4.03.004. axum WS. LOCAL 3/3 (passes 4-6). STORY-056 + STORY-048 UNLOCKED.
- **STORY-072 MERGED** PR #76 (2667987e): shape gradient fills — `FillSpec::Gradient { from, to }` end-to-end (parser→IR→layout→PPTX/PDF/HTML/DOCX). BC-3.04.001 v1.8. LOCAL 3/3 strict-CLEAN (passes 4-5-6). Security CLEAN; pr-reviewer APPROVE; CI 25/25 green. Rebased over STORY-074 em-sizing interaction (font_size_emu sibling sweep).
- **DEP-PREP MERGED** PR #69 (3e3a978f): [workspace.dependencies] centralized + ADR-022 migrations done.

**14 stories remain. 3 active worktrees (see IN-FLIGHT section above).**

**HELD (next batch after in-flight 4 merge):**
- STORY-057/058/064 (slideforge-cli same-crate conflict — deliver AFTER this batch, serialized)
- STORY-056 (←047 now MERGED — UNBLOCKED; may start after in-flight batch)
- STORY-048 (←047 now MERGED — UNBLOCKED)
- STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must be resolved FIRST)

---

## CURRENT POSITION

Phase 3, **Wave 5 IN PROGRESS** (develop `2667987e`, 76 merged PRs). 8 of 22 done. 14 stories remain. 90 stories / 556 pts total.

- Active worktrees: 3 — STORY-081/082/088 in `.worktrees/` on `feature/STORY-<NNN>`. STORY-072/074/079/080/047 cleaned up post-merge. Open PRs: 0.
- Workspace: ~3807+ pass, 21 skip (cold_budget PERMANENTLY FIXED by STORY-080 PR#73).
- Uncertainty pass: COMPLETE. ADR-022 dep-centralization: DONE. ADR-008 P4 amendment: DONE. ADR-021 async runtime: DONE.

---

## NEXT ACTIONS

**RESUME PROCEDURE (zero context):**
1. Run `vsdd-factory:factory-worktree-health`
2. Verify `git rev-parse develop` == origin/develop == `2667987e`
3. Confirm workspace tests green (`cargo nextest run --workspace --no-fail-fast` — expect ~3807+ pass, 21 skip; cold_budget PERMANENTLY FIXED)
4. Read BACKLOG.md WAVE5-DELIVERY for in-flight status
5. For each in-flight story, check `git -C .worktrees/STORY-<NNN> log --oneline -5` to confirm HEAD matches the table above
6. **Continue in priority order:** STORY-082 + STORY-088 (Pass 4 each), then STORY-081 (Pass 2 after full re-impl). Per-story adversary passes are SERIAL (LESSON-7 + rate-limit).

**PER-STORY DELIVERY SEQUENCE (BC-5.39.001):**
adversary LOCAL 3-CLEAN (passes run SEQUENTIALLY) → demo-recorder per-AC → rebase onto develop `2667987e` → push → pr-manager 9-step (orchestrator dispatches security-reviewer + pr-reviewer per LESSON-5) → STANDING MERGE AUTH: CI-green + security CLEAN + pr-reviewer APPROVE → squash-merge → state-manager post-merge burst → worktree cleanup → LESSON-18 sync check.

**APPLY LESSON-21 to EVERY story exit gate** (prevents post-convergence CI-fix cycles): nextest + `cargo test --workspace --all-features` (shared-process) + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` + cross-platform filesystem test discipline.

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
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 5: **8/22 done** (STORY-089 PR#68, STORY-046 PR#70, STORY-055 PR#71, STORY-079 PR#72, STORY-080 PR#73, STORY-074 PR#74, STORY-047 PR#75, STORY-072 PR#76). 3 in-flight worktrees. 14 stories remain. | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. develop 2667987e (76 merged PRs). STORY-072 merged PR #76. 3 worktrees active. 14 stories remain.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-08 |
| **develop SHA** | `2667987e` (76 merged PRs; origin/develop confirmed; 0 open PRs) |
| **Merged this session** | STORY-079 PR#72, STORY-080 PR#73, STORY-074 PR#74, STORY-047 PR#75, STORY-072 PR#76 |
| **Active worktrees** | 3 — STORY-081/082/088, each in `.worktrees/STORY-<NNN>` on `feature/STORY-<NNN>`. Cleaned up: STORY-072/074/079/080/047. |
| **STORY-082 state** | HEAD `67530b17` (Pass-4 fix-burst applied; code UNCHANGED through passes 5-8); adversary 0/3 (Pass-8 F-P8-MED-1 spec drift fixed — sha2 =0.10.9→=0.11.0 in 4 spec locations; spec_version 1.1→1.2; BC-4.01.003 v1.4 + error-taxonomy v2.28 current); NEXT: Pass 9 against spec v1.2 |
| **STORY-088 state** | HEAD `3b0c6295`; adversary 0/3 (3 passes done, last MED fixed); NEXT: Pass 4 |
| **STORY-081 state** | HEAD `c11d6468`; adversary 0/3 (full re-impl after Pass-1 4-CRIT); NEXT: Pass 2 |
| **Workspace tests** | ~3807+ pass / 21 skip (cold_budget PERMANENTLY FIXED) |
| **factory-artifacts** | Pushed to origin. Fresh sessions: clone + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | Check each worktree HEAD vs table above. Per-story: LOCAL adversary 3-CLEAN (SEQUENTIAL) → demo-recorder → rebase onto 2667987e → pr-manager 9-step → STANDING MERGE AUTH → squash-merge → state-manager post-merge burst → worktree cleanup. Rate-limiting active: dispatch adversary/review passes ONE AT A TIME. Apply LESSON-21 exit gate to every story. HELD next batch: STORY-056/048 (unblocked ←047), STORY-057/058/064 (serialize cli), STORY-060/061 (GIT2-OPENSSL first). |

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
| LESSON-19 (SIBLING-SWEEP) | When a fix-burst changes a canonical VALUE, TEXT, COUNT, or ANCHOR (error code, message template, site count, ADR/BC/PC citation, stale-state comment), it MUST sweep ALL sibling artifacts in ONE burst. Partial propagation repeatedly reset STORY-089 3-CLEAN streak (~11 passes). Orchestrator must grep-map every occurrence itself and dispatch ONE coordinated exhaustive fix. |
| LESSON-20 (ADVERSARY-SPEC-PATHS) | Per-story LOCAL adversary dispatches MUST include ABSOLUTE `.factory/` spec paths (story file, traced BCs, traced ADRs, export-architecture or equivalent spec). The per-story worktree does NOT contain the `.factory/` mount. |
| LESSON-21 (LOCAL-GATE-MIRRORS-CI-MATRIX) | Exit gate MUST include: `cargo test --workspace --all-features` (shared-process, catches global-state/test-isolation bugs nextest masks) + `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` + cross-platform/portable filesystem tests (no `/nonexistent` Unix-root paths; no virgin-global-state assumptions). |
| LESSON-22 (ORCHESTRATOR-VERIFY-UPSTREAM-CLAIMS) | Before routing a deferral/"not supported" based on an implementer's claim of an upstream gap, VERIFY the claim against actual upstream code, and confirm any cited story ID EXISTS. |
| LESSON-OPS-RATE-LIMIT | RATE-LIMITING ACTIVE: backend applies sustained server-side throttle. Dispatch adversary/review passes ONE AT A TIME (serialize). Batching 3+ simultaneous agents triggers "Server is temporarily limiting requests" rejections (transient, retry). |
| LESSON-OPS-CWD | cwd DISCIPLINE: every worktree-scoped agent MUST prefix EVERY bash command with `cd <worktree> &&` (shell does NOT persist cd across calls) and use absolute paths. A violation leaked STORY-081/082 stub files into main repo (cleaned). Include this in test-writer/implementer/demo dispatch prompts. |
| LESSON-OPS-CI-DISKSPACE | STORY-047 snapshots CI failure was a DISK-SPACE infra flake (No space left on device / ld bus error), NOT a code defect — re-run on a fresh runner clears it. reqwest rustls feature pulls heavy native deps (aws-lc-sys + ring) increasing disk pressure (FU-047-DEPS-AWSLC). |
| LESSON-OPS-SPECFIX-SCOPE | Architect/product-owner/story-writer dispatches MUST be scoped to .factory/ ONLY. Forbidden from touching crates/ or committing to develop. (One violation occurred: ADR-021 OBS-2 spec-fix agent committed workspace Cargo.toml to develop — since reset.) |

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
| 2026-06-08 | STORY-082-F-P8-MED1 | Adversary Pass-8 F-P8-MED-1 (MEDIUM, spec-text defect — code correct): STORY-082 spec cited stale `sha2 =0.10.9` in 4 places vs canonical workspace pin `=0.11.0` (shared w/ slideforge-math). Fixed all 4 + reframed to `workspace = true`; full version-pin sweep (quick-xml/ooxmlsdk/chumsky/zip all match). story spec_version 1.1→1.2. Worktree code unchanged (67530b17). Streak reset 0/3 by Pass 8; Pass 9 next against HEAD 67530b17. |
| 2026-06-08 | STORY-082-IMP1-SPECFIX | Adversary Pass-5 IMP-1 (HIGH, spec defect — code was correct): E-PAR-023 (empty section name) is a parse error → exit 1 per BC-1.15.003 three-tier model, but story AC-010/EC-010, BC-4.01.003 PC7/EC-010, and error-taxonomy E-PAR-023 row wrongly said exit 2. Corrected all 3 artifacts: BC-4.01.003 v1.3→v1.4, error-taxonomy v2.27→v2.28 (+OBS-1 message-prefix), STORY-082 spec_version 1.1. Worktree code UNCHANGED (HEAD 67530b17). STORY-082 streak reset to 0/3; next adversary Pass 6 against corrected spec. |
| 2026-06-08 | STORY-072-MERGE | PR #76 squash-merged → develop `2667987e` (76 merged PRs). Shape gradient fills: `FillSpec::Gradient { from, to }` added to slideforge-types; E-PAR-016 path removed; wired parser→IR→layout passthrough→4 exporters (PPTX `<a:gradFill>`, PDF krilla LinearGradient, HTML SVG `<linearGradient>`+`url()`, DOCX solid fallback+warn). LOCAL 3/3 strict-CLEAN (passes 4-5-6). Security CLEAN (2 informational suggestions); pr-reviewer APPROVE (3 non-blocking nits); CI 25/25 green. Rebased onto develop resolving STORY-074 `font_size_emu` interaction (LESSON-19 sibling sweep: gradient_integration.rs import + 3 BrandFonts test literals). KNOWN DEFERRAL intact: ShapeNode→ShapeSpec decode + end-to-end DSL path NOT wired (FU-SHAPE-PIPELINE-WIRING wave-gate). Follow-ups registered: FU-072-SEC002-FILLATTR-INVARIANT (add doc-comment invariant on render.rs fill_attr for pre-escaped values), FU-072-PDF-PUBCRATE (tighten draw_gradient_rect pub→pub(crate)), FU-072-EPAR016-DOC-STALE (E-PAR-016 description stale post-STORY-072 — cleanup when convenient). SEC-001 → links to existing FU-SHAPE-PIPELINE-WIRING (eval decode of gradient string); no duplicate created. |
| 2026-06-08 | STORY-047-MERGE | PR #75 squash-merged → develop `95f23df3` (75 merged PRs). Web Preview Server + CSP nonce security hardening: axum WS endpoint; CSP nonce per-response; BC-4.03.004. LOCAL 3/3 strict-CLEAN (passes 4-6). Security CLEAN; pr-reviewer APPROVE; CI green (disk-space infra flake on snapshots job cleared on re-run — NOT a code defect; FU-047-DEPS-AWSLC registered). Follow-ups: FU-047-SEC005-PATH (DiagnosticMessage.file path normalization), FU-047-DF1-CLIENT-RECONCILE, FU-047-DF2-SCR007-CHROME, FU-047-ADR008-AXUM, FU-047-DEADFN (dead pub ws_upgrade_handler), FU-047-DEPS-AWSLC. STORY-056 + STORY-048 now UNBLOCKED (←047 merged). |
| 2026-06-08 | STORY-074-MERGE | PR #74 squash-merged → develop `3f7f99ed` (74 merged PRs). Brand-aware em sizing: `font_size_emu` (i64) on `Brand` drives em→EMU shape resolution; `DEFAULT_EM_IN_EMU` removed from compiled lib (confined to `#[cfg(test)]`); backward-compatible default 457_200. LOCAL 3/3 strict-CLEAN (passes 11-13; 13 total — code converged since pass 3). Security CLEAN (3 LOW deferrals), pr-reviewer APPROVE, CI green. Follow-ups: FU-074-SEC003-PALETTE-FALLBACK, FU-047-ADR008-AXUM. |
| 2026-06-08 | STORY-080-MERGE | PR #73 squash-merged → develop `e08f2f80` (73 merged PRs). Deflake cross-platform tests. cold_budget PERMANENTLY FIXED. LOCAL 3/3 (passes 5-6-7). Security CLEAN, pr-reviewer APPROVE, CI green. |
| 2026-06-08 | STORY-079-MERGE | PR #72 squash-merged → develop `2f6d5da4` (72 merged PRs). slideforge-diagrams SVG DoS hardening. BC-1.12.003 v1.3. error-taxonomy v2.26. LOCAL 3/3 (passes 3-4-5). Security CLEAN, pr-reviewer APPROVE, CI green. |
| 2026-06-08 | WAVE5-FULLWIDTH-LAUNCH | Human confirmed "full 8 parallel now". 8 per-story worktrees created off develop cbebfd57. All 8 stories Stage-1 launched in parallel. Active worktrees: STORY-082/088/072/074/079/080/047/081. Merge model: parallel development, SERIAL merge with rebase + re-gate. |
| 2026-06-08 | STORY-055-MERGE | PR #71 squash-merged → develop cbebfd57. `slideforge build` CLI + miette diagnostics. Unified compile pipeline. LESSON-21 + LESSON-22 codified. CI all-green. |
| 2026-06-08 | STORY-046-MERGE | PR #70 squash-merged → develop fa85d113. Static HTML exporter. P4 Composite Rendering Model. ADR-008 P4 amendment. BC-4.03.003 v1.4. 23-pass cascade. CI all-green. |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
