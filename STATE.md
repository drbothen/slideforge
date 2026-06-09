---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-09
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
wave_5_status: "10 of 22 Wave-5 stories MERGED (…STORY-072 PR#76, STORY-082 PR#77, STORY-088 PR#78). 12 stories remain. In-flight (1 active worktree): STORY-081 (adversary 0/3 — Pass 2 DONE NOT CLEAN; implementer fix burst COMPLETE HEAD 885d83027295946f97bcca71f1ece620fc6dd533; all 5 Pass-2 findings CLOSED; NEXT: adversary Pass 3). CI infra fix PR#79 merged."
develop_sha: "15838de1"
develop_pr_count: 79
error_taxonomy_version: "v2.28"
workspace_tests: "3754 pass / 0 fail (STORY-081 worktree 885d8302; LESSON-21 exit gate ALL GREEN: fmt clean, clippy pedantic+unwrap_used clean, nextest 3754, shared-process cargo test pass, rustdoc -D warnings clean; linux-arm64 CI: 1 unresolved nextest failure — FU-CI-ARM64-TEST-FAILURE)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge | **Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop`. Canonical SHA: `15838de1` (79 merged PRs, 0 open PRs).

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04; ongoing pushes authorized).

**Current position:** Phase 3, **Wave 5 IN PROGRESS**. STORY-088 MERGED PR #78 (15838de1, ADMIN OVERRIDE). CI fix PR #79 merged. 10 of 22 done. 12 stories remain.

---

## IN-FLIGHT WORKTREES — EXACT RESUME STATE

**1 active worktree. Based on cbebfd57; MUST rebase onto develop 15838de1 at PR step.**

### STORY-081 — Slide-Level Inline Markup (EPIC-18, BC-3.02.002, 13 pts)
- **Worktree:** `.worktrees/STORY-081` | **Branch:** `feature/STORY-081` | **HEAD:** `885d83027295946f97bcca71f1ece620fc6dd533` (was c11d6468)
- **Adversary streak:** **0/3** — Pass 2 findings ALL CLOSED (implementer fix burst COMPLETE). NEXT: adversary Pass 3 (fresh 3-clean streak starts here).
- **Pass 2 findings (CLOSED):**
  - C1[CRIT] PDF body/bullet dead-wiring → fixed: `extract_all_inline_text` routed through live draw path; dead `extract_inline_text` removed.
  - C2[HIGH] Vacuous PDF tests → fixed: `build()`-driven PDF assertions with `ActualText` proof added.
  - C3[HIGH] `FrameContent::Subtitle(Arc<str>)` flattened inlines → fixed: new `FrameContent::SubtitleInlines(Vec<InlineNode>)` variant wired through layout + all 4 exporters (PPTX/DOCX/HTML/PDF).
  - HTML/PDF rich title AC-006(3) → fixed: via existing `title_inlines` shadow-field pattern (matching sound DOCX path; `FrameContent::Title` NOT widened — see arch note below).
  - I1[MED] PPTX `InlineNode::Math(_)` silent drop → fixed: `tracing::warn!` EC-008 pattern added.
- **ARCH NOTE for Pass 3 to adjudicate:** `FrameContent::Title` was NOT widened to `Vec<InlineNode>` (cascade ~118 sites); HTML/PDF rich title uses `title_inlines` shadow field. Creates asymmetry: `SubtitleInlines` variant exists but no `TitleInlines` variant. Flagged as potential Pass-3 observation.
- **KNOWN LIMITATION (consistent, not a regression):** PDF subtitle/title inline text content preserved; per-span font switching NOT implemented (consistent with PDF body inline limitation; candidate follow-up).
- **LESSON-21 exit gate:** ALL GREEN — fmt clean; clippy pedantic+unwrap_used clean; nextest 3754 pass 0 fail; shared-process `cargo test` pass; `rustdoc -D warnings` clean.
- **NEXT ACTION:** Adversary Pass 3 (fresh-context). 3-CLEAN streak starts at 0. Findings in `cycles/STORY-081/adversarial-reviews/adversary-STORY-081-pass-2.md`.

---

## WAVE 5 DELIVERY SUMMARY

**10 of 22 done (develop 15838de1, 79 PRs):**

- **STORY-089 MERGED** PR #68 (c722c28b): field-value type validation. FieldSchemaValidator live. error-taxonomy v2.24. ADR-020.
- **STORY-046 MERGED** PR #70 (fa85d113): Static HTML exporter (slideforge-html crate). P4 Composite Rendering Model. BC-4.03.003 v1.4. STORY-047 + STORY-081 UNLOCKED.
- **STORY-055 MERGED** PR #71 (cbebfd57): `slideforge build` CLI + miette diagnostics. STORY-057/058/060/064 UNLOCKED. STORY-056 UNLOCKED (←047+055 both now merged).
- **STORY-079 MERGED** PR #72 (2f6d5da4): slideforge-diagrams SVG DoS hardening. SEC-001+SEC-002 guards. BC-1.12.003 v1.3. error-taxonomy v2.26. LOCAL 3/3 (passes 3-4-5).
- **STORY-080 MERGED** PR #73 (e08f2f80): deflake cross-platform tests. cold_budget PERMANENTLY FIXED. LOCAL 3/3 (passes 5-6-7).
- **STORY-074 MERGED** PR #74 (3f7f99ed): brand-aware em sizing. `font_size_emu` (i64). DEFAULT_EM_IN_EMU removed from lib. LOCAL 3/3 (passes 11-13 of 13).
- **STORY-047 MERGED** PR #75 (95f23df3): preview WS server + CSP nonce security hardening. BC-4.03.004. axum WS. LOCAL 3/3 (passes 4-6). STORY-056 + STORY-048 UNLOCKED.
- **STORY-072 MERGED** PR #76 (2667987e): shape gradient fills — `FillSpec::Gradient { from, to }` end-to-end (parser→IR→layout→PPTX/PDF/HTML/DOCX). BC-3.04.001 v1.8. LOCAL 3/3 strict-CLEAN (passes 4-5-6). Security CLEAN; pr-reviewer APPROVE; CI 25/25 green. Rebased over STORY-074 em-sizing interaction (font_size_emu sibling sweep).
- **STORY-082 MERGED** PR #77 (c60cca36): PPTX slide-grouping sections — full pipeline DSL→parser→eval→layout→PPTX `<p14:sectionLst>`. BC-4.01.003 v1.4 + BC-1.14.003. SlideSectionEntry in slideforge-types (re-exported via layout). Deterministic RFC4122 v5 GUIDs. E-PAR-023 exit 1, W-PAR-002 duplicate warning. CONVERGED 3/3 strict-CLEAN (passes 16-17-18) after 18-pass cascade. Findings caught+fixed: adjacent-duplicate section merge (HIGH, per-instance-id fix), SEC-100/CWE-116 section-name sanitization (HIGH), vacuous AC-010 tests (TD-VSDD-059), E-PAR-023 exit 2→1 spec (BC v1.4, taxonomy v2.28), sha2 pin =0.11.0 spec. Security CLEAN (1 LOW pre-existing, 1 SUGGESTION); pr-reviewer APPROVE (2 non-blocking OBS); CI 25/25 green. Rebased over STORY-074/072 (font_size_emu + slide_sections struct-field sibling sweep). Follow-up: FU-082-SEC-S001-NUL-CALLSITE-TEST.
- **STORY-088 MERGED** PR #78 (15838de1, ADMIN OVERRIDE): bullets list-literal DSL `bullets: ["A","B"]` across all 4 FieldValue value positions (field-value/@var/set-rule AC-012/variant-vars AC-013) via shared `list_literal_elements` combinator; E-PAR-024 (non-string element / nested list, non-recursive O(1) depth tracker); dsl-spec v1.1, error-taxonomy v2.28. CONVERGED 3/3 (passes 8-9-10) after 10-pass cascade — human-approved scope expansion + caught/fixed a self-introduced CRITICAL recursion DoS + quad-duplication→shared-combinator refactor. Security CLEAN; pr-reviewer APPROVE; 19/20 CI green; linux-arm64 nextest failure DEFERRED (FU-CI-ARM64-TEST-FAILURE) via admin merge per human. FU-088-BC10102-ANCHOR registered (pre-existing BC-1.01.002 title/anchor mismatch, spec-steward, non-blocking).
- **DEP-PREP MERGED** PR #69 (3e3a978f): [workspace.dependencies] centralized + ADR-022 migrations done.
- **CI-FIX MERGED** PR #79: ci.yml test-matrix `timeout-minutes` 30→75 + `cache-on-failure: "true"` (Swatinem/rust-cache). Roots out linux-arm64 cold-build-timeout self-perpetuating loop (cancelled jobs never saved cache). ci-workflow-analyzer caught initial no-op (`save-always` invalid for rust-cache) → corrected to `cache-on-failure`.

**12 stories remain. 1 active worktree (see IN-FLIGHT section above).**

**HELD (next batch after in-flight merges):**
- STORY-057/058/064 (slideforge-cli same-crate conflict — deliver AFTER this batch, serialized)
- STORY-056 (←047 now MERGED — UNBLOCKED; may start after in-flight batch)
- STORY-048 (←047 now MERGED — UNBLOCKED)
- STORY-060/061 (FU-SEC-001-GIT2-OPENSSL must be resolved FIRST)

**OPEN FOLLOW-UPS:**
- **FU-CI-ARM64-TEST-FAILURE** (HIGH — investigate on next PR): A nextest test fails on `test (linux-arm64)` only (other 3 platforms + local 3916-test suite PASS). Exact test unknown — arm64 runner finalization hang prevented log retrieval on PR #78. On the NEXT PR, capture the failed test name immediately. Likely a pre-existing perf/timing flake (test_cold_budget_under_200ms / http_4xx retry) on the slow emulated arm64 runner. Deferred via admin-override merge on STORY-088 per human direction.
- **FU-088-BC10102-ANCHOR** (spec-steward; non-blocking): pre-existing BC-1.01.002 H1 title/anchor mismatch.

---

## CURRENT POSITION

Phase 3, **Wave 5 IN PROGRESS** (develop `15838de1`, 79 merged PRs). 10 of 22 done. 12 stories remain. 90 stories / 556 pts total.

- Active worktrees: 1 — STORY-081 in `.worktrees/STORY-081` on `feature/STORY-081` HEAD `885d8302`. STORY-072/074/079/080/047/082/088 cleaned up post-merge. Open PRs: 0.
- Workspace: 3754 pass / 0 fail (STORY-081 worktree; LESSON-21 exit gate ALL GREEN; cold_budget PERMANENTLY FIXED by STORY-080 PR#73; linux-arm64 CI has 1 unresolved nextest failure — FU-CI-ARM64-TEST-FAILURE).
- Uncertainty pass: COMPLETE. ADR-022 dep-centralization: DONE. ADR-008 P4 amendment: DONE. ADR-021 async runtime: DONE.

---

## NEXT ACTIONS

**RESUME PROCEDURE (zero context):**
1. Run `vsdd-factory:factory-worktree-health`
2. Verify `git rev-parse develop` == origin/develop == `15838de1`
3. Confirm workspace tests green (`cargo nextest run --workspace --no-fail-fast` — expect ~3916+ pass, ~20 skip; cold_budget PERMANENTLY FIXED; NOTE: linux-arm64 CI has 1 unresolved nextest failure — FU-CI-ARM64-TEST-FAILURE; capture test name on next PR run)
4. Read BACKLOG.md WAVE5-DELIVERY for in-flight status
5. For each in-flight story, check `git -C .worktrees/STORY-<NNN> log --oneline -5` to confirm HEAD matches the table above
6. **Continue in priority order:** STORY-081 — implementer fix burst COMPLETE (HEAD 885d8302; all 5 Pass-2 findings CLOSED; LESSON-21 exit gate ALL GREEN). NEXT: adversary Pass 3 (fresh-context; 3-CLEAN streak starts at 0). Pass 2 findings archived: `cycles/STORY-081/adversarial-reviews/adversary-STORY-081-pass-2.md`. Arch note for Pass 3: `FrameContent::Title` asymmetry (SubtitleInlines variant exists, no TitleInlines; title uses shadow field). Per-story adversary passes are SERIAL (LESSON-7 + rate-limit).

**PER-STORY DELIVERY SEQUENCE (BC-5.39.001):**
adversary LOCAL 3-CLEAN (passes run SEQUENTIALLY) → demo-recorder per-AC → rebase onto develop `15838de1` → push → pr-manager 9-step (orchestrator dispatches security-reviewer + pr-reviewer per LESSON-5) → STANDING MERGE AUTH: CI-green + security CLEAN + pr-reviewer APPROVE → squash-merge → state-manager post-merge burst → worktree cleanup → LESSON-18 sync check.

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
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 5: **10/22 done** (STORY-089 PR#68, STORY-046 PR#70, STORY-055 PR#71, STORY-079 PR#72, STORY-080 PR#73, STORY-074 PR#74, STORY-047 PR#75, STORY-072 PR#76, STORY-082 PR#77, STORY-088 PR#78). 1 in-flight worktree. 12 stories remain. | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. develop 15838de1 (79 merged PRs). STORY-081 implementer fix burst COMPLETE — all 5 Pass-2 findings CLOSED — LESSON-21 ALL GREEN. 1 worktree active. 12 stories remain.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-09 |
| **develop SHA** | `15838de1` (79 merged PRs; origin/develop confirmed; 0 open PRs) |
| **Merged this session** | STORY-088 PR#78 (ADMIN OVERRIDE), CI-fix PR#79; STORY-072/082/088 also merged this session |
| **Active worktrees** | 1 — STORY-081 in `.worktrees/STORY-081` on `feature/STORY-081` HEAD `885d8302`. Cleaned up: STORY-088 (+ previously 072/074/079/080/047/082). |
| **STORY-081 state** | HEAD `885d8302` (was c11d6468); adversary 0/3; Pass 2 findings ALL CLOSED (implementer fix burst COMPLETE); LESSON-21 ALL GREEN (fmt/clippy/nextest 3754/cargo-test/rustdoc clean); NEXT: adversary Pass 3 (fresh 3-CLEAN streak starts at 0) |
| **STORY-081 arch note** | `FrameContent::Title` NOT widened (cascades ~118 sites); HTML/PDF rich title uses `title_inlines` shadow field. `SubtitleInlines` variant exists; no `TitleInlines` variant. Pass 3 to adjudicate asymmetry. PDF per-span font switching not implemented for subtitle/title (consistent; candidate follow-up). |
| **Workspace tests** | 3754 pass / 0 fail (STORY-081 worktree LESSON-21 gate; cold_budget PERMANENTLY FIXED; linux-arm64 CI: 1 unresolved nextest failure — FU-CI-ARM64-TEST-FAILURE) |
| **factory-artifacts** | Pushed to origin. Fresh sessions: clone + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | STORY-081 implementer fix burst DONE. All 5 Pass-2 findings CLOSED (C1 PDF dead-wiring+dead fn removed, C2 build()-driven PDF assertions+ActualText, C3 SubtitleInlines variant+all-4-exporter wiring, HTML/PDF rich title via title_inlines shadow-field, I1 tracing::warn EC-008). LESSON-21 exit gate ALL GREEN. NEXT: dispatch adversary Pass 3 (fresh-context; 3-CLEAN streak starts at 0). Arch note in IN-FLIGHT section for Pass 3. Pass 2 findings: `cycles/STORY-081/adversarial-reviews/adversary-STORY-081-pass-2.md`. Per-story: LOCAL adversary 3-CLEAN (SEQUENTIAL) → demo-recorder → rebase onto 15838de1 → pr-manager 9-step → STANDING MERGE AUTH → squash-merge → state-manager post-merge burst → worktree cleanup. Rate-limiting: ONE adversary/review pass at a time. On next PR: CAPTURE linux-arm64 nextest failure test name immediately (FU-CI-ARM64-TEST-FAILURE). HELD next batch: STORY-056/048 (unblocked ←047), STORY-057/058/064 (serialize cli), STORY-060/061 (GIT2-OPENSSL first). |

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
| 2026-06-09 | STORY-081-FIX-BURST | STORY-081 implementer fix burst COMPLETE. HEAD 885d83027295946f97bcca71f1ece620fc6dd533 (was c11d6468). All 5 Pass-2 findings CLOSED: C1 PDF body/bullet dead-wiring (extract_all_inline_text routed through live draw path; dead extract_inline_text removed); C2 build()-driven PDF assertion with ActualText proof; C3 FrameContent::SubtitleInlines(Vec<InlineNode>) new variant wired through layout + all 4 exporters; HTML/PDF rich title AC-006(3) via existing title_inlines shadow-field (matching sound DOCX path; FrameContent::Title NOT widened ~118 sites); I1 PPTX InlineNode::Math tracing::warn! EC-008 pattern. LESSON-21 exit gate ALL GREEN (fmt/clippy pedantic+unwrap_used/nextest 3754 pass 0 fail/cargo test shared-process/rustdoc -D warnings). Adversary streak: 0/3 (fix burst does not advance). NEXT: adversary Pass 3 (fresh-context). Arch asymmetry (SubtitleInlines variant exists, no TitleInlines; title uses shadow field) flagged for Pass-3 adjudication. PDF per-span font switching for subtitle/title not implemented (consistent with PDF body inline limitation; candidate follow-up). |
| 2026-06-09 | STORY-081-AC006-3-DECISION | Human adjudicated AC-006(3): fix HTML/PDF rich title in same burst as C3 (widen FrameContent Title/Subtitle inline seam once). NOT deferred. Per adversary recommendation + production-grade default. |
| 2026-06-09 | CI-ARM64-TIMEOUT-FIX | PR #79 merged → develop. ci.yml test-matrix timeout 30→75 min + cache-on-failure:true (Swatinem/rust-cache). Roots out the linux-arm64 cold-build-timeout loop (cancelled jobs never saved cache → perpetual cold builds after foundational-crate changes). ci-workflow-analyzer caught initial no-op (save-always invalid for rust-cache) → corrected to cache-on-failure. |
| 2026-06-09 | STORY-088-MERGE | PR #78 squash-merged (ADMIN OVERRIDE) → develop `15838de1`. Bullets list-literal DSL across all 4 FieldValue value positions via shared list_literal combinator; E-PAR-024; non-recursive O(1) nested-depth tracker. CONVERGED 3/3 (passes 8-9-10), 10-pass cascade. Human-approved scope expansion (AC-012 set-rule list default, AC-013 variant vars list override). Security CLEAN; pr-reviewer APPROVE; 19/20 CI green. dsl-spec v1.1, error-taxonomy v2.28. |
| 2026-06-09 | STORY-088-ARM64-DEFER | PR #78 `test (linux-arm64)` had a genuine nextest FAILURE (other 3 platforms + local 3916-test suite PASS; exact test UNKNOWN — runner hung in finalization, logs unretrievable). Human directed admin-override merge + defer investigation to next PR. STORY-088 tests run <0.04s on all platforms; suspected pre-existing perf/timing flake on slow emulated arm64 runner, not a STORY-088 defect. Tracked: FU-CI-ARM64-TEST-FAILURE. |
| 2026-06-09 | STORY-088-SCOPE-EXPANSION | Adversary Pass-4 surfaced 2 MED intent-pending findings (list-literals produced cryptic errors in set-rule defaults + variant vars overrides). Human ADJUDICATED: extend full support. story v1.2→v1.3 (AC-012 set-rule list default, AC-013 variant vars list override; same Value::List semantics + E-PAR-024 element rules, no new merge semantics). Implemented at worktree HEAD 72787dcb (SetRuleValue::List variant, variant_value list arm, eval; 13 TDD tests; OBS-088-P4-002 stale #[ignore] doc-comment fixed; exit gate green 3740 pass). FU-088-BC10102-ANCHOR registered (pre-existing BC-1.01.002 title/anchor mismatch, spec-steward, non-blocking). STORY-088 streak reset 0/3; Pass 5 next against story v1.3. |
| 2026-06-09 | STORY-082-MERGE | PR #77 squash-merged → develop `c60cca36` (77 merged PRs). PPTX slide-grouping sections (`<p14:sectionLst>`): full pipeline DSL→parser→eval(single-pass per-instance-id membership)→layout passthrough→`SectionListBuilder::inject`. SlideSectionEntry in slideforge-types (re-exported via layout). Deterministic RFC4122 v5 GUIDs. E-PAR-023 (empty name, fatal exit 1), W-PAR-002 (duplicate, warning exit 0, both emitted). CONVERGED 3/3 strict-CLEAN (passes 16-17-18) after 18-pass LOCAL cascade. Findings caught+fixed: adjacent-duplicate section merge (HIGH, per-instance-id fix), SEC-100/CWE-116 section-name sanitization (HIGH), vacuous AC-010 tests (TD-VSDD-059), E-PAR-023 exit 2→1 spec (BC v1.4, taxonomy v2.28), sha2 pin =0.11.0 spec. Security CLEAN (1 LOW pre-existing, 1 SUGGESTION); pr-reviewer APPROVE (2 non-blocking OBS); CI 25/25 green. Follow-ups: FU-082-SEC-S001-NUL-CALLSITE-TEST (NUL-byte call-site test via build_ext_lst). |
| 2026-06-09 | STORY-082-OBS-P15-1 | Adversary Pass-15 OBS-P15-1 (LOW, spec-location anchor): STORY-082 spec showed `SlideSectionEntry` defined in `slideforge-layout`; actually defined in `slideforge-types/src/deck.rs`, re-exported via `slideforge-layout/src/lib.rs`. Corrected across 6 spec locations (Subsystem Anchor, Scope Overview §2, Tasks, File Structure table, Forbidden Dependencies, Test Strategy); story spec_version 1.2→1.3. Worktree code unchanged (separately, test fix F-P15-MED-1 landed at HEAD e40a410f). |
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
