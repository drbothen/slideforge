---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-06-02
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
wave_4_batch_a_complete: 9
wave_4_batch_a_total: 10
wave_4_started: 2026-05-31
wave_4_total_stories: 18
wave_4_total_points: 114
wave_5_total_points: 109
develop_sha: "e5d818e7"
develop_pr_count: 48
workspace_tests: "~2700+ (48 PRs merged)"
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

Phase 3, Wave 4 Batch A = 9/10. ONLY STORY-077 remains.
- develop = `e5d818e7` (48 merged PRs). Open PRs: 0. Active worktrees: `.worktrees/STORY-077` only.
- This session merged STORY-045 (PR #48, PDF/UA-1+veraPDF) and STORY-075 (PR #47, footer detection).

---

## STANDING AUTHORIZATIONS

**STANDING MERGE AUTH (granted by human 2026-06-02):** The orchestrator MAY auto-merge any PR that is fully CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, WITHOUT asking the human each time (revocable by human). This RELAXES LESSON-6 for converged PRs. LESSON-6's "GitHub blocks author self-approve" means the orchestrator does `gh pr merge --squash` itself once those gates pass.

---

## STORY-077 — RESUME HERE (turnkey)

**Worktree:** `/Users/jmagady/Dev/slideforge/.worktrees/STORY-077`
**Branch:** `feature/S-077` @ **`6c72e686`** (LOCAL-ONLY, not pushed — correct, mid LOCAL adversary cascade)
**Story:** v1.5, 13 pts, P0, EPIC-18. SectionBlock IR extension + inline-markup parser (scope expanded by human 2026-06-02 per DIR-077-002).
**Crates touched:** slideforge-types + slideforge-syntax + slideforge-eval. ~32 layout call-site adjustments. Blocks STORY-041/042.
**BCs:** BC-3.02.002 v1.5, BC-1.14.003 v1.3.
**error-taxonomy:** v2.10 (E-EVL-012/013, E-PAR-019/020 registered this session).

### Cascade History — ALL FIXED (passes 1–5, full detail in burst-log.md)

Passes 1-4: Expr::Call unreachable; paper→genuine test; delimiter spans; E-EVL/E-PAR collision → E-EVL-012/013, E-PAR-019/020; kind-based sentinel routing. All fixed.

**Pass-5 fix (6c72e686):** `strip_custom_wrapper()` strips chumsky `Custom("…")` wrapper from all error reason strings; `InlineMarkupRoute` carries `original_msg`; `mod.rs` passes `original_msg` not the tagged blob. Bonus: same fix applies to E-PAR-012/013/014.

**Streak: 0/3 strict-CLEAN.**

### NEXT ACTION (pass 6)

Dispatch LOCAL adversary **pass 6** (`vsdd-factory:adversary`) with cwd = `/Users/jmagady/Dev/slideforge/.worktrees/STORY-077` @ `6c72e686`.

**Verify the F-077-P5-001 fix:** rendered SyntaxError messages contain NO `"SLIDEFORGE_INLINE_ROUTE"`, NO `"Custom("`, NO stray `"|"` sentinel; delimiter codes (E-PAR-019/020) and error messages are correct; `strip_custom_wrapper` did NOT regress other error messages.

**Continue cascade to 3/3 strict-CLEAN.** Then:
1. demo-recorder per-AC
2. Rebase `feature/S-077` onto `origin/develop` (`e5d818e7`) — expect clean (STORY-045 touches pdf + veraPDF; STORY-075 touches brand/footer; STORY-077 touches types/eval/syntax/layout; watch for a `slideforge-types/block.rs` vs `deck.rs` merge: STORY-045 added `ContentBlock::produces_structure_group` in `block.rs`, STORY-077 touches `deck.rs` — should be clean but verify)
3. Push (force-with-lease after rebase — feature branch, OK)
4. pr-manager 9-step PR targeting develop
5. Orchestrator dispatches independent security-reviewer + pr-reviewer (LESSON-5: pr-manager CANNOT spawn sub-agents)
6. Under STANDING MERGE AUTH: merge when CI-green + security-reviewer CLEAN + pr-reviewer APPROVE → `gh pr merge --squash`
7. state-manager post-merge burst
8. devops worktree cleanup

Wave 4 Batch A → 10/10.

---

## STORY-077 ADJUDICATIONS — binding, do NOT reopen

- Sentinel+hex serialization across the chumsky Rich (String-backed reason) boundary is a LEGITIMATE pragmatic pattern (typed custom reason = large cross-cutting refactor, out of scope); adversary pass-5 confirmed; hex encoding total, no misroute/collision risk.

---

## STORY-077 ADVERSARY OUT-OF-PERIMETER — do NOT report, do NOT reset streak

- math-chunk silent-drop in register sub-blocks (pre-existing project-wide behavior)
- OBS-077-P4-A: duplicate public struct name `SectionBlock` (`slideforge-types` IR vs `slideforge-plugin-api` trait) — pre-existing, namespaced-valid, STORY-041/042 concern
- slide-level inline-markup precise spans / `FieldValue::Inlines` conversion — deferred to STORY-081
- E-EVL-007..E-EVL-011 not registered in error-taxonomy — pre-existing taxonomy debt (follow-up)

---

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + arch (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 81 stories, 21 epics, 6 waves, 491 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. Wave 4 Batch A 9/10. STORY-077 pass 5 done (F-077-P5-001 FIXED @ 6c72e686); pass 6 NEXT; streak 0/3. | Per-story delivery |
| Phases 4–7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

## Wave 4 Batch A Status (9 MERGED / 1 IN PROGRESS)

MERGED (PRs #39–#48, develop e5d818e7): STORY-035, STORY-036, STORY-043, STORY-044, STORY-073, STORY-075, STORY-076, STORY-078, STORY-045.

IN PROGRESS: STORY-077 — feature/S-077 @ `6c72e686` (wt, not pushed). Adversary pass 6 NEXT. Streak 0/3.

---

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-06-02 |
| **Position** | Wave 4 Batch A 9/10. STORY-077 inline-markup adversary cascade: passes 1-5 done, F-077-P5-001 FIXED @ 6c72e686, pass 6 NEXT. Streak 0/3. |
| **develop SHA** | `e5d818e7` (48 merged PRs) — run `git fetch` before starting; local develop ref may be stale |
| **Active worktrees** | `.worktrees/STORY-077` only (feature/S-077 @ `6c72e686` — NOT pushed) |
| **Open PRs** | 0 |
| **Workspace crates** | 16 |
| **BC deltas (local)** | BC-2.01.001 v1.4, BC-3.02.002 v1.5, BC-3.05.001 v1.3.5, BC-1.14.003 v1.3, BC-4.03.001 v1.3, error-taxonomy v2.10 — all on factory-artifacts (local-only) |
| **factory-artifacts** | Local only — push requires explicit human authorization |

---

## Follow-Ups / Drift (non-blocking)

| Item | Severity | Notes |
|------|----------|-------|
| STORY-081: slide-level inline markup | P0 blocker for v1.0 | Wave 5, 13 pts, EPIC-18, draft. Depends on STORY-077. Not started. After STORY-077 merges. |
| SEC-001: veraPDF Docker `verapdf/cli:latest` not digest-pinned (CWE-494) | MED | CI-only; merged in #48; fix before v1.0 / Phase 6 |
| SEC-002: `emu_to_pt` i64→f32 precision loss | LOW | Phase 6 Kani |
| SEC-003: CI tee predictable temp path (self-hosted only) | LOW | Phase 6 |
| E-EVL-007..011 not in error-taxonomy | LOW | Taxonomy backfill; not a story blocker |
| BC-1.14.001/002 still `subsystem: SS-TBD` | LOW | Fold into next spec-hygiene pass |
| OBS-P6-001: PDF exporter ignores `opts.strict`/warnings | LOW | Wave-gate concern post-STORY-045 |

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

## Key Spec References for STORY-077

| Document | Notes |
|----------|-------|
| `.factory/cycles/STORY-077/inline-markup-directive.md` | DIR-077-002: inline-markup scope (BINDING) |
| `.factory/cycles/STORY-077/section-parse-directive.md` | DIR-077-001 + DIR-077-001-A: section-parse ownership (BINDING) |
| `.factory/cycles/STORY-077/chumsky-parse-warning-research.md` | chumsky 0.10.1 parse-time diagnostics verdict |
| `.factory/specs/behavioral-contracts/` | BC-3.02.002 v1.5, BC-1.14.003 v1.3 |
| `.factory/specs/prd-supplements/error-taxonomy.md` | v2.10: E-EVL-012/013, E-PAR-019/020 registered |
| `.factory/stories/stories/STORY-077-section-block-ir-extension.md` | Story v1.5 |
| `.factory/cycles/STORY-077/burst-log.md` | Full cascade history + STORY-045 obligations archive |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
