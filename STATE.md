---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-07
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
wave_5_status: "IN_PROGRESS — 21 stories remain / 114 pts. STORY-089 MERGED PR #68 (c722c28b)."
develop_sha: "c722c28b"
develop_pr_count: 68
error_taxonomy_version: "v2.24"
workspace_tests: "3393+/3393+ (develop c722c28b; + STORY-089 field-validation suite (~54 tests) + e2e; FieldSchemaValidator live at build; 18 skipped; known flaky: slideforge-diagrams cold_budget timing tracked STORY-080)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## ZERO-CONTEXT RESUME — START HERE

**Project:** slideforge — data-reactive branded document platform (Rust, greenfield, Phase 3 TDD)
**Repository:** https://github.com/drbothen/slideforge | **Workspace:** /Users/jmagady/Dev/slideforge

**Verify dev branch:** `git rev-parse develop` must equal `git rev-parse origin/develop`. Canonical SHA: `c722c28b` (68 merged PRs, 0 open PRs).

**Factory worktree:** `.factory/` on branch `factory-artifacts`. Pushed to origin (human-authorized 2026-06-04). Fresh machines: `git clone <repo> && git worktree add .factory factory-artifacts`.

**Current position:** Phase 3, **Wave 5 IN PROGRESS** (human authorized 2026-06-07). STORY-089 MERGED PR #68 (c722c28b) — 21 stories / 114 pts remain.

**Wave 5 — 21 stories / 114 pts remaining:**

- **STORY-089 MERGED:** field-value type validation (BC-1.18.001 v1.5, ADR-020, error-taxonomy v2.23→v2.24, E-VAL-104 T1/T2). FieldSchemaValidator wired live. Latent dead-letter bug closed. 3-CLEAN (~11-pass cascade). PR #68 c722c28b. STORY-082 spec reconciled (BC-4.01.003 v1.3, export-arch v1.1, taxonomy v2.24 — see STORY-082-SPEC-RECON).
- **Next P0 stories:** STORY-082 (EPIC-14), STORY-081 (EPIC-15), STORY-088 (EPIC-17).
- **Dependency chains:** EPIC-14: 046→047→048; EPIC-15: 055→056→059 (056 also needs 047); EPIC-16: 060→061→062/063; EPIC-17: 064→065; independents: 072/074/079/080/081/082/088.
- **SEC-001-HARDENING (OPEN, non-blocking):** residual string-layer bypass vectors in ImagePathValidator. ANCHORED to image-loading story. Severity: SUGGESTION.
- **SEC-001-DIAG-HARDENING (OPEN, non-blocking LOW):** unbounded user-authored strings embedded verbatim in diagnostic messages (E-VAL-104 T2 + W-VAL-103 pattern). Truncate to ~512 chars. ANCHORED to future validator/diagnostic-hardening story.

**Startup procedure:** (1) run `vsdd-factory:factory-worktree-health` (2) verify `develop == origin/develop` (3) confirm workspace tests green (4) read `.factory/BACKLOG.md` and TaskCreate one task per OPEN item (5) read NEXT ACTIONS below (6) await human go-ahead before picking a story.

**Durable task source:** `.factory/BACKLOG.md` — rebuild in-session tasks from OPEN items there on every session start. State-manager mirrors it alongside STATE.md at every milestone.

**LESSON-18 (MANDATORY after every merge):** After `gh pr merge --squash`, run `git fetch && git merge --ff-only origin/develop` (or `git restore --source=HEAD --staged --worktree .`) to sync the working tree. `git update-ref refs/heads/develop origin/develop` alone moves the branch pointer but leaves the working tree STALE — gate agents will review stale files. Include a disk-presence check before dispatching any gate agent. Discovered: Wave 4 re-gate first attempt (2026-06-07). Lessons file: `.factory/cycles/wave-4-gate/lessons.md`.

**Phases 4–7 (holdout / adversarial / formal hardening / convergence) remain ahead before v1.0.**

---

## CURRENT POSITION

Phase 3, **Wave 5 IN PROGRESS** (develop `c722c28b`, 68 merged PRs). STORY-089 MERGED PR #68. 21 stories / 114 pts remain. 89 stories / 553 pts total.

- Active worktrees: none. Open PRs: 0.
- Workspace: 3393+ pass + STORY-089 field-validation suite (~54 tests) + e2e (18 skipped; known flaky: cold_budget tracked STORY-080).

---

## NEXT ACTIONS

**STATUS: Wave 5 IN PROGRESS. STORY-089 MERGED (PR #68, c722c28b). 21 stories / 114 pts remain. DO NOT auto-pick next story — orchestrator/human decides.**

STORY-089 delivered: FieldType enum + type_matches() + E-VAL-104 (T1 type-mismatch, T2 OneOf violation) + FieldSchemaValidator wired into Stage-5. 8 Priority-1 annotations. chart.data optional. Closed latent validate_fields dead-letter (ADR-020 Decision 8).

**Wave 5 remaining delivery order (P0 first):**
1. **STORY-082** — EPIC-14 independent. Ready.
2. **STORY-081** — EPIC-15 independent (needs 047 before 056). Ready.
3. **STORY-088** — EPIC-17 independent. Ready.
4. Chain stories per dependency graph: 046→047→048; 055→056→059; 060→061→062/063; 064→065; independents 072/074/079/080.

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
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3/4 GATE PASSED. Wave 4: 23/23 COMPLETE + follow-ups CLOSED. Wave 5: IN PROGRESS — STORY-089 MERGED PR #68 (c722c28b); 21 stories/114 pts remain. | Per-story delivery |
| Phases 4-7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

## Wave 4 Story Status (summary)

- **Batch A (10/10 MERGED):** PRs #39-#50 — STORY-035/036/043/044/073/075/076/078/045/077 (+ STORY-077 follow-up PR #50).
- **Batch B (6/6 MERGED):** PRs #51-#56 — STORY-041/042/037/038/039/040; pptx chain + docx chain complete.
- **Batch C (5/5 MERGED):** PR #57 STORY-083, PR #58 STORY-084, PR #59 STORY-085, PR #60 STORY-049, PR #61 STORY-050.
- **Remediation (3 MERGED):** PR #62 STORY-086 (content threading + TextTag), PR #63 STORY-087 (color-coded slide types), PR #64 image-alt fix (NEW-INT-001).
- **Wave 4 = 23/23 COMPLETE. Re-gate FULLY PASSED on develop 02d484cf.**

---

## Session Resume Checkpoint

**Wave 5 IN PROGRESS. develop c722c28b (68 merged PRs). STORY-089 MERGED PR #68. 21 stories / 114 pts remain. 89 stories / 553 pts total.**

| Field | Value |
|-------|-------|
| **Date** | 2026-06-07 |
| **Position** | Wave 5 IN PROGRESS. STORY-089 MERGED PR #68 (c722c28b). FieldSchemaValidator live; error-taxonomy v2.24 (v2.23 at STORY-089; v2.24 at STORY-082 spec recon); BC-1.18.001 v1.5; ADR-020 Decision 8 closed. STORY-082 spec reconciled (BC-4.01.003 v1.3, export-arch v1.1). 21 stories/114 pts remain. |
| **develop SHA** | `c722c28b` (68 merged PRs; origin/develop confirmed; 0 open PRs) |
| **Active worktrees** | none |
| **Workspace tests** | 3393+ + ~54 field-validation tests + e2e (18 skipped; known flaky: cold_budget STORY-080) |
| **factory-artifacts** | PUSHED to origin. Fresh sessions: clone + `git worktree add .factory factory-artifacts`. |
| **RESUME INSTRUCTION** | STORY-089 done. Next P0s (orchestrator/human decides order): STORY-082 (EPIC-14), STORY-081 (EPIC-15), STORY-088 (EPIC-17). Chain order: EPIC-14 046→047→048; EPIC-15 055→056→059; EPIC-16 060→061→062/063; EPIC-17 064→065; independents 072/074/079/080. Open follow-ups: SEC-001-HARDENING (image-loading story), SEC-001-DIAG-HARDENING (diagnostic truncation, future story). Phases 4-7 remain for v1.0. |

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
| LESSON-19 (SIBLING-SWEEP) | When a fix-burst changes a canonical VALUE, TEXT, COUNT, or ANCHOR (error code, message template, site count, ADR/BC/PC citation, stale-state comment), it MUST sweep ALL sibling artifacts in ONE burst: production code + every consuming spec (BC/ADR/taxonomy/story) + test files + doc-comments + fixtures — verified by exhaustive grep BEFORE re-running the adversary. Partial propagation repeatedly reset STORY-089 3-CLEAN streak (~11 passes). Orchestrator must grep-map every occurrence itself and dispatch ONE coordinated exhaustive fix, not per-file partial fixes. Full narrative: `.factory/cycles/STORY-089/lessons.md`. |

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
