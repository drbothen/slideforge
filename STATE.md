---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-03
phase_1_approved: 2026-05-25
phase_2_approved: 2026-05-25
phase_1_convergence: "17 passes, 69 findings, 3/3 clean (passes 15-16-17)"
phase_2_convergence: "22 passes, 96+ findings, 3/3 clean (passes 20-21-22)"
prd_bcs: 109
prd_hs: 15
prd_vps: 15
prd_supplements: 4
spikes_resolved: 7
spikes_total: 7
total_stories: 81
total_points: 491
total_waves: 6
total_epics: 21
dtu_required: false
dtu_assessment: 2026-05-24
dtu_clones_built: n/a
dtu_services: []
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed"
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_batch_a_complete: 10
wave_4_batch_a_total: 10
wave_4_started: 2026-05-31
wave_4_total_stories: 18
wave_4_total_points: 114
wave_5_total_points: 109
develop_sha: "9730e6a3"
develop_pr_count: 54
error_taxonomy_version: "v2.13"
workspace_tests: "~2900+ (54 merged PRs + 1 fix commit)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. Generates branded .pptx, .docx, .pdf, .html, and web preview from a single .sf DSL file with data binding, iteration, conditionals, and a plugin-first architecture.

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

---

## CURRENT POSITION

Phase 3, **Wave 4 — 14/18 merged. Batch B pptx chain (037→038) DONE. Batch B docx chain (041→042) DONE.**

- `develop` = `9730e6a3` (54 merged PRs + 1 post-merge fix commit). **Open PRs: 0. Active worktrees: none.**
- Workspace builds clean. ~2900+ tests pass, 0 failures.
- STORY-037 MERGED PR #52 (2ebf184f). STORY-038 MERGED PR #54 (c031805c). Post-merge fix commit `9730e6a3` closes F-038-P2-M1 (Title/Subtitle shapes gate through AC-011 idx-chain).
- STORY-041 MERGED PR #51 (a3b47303). STORY-042 MERGED PR #53 (56f3f57d). Docx chain COMPLETE.
- ADR-015 + Addendum A on factory-artifacts: governs brand→pptx OOXML rendering boundary + STORY-037/038 scope split. Relevant to STORY-039/040.
- S1/S2/S3/S4 + SEC-001 (CWE-190) + DEF-P5-001: all resolved in scope in STORY-038.

**Batch B remaining:** STORY-039 → STORY-040 (pptx accessibility + speaker notes). **USER HAS AUTHORIZED proceeding.**
**Batch C (after Batch B):** STORY-049 → STORY-050.
**Wave 4 gate** runs only after ALL 18 Wave 4 stories merge.

---

## NEXT ACTIONS (fresh orchestrator — execute in order)

1. **STORY-039** — PPTX Accessibility Metadata. Depends on STORY-038 (MERGED). Authorized.
   - Full per-story delivery: `git worktree add .worktrees/S-039 -b feature/S-039 develop`
   - Flow: stubs → failing tests → Red Gate + density → implement → per-story adversary 3-CLEAN → demo → push → PR → security-reviewer + pr-reviewer → CI green → `gh pr merge --squash` → cleanup → state-manager burst.
   - Architecture context: ADR-015 + Addendum A (`.factory/specs/architecture/adr/`) governs pptx rendering boundary.
   - Reference: `workflows/phases/per-story-delivery.md`

2. **STORY-040** — PPTX Speaker Notes + notesMaster1.xml. Depends on STORY-039. Completes Batch B pptx chain.

3. **Batch C:** STORY-049 → STORY-050. After Batch B complete.

4. **Wave 4 gate** after all 18 stories merged.

---

## STANDING AUTHORIZATIONS

**STANDING MERGE AUTH (granted 2026-06-02):** Orchestrator MAY squash-merge any PR that is CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, without re-asking the human. GitHub blocks author self-approve, so orchestrator runs `gh pr merge --squash` directly once those gates pass. Revocable by human.

**factory-artifacts LOCAL ONLY:** Many unpushed commits accumulated this session. A fresh session on this machine resumes from local `.factory/` worktree. Pushing factory-artifacts to remote requires explicit human authorization.

---

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + arch (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 81 stories, 21 epics, 6 waves, 491 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. Wave 4: 14/18 merged. Batch B remaining: STORY-039→040. Then Batch C: STORY-049→050. | Per-story delivery |
| Phases 4–7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

## Wave 4 Story Status

**Batch A — ALL MERGED (10/10):** PRs #39–#49 (develop c8913cad). STORY-035, -036, -043, -044, -073, -075, -076, -078, -045, -077. STORY-077 follow-ups PR #50 (f2573bb1, 2026-06-03).

**Batch B — 4/4 MERGED:**
- STORY-041 MERGED PR #51 (a3b47303, 2026-06-03)
- STORY-042 MERGED PR #53 (56f3f57d, 2026-06-03) — docx chain COMPLETE
- STORY-037 MERGED PR #52 (2ebf184f, 2026-06-03)
- STORY-038 MERGED PR #54 (c031805c, 2026-06-03) — pptx chain 037→038 DONE; post-merge fix 9730e6a3

**Batch B remaining (pptx):** STORY-039 → STORY-040 (authorized)
**Batch C (after Batch B):** STORY-049 → STORY-050

---

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-06-03 |
| **Position** | Wave 4: 14/18 merged. Batch B pptx (037→038) DONE + docx (041→042) DONE. NEXT: STORY-039 (authorized). |
| **develop SHA** | `9730e6a3` (54 PRs + 1 fix commit) |
| **Active worktrees** | none |
| **Open PRs** | 0 |
| **Workspace crates** | 16 |
| **Spec deltas this session** | ADR-015 + Addendum A; BC-3.02.002 v1.5; BC-1.14.003 v1.3; error-taxonomy v2.13 — all on factory-artifacts |
| **factory-artifacts** | Local only — push requires explicit human authorization |

---

## Standing Process Rules

| ID | Rule |
|----|------|
| LESSON-1 | Adversary dispatches MUST pass the ABSOLUTE worktree path (`--cwd /Users/jmagady/Dev/slideforge/.worktrees/STORY-NNN`). |
| LESSON-2 | Canonical clippy gate: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`. Bare `-D warnings` misses pedantic lints. Also run `RUSTDOCFLAGS="-D warnings" cargo doc` for doc-link gates. |
| LESSON-3 | Re-run full pre-push gate (fmt + pedantic clippy + nextest) after EVERY commit to a feature branch before PR. |
| LESSON-5 | pr-manager CANNOT spawn sub-agents. Orchestrator dispatches security-reviewer + pr-reviewer independently. |
| LESSON-6 | RELAXED by STANDING MERGE AUTH: orchestrator may merge when CI-green + security-reviewer CLEAN + pr-reviewer APPROVE. |
| LESSON-7 | Per-story adversary convergence passes run SEQUENTIALLY (one at a time). Never parallelize passes of one story — parallel final passes can disagree, wasting a pass and creating adjudication ambiguity. Only parallelize across DIFFERENT stories. |
| LESSON-8 | Every pre-push gate AND implementer exit-gate MUST run the FULL canonical clippy (LESSON-2) + `cargo fmt --all -- --check`. Bare `-D warnings` misses pedantic + fmt and ships CI-RED PRs (happened twice on STORY-038). |
| LESSON-9 | When PR-level security/pr-reviewer findings are fixed AFTER per-story adversary convergence, the code diff changes — RE-RUN security-reviewer + pr-reviewer + wait for CI before merge. Do not assume prior convergence still holds. |
| LESSON-10 | For strict 3-CLEAN on compliance/extension stories: do a PROACTIVE exhaustive doc-vs-code + sibling-site (warn/error-propagation) consistency audit BEFORE final convergence passes, to avoid per-finding ping-pong (STORY-038 took 16 passes). |
| LESSON-11 | Stub/test-writer for EXTENSION stories often produces a high GREEN-BY-DESIGN ratio (prior-merged behavior is legitimate PRE-EXISTING-BEHAVIOR). But stricter new tests can expose REAL bugs in already-merged code — treat such finds as in-scope fixes for the compliance story. |

---

## Open Follow-Ups (non-blocking)

| Item | Severity | Target |
|------|----------|--------|
| SEC-042-001 (CWE-400): docx section serializers no upper bound on items count | LOW | STORY-049 / layout hardening |
| SEC-001 (CWE-494, veraPDF): Docker `verapdf/cli:latest` not digest-pinned | MED | Before v1.0 / Phase 6 |
| SEC-003 (CWE-189): `emu_to_pt` i64→f32 precision loss | LOW | Phase 6 Kani |
| SEC-004: CI tee predictable temp path (self-hosted only) | LOW | Phase 6 |
| SEC-037-001: pptx InlineNode::Link needs ALLOWED_LINK_SCHEMES + SafeUrl (CWE-601) | LOW | pptx link-rendering story / shared SafeUrl hardening |
| SafeUrl shared newtype (pptx/pdf/html will hit same docx SEC-001 gap) | cross-exporter | Hardening story before Phase 6 |
| cold_budget flaky perf test (test_cold_budget_under_200ms, slideforge-diagrams): macOS CI jitter | LOW | STORY-080 |
| OBS-FU-HTML-REDIR: HTML exporter URL open-redirect (CWE-601) | security | STORY-046 |
| E-PAR-021 cosmetic message format (nesting_depth_exceeded_msg raw byte offset) | minor | Message-cleanup follow-up |
| OBS-FU-P1-A: DSL inner-quote round-trip | minor | Lexer follow-up |
| OBS-FU-P1-C: Link URL `)` truncation (Wikipedia disambiguation links) | minor | Link-parser follow-up |
| BC-1.14.001/002 still `subsystem: SS-TBD` | LOW | Next spec-hygiene pass |
| OBS-P6-001: PDF exporter ignores `opts.strict`/warnings | LOW | Post-STORY-045 wave-gate |
| DEF-041-P5-001: EC-002 paragraph-break preservation depends on STORY-035 evaluator (deferred) | integration | Wave-gate |
| STORY-081: slide-level inline markup (P0 for v1.0, Wave 5, EPIC-18, depends STORY-077) | P0 | Wave 5 |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
