---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-05-31
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
total_stories: 77
total_points: 462
total_waves: 6
total_epics: 21
dtu_required: false
dtu_assessment: 2026-05-24
dtu_clones_built: n/a
dtu_services: []
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed"
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); #[non_exhaustive] hardening + slideforge-brand [workspace.dependencies] + conventions.md v1.3; adversary pass 8 strict-CLEAN; holdout must-pass 5/5"
wave_4_batch_a_complete: 1
wave_4_batch_a_total: 8
wave_4_started: 2026-05-31
wave_4_total_stories: 17
wave_4_total_points: 104
wave_4_batch_a: "STORY-035→036, STORY-043→044→045, STORY-073, STORY-075, STORY-076, STORY-077 (parallel)"
wave_4_batch_b: "STORY-037→038→039→040, STORY-041→042 (parallel, after Batch A)"
wave_4_batch_c: "STORY-049→050 (after Batch B)"
wave_4_new_p0: "STORY-077 (8pts SectionBlock IR extension) — architect-directed spin-out from STORY-035 F-002 descope; blocks STORY-041/042"
story035_status: "MERGED — PR #39, squash commit 0e7d9fde (2026-05-31). 10-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 8/9/10. Option D single-source eval-stage routing. AC-005 descoped → STORY-077."
develop_sha: "0e7d9fde"
develop_pr_count: 39
workspace_tests: "~2584 (full-suite run 2026-05-31 @ 584cbc6f; fix-PR #38 additive config+attrs only)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. Generates branded .pptx, .docx, .pdf, .html, and web preview from a single .sf DSL file with data binding, iteration, conditionals, and a plugin-first architecture.

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

## Session Resume Brief (2026-05-31 — Wave 4 Batch A IN PROGRESS)

### Where we are

Phase 3, Wave 4. Wave 3 COMPLETE + Gate PASSED (2026-05-31, 22 stories, 2584 tests GREEN, adversary 3-CLEAN passes 6/7/8, fix-PR #38 @ `7d266ad7`). **Wave 4 STARTED, Batch A 1/8 complete.**

**STORY-035 (Writing Register Routing) MERGED** — PR #39, squash `0e7d9fde`, 2026-05-31. 10-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 8/9/10. Option D single-source eval-stage routing (eval populates `Slide.register_content`; layout clones; slideforge-types duplicate method deleted). AC-005 section-node descoped → STORY-077 (EC-003). BC-1.14.004 added.

develop: `0e7d9fde` (39 merged PRs). 0 active feature worktrees. 0 open PRs.

**STORY-077 CREATED:** SectionBlock IR Extension (EPIC-18, P0, 8pts, Wave 4 Batch A). Architect-directed spin-out from STORY-035 F-002 descope. Anchors BC-3.02.002 + BC-1.14.003. Blocks STORY-041/042.

### Top next action

**Dispatch next Wave 4 Batch A story.** STORY-035 is done. STORY-036 (no-bleed invariant) is now unblocked — its only dep was STORY-035, now merged. Remaining Batch A (7 stories):

| Story | Title | Unblocked? |
|-------|-------|-----------|
| STORY-036 | No-Bleed Invariant | YES — dep STORY-035 merged |
| STORY-043 | PDF Core (scaffolds slideforge-pdf crate first task, moves from `[workspace] exclude` → `members`) | YES |
| STORY-044 | PDF Layout Integration | After STORY-043 |
| STORY-045 | PDF Export Pipeline | After STORY-044 |
| STORY-073 | Bullets Layout (P1 pull-in) | YES |
| STORY-075 | Footer Detection (P1 pull-in) | YES |
| STORY-076 | srgbClr Transform (P1 pull-in) | YES |
| STORY-077 | SectionBlock IR Extension (P0, blocks STORY-041/042) | YES |

### BC deltas in effect for Wave 4

BC-2.01.001 v1.2 (EC-006 srgbClr, Option B); BC-2.01.003 v1.9 (EC-003 widened); error-taxonomy v2.3.

### Process improvements (apply before each story)

Before accepting any "clippy clean": run `rustup update stable && cargo clippy --workspace --all-targets --all-features -- -D clippy::pedantic -D clippy::unwrap_used`. Add cross-crate compile check for stories touching slideforge-brand or slideforge-types.

---

## Current Status

Phase 3 IN PROGRESS. Wave 1 COMPLETE (gate PASSED). Wave 2 COMPLETE (gate PASSED). Wave 3 COMPLETE (22/22 stories, gate PASSED 2026-05-31). **Wave 4 STARTED — 17 stories / 104 pts, Batch A 1/8 complete (STORY-035 MERGED).**

develop: `0e7d9fde` (39 merged PRs, ~2584 tests, 0 failures). 0 active feature worktrees. 77 stories / 462 pts total.

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 decision docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + architecture (14 ADRs, 15 VPs, 20 crates) + UX spec. 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 77 stories, 21 epics, 6 waves, 462 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Wave 1: GATE PASSED. Wave 2: GATE PASSED. Wave 3: GATE PASSED 2026-05-31. **Wave 4 STARTED — Batch A 1/8 (STORY-035 MERGED).** | Per-story delivery |
| Phase 4: Holdout Evaluation | NOT STARTED | Per-wave holdout gates |
| Phase 5: Adversarial Refinement | NOT STARTED | Post-implementation cascade |
| Phase 6: Formal Hardening | NOT STARTED | Kani + fuzz + mutants + semgrep |
| Phase 7: Convergence | NOT STARTED | 7-dimension convergence assessment |

## Wave 4 Batch A Status

| Story | Title | Status | PR | Commit |
|-------|-------|--------|----|--------|
| STORY-035 | Writing Register Routing | MERGED | #39 | 0e7d9fde |
| STORY-036 | No-Bleed Invariant | NOT STARTED (unblocked) | — | — |
| STORY-043 | PDF Core (new crate) | NOT STARTED | — | — |
| STORY-044 | PDF Layout Integration | NOT STARTED (after 043) | — | — |
| STORY-045 | PDF Export Pipeline | NOT STARTED (after 044) | — | — |
| STORY-073 | Bullets Layout | NOT STARTED | — | — |
| STORY-075 | Footer Detection | NOT STARTED | — | — |
| STORY-076 | srgbClr Transform | NOT STARTED | — | — |
| STORY-077 | SectionBlock IR Extension (P0) | NOT STARTED | — | — |

## Decisions Log (milestones)

- 2026-05-23 — Workspace resolved, mode: greenfield. Market intelligence: GO.
- 2026-05-24 — ALL 25 DSL DESIGN QUESTIONS COMPLETE. ALL 7/7 SPIKES RESOLVED. PRD COMPLETE (109 BCs). ARCHITECTURE COMPLETE (14 ADRs, 15 VPs, 20 crates). UX SPEC COMPLETE.
- 2026-05-25 — PHASE 1 CONVERGED (17 passes, 69 findings, 3/3 clean) + APPROVED. PHASE 2 CONVERGED (22 passes, 96+ findings, 3/3 clean) + APPROVED. PHASE 3 STARTED.
- 2026-05-26 — Wave 1: STORIES 001–010 + 051/052/053/054 MERGED (14 stories).
- 2026-05-27 — WAVE 1 GATE PASSED. WAVE 2: all 7 stories MERGED (PRs #14–#20). WAVE 2 GATE PASSED — 11 gate passes, 19 findings fixed, 3/3 clean.
- 2026-05-27/28 — WAVE 3 BATCH 1 COMPLETE — 6 new crates (data, brand, layout, math, charts, diagrams), 494 new tests, PRs #21–#26.
- 2026-05-28/30 — WAVE 3 BATCH 2 COMPLETE — 13 stories MERGED (PRs #27–#34, commits 7bc71f9c → 066d625f). Notable: STORY-020 29-pass adversary; STORY-028 32-pass adversary.
- 2026-05-30 — WAVE 3 BATCH 3 COMPLETE — STORY-021 (PR #35, 362c4a1f), STORY-024 (PR #36, 924cdc04), STORY-025 (PR #37, 584cbc6f) ALL MERGED.
- 2026-05-31 — WAVE 3 GATE PASSED — 2584 tests GREEN; holdout must-pass 5/5; adversary 8 passes 3-CLEAN (6/7/8); fix-PR #38 (7d266ad7). WAVE 3 COMPLETE.
- 2026-05-31 — WAVE 4 STARTED — 17 stories / 104 pts (STORY-077 added P0; STORY-073/075/076 pulled in P1; STORY-072/074 deferred Wave 5). BC-2.01.001 v1.2, BC-2.01.003 v1.9, error-taxonomy v2.3.
- 2026-05-31 — STORY-035 MERGED (PR #39, 0e7d9fde) — Writing Register Routing. 10-pass adversary cascade, 3/3 strict-CLEAN (passes 8/9/10). Option D single-source eval-stage routing. AC-005 descoped → STORY-077. Wave 4 Batch A: 1/8 complete.
- 2026-05-31 — STORY-077 CREATED — SectionBlock IR Extension (architect-directed spin-out, EPIC-18, P0, 8pts). Blocks STORY-041/042. Project total: 77 stories / 462 pts.

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-05-31 |
| **Position** | Phase 3, Wave 4 Batch A — 1/8 complete. STORY-035 MERGED. Top action: dispatch next Batch A story (STORY-036 unblocked). |
| **develop SHA** | 0e7d9fde (39 merged PRs) |
| **Workspace tests** | ~2584 (full-suite 2026-05-31 @ 584cbc6f; fix-PR #38 additive only) |
| **Workspace crates** | 13 (7 Wave 1 + 6 Batch 1: data, brand, layout, math, charts, diagrams) |
| **Active worktrees** | 0 |
| **Open PRs** | 0 |
| **STORY-035** | MERGED — PR #39, 0e7d9fde. 10-pass adversary, 3/3 strict-CLEAN (P8/9/10). Option D routing. AC-005 → STORY-077. |
| **STORY-077** | Created 2026-05-31. SectionBlock IR Extension, EPIC-18, P0, 8pts, Batch A. Blocks STORY-041/042. |
| **Wave 4 Batch A remaining** | STORY-036 (unblocked), STORY-043→044→045 (pdf crate), STORY-073, STORY-075, STORY-076, STORY-077 |
| **STORY-043 prerequisite** | Scaffold slideforge-pdf crate + move from `[workspace] exclude` → `members` as first task. |
| **BC deltas in effect** | BC-2.01.001 v1.2 (EC-006 srgbClr, Option B); BC-2.01.003 v1.9 (EC-003 widened); error-taxonomy v2.3. |
| **factory-artifacts** | Local only. Push requires explicit human authorization per CLAUDE.md. |
| **Holdout caveat** | Full mean-satisfaction holdout (≥0.85) deferred to post-exporter waves; 9/15 scenarios blocked by missing CLI + exporters. |
| **Archived history** | Verbose per-merge decisions log + STORY-035 "in flight" checkpoint → .factory/cycles/STORY-035/burst-log.md + session-checkpoints.md |

## Per-Story Delivery Flow (reference)

1. Create worktree: `git worktree add .worktrees/STORY-NNN -b feature/S-NNN develop`
2. test-writer: stubs + failing tests (Red Gate — tests must FAIL before implementer starts)
3. implementer: TDD (make tests pass, zero `.unwrap()`, clippy::pedantic clean)
4. adversary: 3 consecutive strict-CLEAN passes (BC-5.39.001)
5. `git push origin feature/S-NNN` → PR targeting `develop` → CI (17 checks) → squash-merge → state update
6. Remove worktree: `git worktree remove .worktrees/STORY-NNN`

## Key Spec References

| Document | Scope |
|----------|-------|
| .factory/specs/behavioral-contracts/ | 109 BCs organized by section |
| .factory/specs/architecture/ARCH-INDEX.md | 12 architecture sections, 14 ADRs |
| .factory/specs/prd.md | PRD + 109 BCs + 15 holdout scenarios |
| .factory/stories/wave-schedule.md | 6 waves, batching, dependency order |
| .factory/stories/STORY-INDEX.md | 77 stories with status |
| .factory/stories/sprint-state.yaml | Current sprint/wave state |
| .factory/planning/q1-decision-final.md | Computation, formats, registers, charts, math, brand, roadmap |
| .factory/planning/q3-decision-final.md | Plugin-first architecture, 10 surfaces, trait signatures |

## Quality Bar (Non-Negotiable)

Production-grade from day 1. Key enforced gates:
- `#![forbid(unsafe_code)]` on all crates; zero `.unwrap()` outside tests; `clippy::pedantic` clean; `#![warn(missing_docs)]` on public APIs
- Kani proofs for pure-core functions (Phase 6); `cargo-fuzz` harness; `cargo-mutants` with documented kill-rate budget
- WCAG AA: web preview + HTML; PDF/UA-1 on PDF; OOXML accessibility linter on PPTX
- < 500ms cold build, < 50ms incremental (25-slide deck) — CI benchmark gate
- Signed releases + SBOM; `cargo audit` + `cargo deny` in CI; semgrep/CodeQL per PR
- Cross-platform: macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64
- Full quality bar table: CLAUDE.md

## Drift Items

| Date | Item | Severity | Notes |
|------|------|----------|-------|
| 2026-05-28 | LOCAL adversary 3-CLEAN on STORY-034 ran macOS-only, missed Linux Trebuchet MS substitution failure (BC-1.12.001 violation) | LOW | Required 4 CI iterations. Process-gap candidate: should LOCAL adversary spawn a Linux-container test pass for font/text/SVG/rendering stories? Surface to user for codification decision. |
