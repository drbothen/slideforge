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
develop_sha: "c8913cad"
develop_pr_count: 49
workspace_tests: "~2680+ (49 PRs merged)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. Generates branded .pptx, .docx, .pdf, .html, and web preview from a single .sf DSL file with data binding, iteration, conditionals, and a plugin-first architecture.

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

---

## POSITION

Phase 3, **Wave 4 Batch A = 10/10 COMPLETE.**
- develop = `c8913cad` (49 merged PRs). Open PRs: 0. Active worktrees: none (`.worktrees/STORY-077` removed post-merge).
- **Next:** Wave 4 gate (wave-gate skill) OR Wave 4 Batch B start — STORY-037→038→039→040 + STORY-041→042 (parallel after Batch A, per wave-schedule.md). Confirm with orchestrator before proceeding.

---

## STANDING AUTHORIZATIONS

**STANDING MERGE AUTH (granted by human 2026-06-02):** The orchestrator MAY auto-merge any PR that is fully CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, WITHOUT asking the human each time (revocable by human). This RELAXES LESSON-6 for converged PRs. LESSON-6's "GitHub blocks author self-approve" means the orchestrator does `gh pr merge --squash` itself once those gates pass.

---

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + arch (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 81 stories, 21 epics, 6 waves, 491 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. Wave 4 Batch A 10/10 COMPLETE (STORY-077 merged PR #49 c8913cad 2026-06-03). Batch B next. | Per-story delivery |
| Phases 4–7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

## Wave 4 Batch A Status (10/10 COMPLETE)

ALL MERGED (PRs #39–#49, develop c8913cad): STORY-035, STORY-036, STORY-043, STORY-044, STORY-073, STORY-075, STORY-076, STORY-078, STORY-045, **STORY-077**.

Batch B (next, parallel): STORY-037→038→039→040, STORY-041→042.
Batch C (after Batch B): STORY-049→050.

---

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-06-03 |
| **Position** | Wave 4 Batch A 10/10 COMPLETE. STORY-077 merged as PR #49 (c8913cad). Next: Wave 4 gate OR Batch B — confirm with orchestrator. |
| **develop SHA** | `c8913cad` (49 merged PRs) |
| **Active worktrees** | None |
| **Open PRs** | 0 |
| **Workspace crates** | 16 |
| **BC deltas** | BC-3.02.002 v1.5, BC-1.14.003 v1.3, error-taxonomy v2.12 — all on factory-artifacts |
| **factory-artifacts** | Local only — push requires explicit human authorization |

---

## Follow-Ups / Drift (non-blocking)

| Item | Severity | Status | Notes |
|------|----------|--------|-------|
| STORY-081: slide-level inline markup | P0 blocker for v1.0 | draft | Wave 5, 13 pts, EPIC-18. Depends on STORY-077 (now merged). Not started. |
| SEC-001: veraPDF Docker `verapdf/cli:latest` not digest-pinned (CWE-494) | MED | open | CI-only; merged in #48; fix before v1.0 / Phase 6 |
| SEC-002: `[text](url)` link URL has no scheme validation — `javascript:`/`data:` stored verbatim | LOW | deferred | No HTML exporter yet; enforce at parse boundary (allowlist http/https/mailto, new E-PAR code) BEFORE STORY-046 (HTML exporter) ships. Concrete dependency = STORY-046. |
| SEC-003 (formerly SEC-002): `emu_to_pt` i64→f32 precision loss | LOW | open | Phase 6 Kani |
| SEC-004 (formerly SEC-003): CI tee predictable temp path (self-hosted only) | LOW | open | Phase 6 |
| E-EVL-007..011 unregistered + E-PAR-012 retired-code reuse | LOW | follow-up story drafted | See STORY-077 lessons. Taxonomy-completeness story needed before Phase 6. Recurred 3+ times in cascade — codified as follow-up. |
| OBS-077-P24-A: `_` italic flanking guard (open-only) | LOW | pending-intent | `_apply file_path here_` closes italic at word-internal `_`. Spec-conformant (DIR-077-002 §1 mandates NO flanking). Right-flanking symmetry is a UX enhancement; PO decision + STORY-081 candidate. |
| OBS-077-P25-A: parse→eval round-trip integration test consolidation | LOW | optional | AC-002 covered across two test layers; single source-to-eval round-trip would consolidate. Optional follow-up. |
| BC-1.14.001/002 still `subsystem: SS-TBD` | LOW | open | Fold into next spec-hygiene pass |
| OBS-P6-001: PDF exporter ignores `opts.strict`/warnings | LOW | open | Wave-gate concern post-STORY-045 |

---

## Standing Process Rules (reminders for fresh session)

| ID | Rule |
|----|------|
| LESSON-1 | Adversary dispatches MUST pass the ABSOLUTE worktree path (`--cwd /Users/jmagady/Dev/slideforge/.worktrees/STORY-NNN`). |
| LESSON-2 | Canonical clippy gate: `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`. Bare `-D warnings` misses pedantic lints. |
| LESSON-3 | Re-run full pre-push gate (fmt + pedantic clippy + nextest) after EVERY commit to a feature branch before PR. |
| LESSON-5 | pr-manager CANNOT spawn sub-agents. Orchestrator dispatches security-reviewer + pr-reviewer independently. |
| LESSON-6 | RELAXED by STANDING MERGE AUTH: orchestrator may merge when CI-green + security-reviewer CLEAN + pr-reviewer APPROVE. |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
