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
- STORY-077 LOCAL adversary cascade: passes 1–21 done, all fixed. HEAD `9c862fef`. Pass 22 NEXT. Streak 0/3.

---

## STANDING AUTHORIZATIONS

**STANDING MERGE AUTH (granted by human 2026-06-02):** The orchestrator MAY auto-merge any PR that is fully CI-green + security-reviewer CLEAN + pr-reviewer APPROVE, WITHOUT asking the human each time (revocable by human). This RELAXES LESSON-6 for converged PRs. LESSON-6's "GitHub blocks author self-approve" means the orchestrator does `gh pr merge --squash` itself once those gates pass.

---

## STORY-077 — RESUME HERE (turnkey)

**Worktree:** `/Users/jmagady/Dev/slideforge/.worktrees/STORY-077`
**Branch:** `feature/S-077` @ **`9c862fef`** (LOCAL-ONLY, not pushed — correct, mid LOCAL adversary cascade)
**Story:** v1.5, 13 pts, P0, EPIC-18. SectionBlock IR extension + inline-markup parser (scope expanded by human 2026-06-02 per DIR-077-002).
**Crates touched:** slideforge-types + slideforge-syntax + slideforge-eval. ~32 layout call-site adjustments. Blocks STORY-041/042.
**BCs:** BC-3.02.002 v1.5, BC-1.14.003 v1.3.
**error-taxonomy:** v2.12 (E-PAR-019/020/021 + E-EVL-012/013/014 registered this cascade).

### Cascade Position (passes 1–21 complete — full detail in burst-log.md)

Passes 1–21 all fixed. Key cascade summary: UTF-8 char-boundary panic (P7, CRIT), unbounded recursion → E-PAR-021 depth cap (P7), section register_content dropped at layout boundary (P10, HIGH), E-PAR-019/020/021 non-fatal → fatal (P14, HIGH), slide-level inline-markup chunks silently dropped (P17, HIGH, streak reset), unclosed-link emitted no error (P21, MED, streak reset).

**Streak: 0/3 strict-CLEAN** (reset by P21 F-077-P21-001, fixed at 9c862fef).
**Build at last verified-green (2dbce955):** 2595 passed / 3 skipped (1 pre-existing perf flake `test_cold_budget_under_200ms` in slideforge-diagrams). Commit 9c862fef adds 4 tests (unclosed-link): 2598/2599 (same flake).

### NEXT ACTION

Resume LOCAL adversary cascade at **pass 22** (`vsdd-factory:adversary`), cwd = `/Users/jmagady/Dev/slideforge/.worktrees/STORY-077` @ `9c862fef`. Continue to 3/3 strict-CLEAN. Then:
1. demo-recorder per-AC
2. Rebase `feature/S-077` onto `origin/develop` (`e5d818e7`) + push (force-with-lease)
3. pr-manager 9-step PR targeting develop
4. Orchestrator dispatches independent security-reviewer + pr-reviewer (LESSON-5)
5. Under STANDING MERGE AUTH: merge when CI-green + security-reviewer CLEAN + pr-reviewer APPROVE
6. state-manager post-merge burst; devops worktree cleanup

Wave 4 Batch A → 10/10.

---

## STORY-077 ADJUDICATIONS — binding, do NOT reopen

- Sentinel+hex serialization across the chumsky Rich boundary is LEGITIMATE (typed custom reason = large cross-cutting refactor, out of scope).
- F-077-P10-001 GeneratedSection.register_content propagation IN-SCOPE (applies existing LaidOutSlide.register_content slide-path pattern; no new architecture decision).
- E-PAR-019/020/021 are strict-build-fatal (accumulated AND fatal; `--warn-only` demotion is a future global concern; consistent with E-PAR-017).
- F-077-P17-001 slide-level flat-text preservation IN-SCOPE (DIR-077-002 §4 mandates `Value::Str` flat text); structural InlineNode upgrade deferred to STORY-081.
- F-077-P18-001 markup-wrapped brand-ref preserved in SET-RULES ONLY (slide/vars brand-ref preservation is NOT existing behavior; out of scope, STORY-081/PO).

---

## STORY-077 ADVERSARY OUT-OF-PERIMETER — do NOT report, do NOT reset streak

- math-chunk silent-skip in eval (MathInline/MathDisplay/MathInterp) — pre-existing STORY-009
- OBS-077-P4-A: duplicate public struct name `SectionBlock` (IR vs trait) — pre-existing, namespaced-valid, STORY-041/042
- slide-level inline-markup STRUCTURAL upgrade (`FieldValue::Inlines`) — STORY-081
- E-EVL-007..E-EVL-011 not registered in error-taxonomy — pre-existing taxonomy debt (follow-up)
- F-077-P6-002 / E-PAR-012 retired-code reuse — taxonomy-reconciliation follow-up
- Zero-origin spans in section-eval (`SourceSpan::default()`) — documented pre-existing eval limitation
- OBS-P8-B `FieldValue::Shape` arm structurally-unreachable (proven)
- F-077-P11-003 Custom-map Null placeholders for detail/report keys — STORY-041/042
- OBS-077-P21-A SSOT test uses local literal copy (LOW, pre-existing F-077-P2-002 lineage)

---

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + arch (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 81 stories, 21 epics, 6 waves, 491 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Waves 1/2/3 GATE PASSED. Wave 4 Batch A 9/10. STORY-077 passes 1–21 done; pass 22 NEXT; streak 0/3 (P21 reset); HEAD 9c862fef. | Per-story delivery |
| Phases 4–7 | NOT STARTED | Holdout / Adversarial / Formal Hardening / Convergence |

## Wave 4 Batch A Status (9 MERGED / 1 IN PROGRESS)

MERGED (PRs #39–#48, develop e5d818e7): STORY-035, STORY-036, STORY-043, STORY-044, STORY-073, STORY-075, STORY-076, STORY-078, STORY-045.

IN PROGRESS: STORY-077 — feature/S-077 @ `9c862fef` (wt, not pushed). Adversary pass 22 NEXT. Streak 0/3 (reset by P21).

---

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-06-02 |
| **Position** | Wave 4 Batch A 9/10. STORY-077 inline-markup adversary cascade: passes 1–21 done, all fixed, HEAD 9c862fef. Streak 0/3 (reset by P21 F-077-P21-001 unclosed-link). Pass 22 NEXT. |
| **develop SHA** | `e5d818e7` (48 merged PRs) — run `git fetch` before starting; local develop ref may be stale |
| **Active worktrees** | `.worktrees/STORY-077` only (feature/S-077 @ `9c862fef` — NOT pushed) |
| **Open PRs** | 0 |
| **Workspace crates** | 16 |
| **BC deltas (local)** | BC-2.01.001 v1.4, BC-3.02.002 v1.5, BC-3.05.001 v1.3.5, BC-1.14.003 v1.3, BC-4.03.001 v1.3, error-taxonomy v2.12 — all on factory-artifacts (local-only) |
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
| LESSON: clean pass 16 missed HIGH (P17 text loss); fresh-context pass 17 caught it | LOW | Validates strict 3-CLEAN + fresh-context protocol; no process change needed |
| Follow-up story: register E-EVL-007..011 + reconcile E-PAR-012 retired-code reuse | LOW | Taxonomy-completeness debt; surfaced repeatedly in STORY-077 cascade |
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
| `.factory/specs/prd-supplements/error-taxonomy.md` | v2.12: E-PAR-019/020/021 + E-EVL-012/013/014 registered |
| `.factory/stories/stories/STORY-077-section-block-ir-extension.md` | Story v1.5 |
| `.factory/cycles/STORY-077/burst-log.md` | Full cascade history + STORY-045 obligations archive |

---

## Quality Bar (Non-Negotiable Gates)

Production-grade from day 1. Full table in CLAUDE.md. `#![forbid(unsafe_code)]`; zero `.unwrap()` outside tests; `clippy::pedantic`; `#![warn(missing_docs)]`; Kani+fuzz+mutants (Phase 6); WCAG AA; PDF/UA-1; < 500ms cold build; signed releases; SBOM; cross-platform macOS+Linux+Windows.
