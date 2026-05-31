---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-05-31
wave_3_gate_fix_pr: "PR #38 squash-merged 7d266ad7 2026-05-31"
wave_3_batch_3_story024_merged: "PR #36, 924cdc04, 2026-05-30"
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
wave_1_gate: "PASS 2026-05-27 — 3 gate passes, 11 findings fixed (keyword sync, field sync, crate attrs)"
wave_1_completed: 2026-05-27
wave_2_gate: "PASS 2026-05-27 — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)"
wave_2_completed: 2026-05-27
wave_3_batch_1_completed: 2026-05-28
wave_3_batch_2_completed: 2026-05-30
wave_3_batch_3_started: 2026-05-30
wave_3_batch_3_completed: 2026-05-30
wave_3_batch_3_stories: "STORY-021 (PR #35, 362c4a1f) + STORY-024 (PR #36, 924cdc04) + STORY-025 (PR #37, 584cbc6f) — ALL MERGED"
wave_3_gate_test_suite: "GREEN @ 584cbc6f — 0 failures (~2584 tests, full cargo test --workspace --no-fail-fast 2026-05-31); fix-PR #38 (7d266ad7) additive only"
wave_3_gate_holdout: "must-pass PASS (5/5 ≥0.60; SSRF/HS-013 0.90); mean 0.767 over 6 evaluable scenarios (9 blocked — no CLI/exporters yet)"
wave_3_gate_adversarial: "CONVERGED — 8 passes, strict-CLEAN at passes 6/7/8; fix-PR #38 merged 7d266ad7"
wave_3_gate_status: "PASSED 2026-05-31 — test suite GREEN 2584 tests 0 failures; holdout must-pass 5/5; adversary 8 passes 3-CLEAN (6/7/8); fix-PR #38 squash-merged 7d266ad7"
wave_3_completed: 2026-05-31
wave_3_gate: "PASSED 2026-05-31 — PR #38 (7d266ad7); #[non_exhaustive] hardening + slideforge-brand [workspace.dependencies] + conventions.md v1.3; adversary pass 8 strict-CLEAN; holdout must-pass 5/5 PASS"
wave_4_started: 2026-05-31
wave_4_total_stories: 17
wave_4_total_points: 104
wave_4_batch_a: "STORY-035→036, STORY-043→044→045, STORY-073, STORY-075, STORY-076, STORY-077 (parallel)"
wave_4_batch_b: "STORY-037→038→039→040, STORY-041→042 (parallel, after Batch A)"
wave_4_batch_c: "STORY-049→050 (after Batch B)"
wave_4_pulled_in_p1: "STORY-073 (5pts bullets layout), STORY-075 (3pts footer detection), STORY-076 (3pts srgbClr transform)"
wave_4_new_p0: "STORY-077 (8pts SectionBlock IR extension) — architect-directed spin-out from STORY-035 F-002 descope; blocks STORY-041/042"
wave_4_deferred_p2: "STORY-072 → Wave 5 (gradient fills), STORY-074 → Wave 5 (brand-em-sizing)"
wave_4_bc_deltas: "BC-2.01.001 v1.1→v1.2 (EC-006 srgbClr transform, Option B); BC-2.01.003 v1.8→v1.9 (EC-003 widened); error-taxonomy v2.2→v2.3 (no new variant)"
story035_status: "IN_PROGRESS — Red Gate 3392b598 + Green 4f8a9487 + adversary LOCAL Pass 1 (2 HIGH/2 MED/1 LOW, streak 0/3) + fix-burst 1b4f654e. 743/743 GREEN, clippy/fmt clean. Adversary Pass 2 in flight. F-001 Option D directive issued; AC-005 section-node descoped to STORY-077; EC-003 DESCOPED; BC-1.14.004 added to spec."
develop_sha: "7d266ad7"
develop_pr_count: 38
workspace_tests: "~2584 (full-suite run 2026-05-31 @ 584cbc6f pre-fix-PR; fix-PR #38 is additive #[non_exhaustive] + dep config only)"
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. It generates branded .pptx, .docx, .pdf, .html, and a live web preview from a single indentation-significant DSL (.sf files) with data binding, iteration, conditionals, and a plugin-first architecture.

**Tagline:** "Branded documents from structured data — one source, every format."

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

## Session Resume Brief (2026-05-31 — Wave 4 Batch A IN PROGRESS)

### Fresh Session Resume Checklist

Before doing ANY work, a new session should:

1. **Sync local develop:** `git pull origin develop`. develop is at `7d266ad7` (38 merged PRs, ~2584 tests).
2. **Verify worktree health:** `git worktree list` should show: main worktree + `.factory/` + `.worktrees/STORY-035` (feature/S-035 in progress).
3. **Top priority on resume:** **STORY-035 adversary Pass 2** — verdict pending (streak 0/3; Pass 2 in flight). Then continue Batch A: STORY-036, STORY-043→044→045, STORY-073, STORY-075, STORY-076, STORY-077. See wave-schedule.md for story specs.
4. **STORY-043 prerequisite:** slideforge-pdf crate must be scaffolded and moved from `[workspace] exclude` → `members` as the FIRST task of STORY-043 delivery. slideforge-html stays excluded until Wave 5/STORY-046.
5. **BC deltas in effect for Wave 4:** BC-2.01.001 v1.2 (EC-006 srgbClr transform handling, Option B); BC-2.01.003 v1.9 (EC-003 widened to srgbClr); error-taxonomy v2.3. Story specs for STORY-075/076 already reference these versions.

### Where we are

Phase 3, Wave 3 — **COMPLETE.** Wave 4 — **STARTED, Batch A IN PROGRESS.** All 22 Wave 3 stories merged; Wave 3 Gate PASSED (2584 tests GREEN; holdout must-pass 5/5; adversary 8 passes strict-CLEAN 6/7/8; fix-PR #38 @ `7d266ad7`). Wave 4: 17 stories / 104 pts (expanded from 16/96: STORY-077 added per architect directive). STORY-073/075/076 pulled in as P1; STORY-072/074 deferred to Wave 5 as P2; STORY-077 added P0. BC-2.01.001 v1.2 + BC-2.01.003 v1.9 + error-taxonomy v2.3 landed. **STORY-035 adversary Pass 2 in flight (streak 0/3).**

### Wave 4 Batch Plan (human-approved 2026-05-31)

Wave 4: 17 stories / 104 pts. STORY-073/075/076 pulled in as P1; STORY-072/074 deferred to Wave 5. STORY-077 added as architect-directed P0 spin-out.

- **Batch A (parallel):** STORY-035→036, STORY-043→044→045 (slideforge-pdf NEW CRATE), STORY-073, STORY-075, STORY-076, STORY-077
- **Batch B (parallel, after Batch A gate):** STORY-037→038→039→040 (PPTX), STORY-041→042 (DOCX; blocked on STORY-077)
- **Batch C (after Batch B gate):** STORY-049→050 (plugin registry + E2E)

**BC deltas in effect:** BC-2.01.001 v1.2 (EC-006 srgbClr, Option B); BC-2.01.003 v1.9 (EC-003 widened); error-taxonomy v2.3.
**STORY-043 prerequisite:** scaffold slideforge-pdf + move from `[workspace] exclude` → `members` as first task.

### Top 3 next actions (in order)

1. **Complete STORY-035 adversary cascade:** Pass 2 in flight (streak 0/3). Need 3 consecutive clean passes. Fix-burst 1b4f654e closed Pass 1 findings (F-001 Option D implemented; F-002 AC-005/EC-003 descoped). Await Pass 2 verdict.
2. **Continue Batch A in parallel:** Dispatch remaining stories (STORY-036, STORY-043→044→045, STORY-073, STORY-075, STORY-076, STORY-077). STORY-043 starts with slideforge-pdf workspace scaffolding. STORY-077 is prerequisite for STORY-041/042.
3. **Process improvements (apply now):** Add `-D clippy::pedantic -D clippy::unwrap_used` to `just check` Justfile target. Add cross-crate compile check to per-story-delivery flow for stories touching slideforge-brand or slideforge-types.

### Orchestrator Playbook Improvements (codify before next cascade)

From the STORY-020 (29 passes) + STORY-028 (32 passes) + STORY-021 (23 passes) marathon cascades:

1. **BC version bump auto-sweep**: When PO bumps a BC version, the orchestrator should automatically dispatch:
   - implementer for code-comment sweep (grep crates/ for "BC-X.YY.NNN vX.Y.Z" stale refs)
   - story-writer for story-spec body sweep (grep .factory/stories/ for stale refs)
   - architect for VP-INDEX propagation if VPs changed
   Before declaring the BC bump fix-burst complete.

2. **Implementer commit-SHA verification**: When implementer reports N file changes, orchestrator should verify against `git log --stat HEAD` before accepting closure. Pass-27 had implementer claim 33 renames in 4 files; only 8 in 1 file actually shipped. TD-VSDD-059 paper-fix at agent-process level.

3. **Scope creep detection**: When implementer touches files OUTSIDE the story's perimeter (frontmatter behavioral_contracts), surface for orchestrator authorization BEFORE the fix-burst. Pass-27 partial-fix of cross-story phantom-BC pattern is the example.

4. **Convergence asymptote awareness**: After ~10 passes with only LOW findings, the orchestrator should explicitly check in with the human about strict-CLEAN vs PR-merge-CLEAN gate selection. Per BC-5.39.001, PR-merge is the canonical merge gate; strict 3-CLEAN is the cascade-closure criterion. Each pass 1-3 new LOW findings via fresh-context.

5. **Documentation sweep on green-phase**: When implementer closes a Red Gate (todo!() → real implementation), the same commit should remove the "/// Red Gate: panics with todo!()" doc lines. 78 stale Red Gate doc lines were found in STORY-028 across 3 files.

6. **Display-vs-protocol coupling (STORY-021 Pass 15 structural fix)**: Never embed user-facing Display strings in inter-layer error protocol formats. Source layer emits canonical machine-friendly format; dispatcher/consumer layer constructs rich Display. Any cross-layer message parsing that calls `.to_string()` on an error enum variant is a smell — flag for adversary inspection.

7. **CI clippy version drift (STORY-021)**: Local clippy clean with `-D warnings` does NOT equal CI clean with `-D clippy::pedantic -D clippy::unwrap_used`. `just check` must mirror CI clippy flags exactly. Include the CI clippy invocation (with all pedantic flags) in the per-story-delivery pre-push checklist.

8. **Spec-entity retirement parity (STORY-021 Pass 13→14)**: When code retires or deprecates a spec entity (e.g., E-DAT-014), the matching spec update (error-taxonomy.md, BC file) MUST be in the same fix burst. "Constant retained for SemVer compat" is not a valid justification for a missing spec update — surface the tension explicitly.

9. **#[non_exhaustive] default for public enums**: All public enums in slideforge-* crates should default to #[non_exhaustive] to prevent SemVer breakage on variant additions. Adversary should flag any new public enum lacking this attribute.

### Task state (TaskList does not persist — captured here)

Completed in 2026-05-28/30 session:
- Fix PR #30 STORY-034 conflict + merge (4 CI iterations: rebase + Linux font_resolver + Trebuchet MS via freefont/usvg fallback)
- STORY-030 Pass 10-17 adversary cycles + 4 fix bursts + PR #31 merge (1 CI iteration)
- STORY-023 Pass 11-20 adversary cycles + 5 fix bursts + PR #32 MERGED (dd6054c1)
- STORY-020 29-pass LOCAL adversary cascade, 3-CLEAN passes 27-28-29; PR #33 MERGED (143f1b78)
- STORY-028 32-pass LOCAL adversary cascade, 3-CLEAN passes 30-31-32; PR #34 MERGED (066d625f); follow-ups STORY-072/073/074 created
- STORY-021 23-pass LOCAL adversary cascade, 3-CLEAN passes 21-22-23; ~80 findings; 10 ACs demo'd (30 files); PR #35 MERGED (362c4a1f)
- STORY-024 Red Gate complete: `b61b0d04` on feature/S-024 — 15 failing tests covering 10 ACs; BrandExtractor stub in extractor.rs
- STORY-024 implementer complete — 211 slideforge-brand tests passing
- STORY-024 LOCAL adversary cascade: 11 passes, 3/3 strict-CLEAN (P9-10-11). BC-2.01.003 v1.8; E-BRD-006; VP-051+052; STORY-075+076 created
- STORY-025 Red Gate complete: `27d67613` on feature/S-025 — 9 should_panic + 12 structural failing tests; SlideOverlay implemented; resolve_overlay + infer_media_type stubbed; 42 Slide constructions updated (sibling sweep)
- STORY-024 demos (11 ACs) + push + PR #36 + merge (924cdc04). Wave 3 Batch 3: 2/3 merged.
- STORY-025 push feature/S-025 + 9-step PR cycle (pr-manager) → squash-merge → PR #37 → 584cbc6f. Wave 3 Batch 3: 3/3 COMPLETE.

Pending:
- Wave 4 planning (wave-schedule.md — story list, prerequisites, batching)
- STORY-072/073/074/075/076 wave assignment (all draft, Wave TBD — schedule in Wave 4+ planning)
- Justfile `just check` update: add `-D clippy::pedantic -D clippy::unwrap_used` to match CI

### Lessons captured (archived to cycle files — see below)

21 lessons captured across Batch 2 + STORY-021. Archived to cycle files per content routing rules.

- Batch 2 lessons (12 entries): `.factory/cycles/STORY-028/lessons.md`
- STORY-021 lessons (7 entries + structural changes table): `.factory/cycles/STORY-021/lessons.md`

**Top 3 actionable lessons for next session (kept inline for pickup):**

1. **Display-vs-protocol coupling (STORY-021 P15 — highest impact):** Source layer emits canonical `[E-DAT-NNN] HTTP {status} from '{url}'` format. Dispatcher constructs rich Display variant with `--offline` hint. Never mix user-facing Display with inter-layer machine-readable protocol. Apply to all new cross-layer error emission.

2. **CI clippy version drift:** `just check` must use `-D clippy::pedantic -D clippy::unwrap_used` to match CI. Run `rustup update stable` before any pre-push lint claim. Run `cargo clippy --fix` to auto-fix doc_markdown / uninlined_format_args / unnecessary_literal_bound.

3. **Spec-entity retirement parity:** When code retires a spec entity (E-DAT-NNN constant, error variant), the spec update (error-taxonomy.md, BC file) MUST be in the same fix burst.

---

## Current Status

Phase 3 IN PROGRESS. Wave 1 COMPLETE (14/14, gate PASSED). Wave 2 COMPLETE (7/7, gate PASSED). Wave 3 COMPLETE (22/22 stories, gate PASSED 2026-05-31). **Wave 4 STARTED — 17 stories / 104 pts, human-approved Batch A/B/C plan.** Batch A in progress.

develop branch: `7d266ad7` (38 merged PRs, ~2584 tests, 0 failures). 1 active feature worktree (STORY-035 in .worktrees/STORY-035, adversary Pass 2 in flight). 77 stories / 462 pts total.

**STORY-035 IN PROGRESS:** Red Gate (3392b598) + Green (4f8a9487) + LOCAL adversary Pass 1 (2 HIGH/2 MED/1 LOW) + fix-burst (1b4f654e). 743/743 GREEN, clippy/fmt clean. Adversary Pass 2 running now (streak 0/3). Architect directive issued: F-001→Option D (eval populates Slide.register_content; layout clones; delete slideforge-types method); F-002→AC-005 section-node descoped + EC-003 DESCOPED → STORY-077. BC-1.14.004 added to spec.

**STORY-077 CREATED:** SectionBlock IR Extension (EPIC-18, P0, 8pts, Wave 4 Batch A). Architect-directed spin-out from STORY-035 F-002 descope. Anchors BC-3.02.002 + BC-1.14.003. Blocks STORY-041/042.

## Wave 3 Batch 2 Story Status

| Story | Title | Status | PR | Commit | Notes |
|-------|-------|--------|----|--------|-------|
| STORY-019 | HTTP/HTTPS DataSource + SSRF | MERGED | #27 | 7bc71f9c | 9-pass adversary, 3/3 clean |
| STORY-032 | Chart Empty Data Placeholder | MERGED | #28 | 5ad267be | 7-pass adversary, 3/3 clean |
| STORY-027 | Layout: DOCX Section Generation | MERGED | #29 | 7641d4ea | 8-pass adversary, 3/3 clean |
| STORY-023 | Brand Synthesis: brand.toml → 31 Layouts | MERGED | #32 | dd6054c1 | 20-pass LOCAL adversary 3/3 clean (P18-20); PR-level 2 cycles, CLEAN at cycle 2; F1/F2 fixed in 4f78aa1c |
| STORY-030 | Math MathML + PDF Paths | MERGED | #31 | 19e79696 | 9 adversary iterations (Pass 9-17) incl. 2 paper-fix corrections (TD-VSDD-059): font-engine refactor (ab_glyph 0.2.31 + embedded LM Math 733KB OTF); paper-fix detection Pass 11 cache (OnceLock dead code); 3/3 CLEAN Pass 15/16/17 |
| STORY-034 | SVG Normalization via usvg | MERGED | #30 | 0cb4b982 | 4 CI iterations (force-push rebase + usvg font_resolver fix for Linux Trebuchet MS substitution) |
| STORY-020 | DataSource: Excel + SQLite | MERGED | #33 | 143f1b78 | 29-pass adversary, 3/3 CLEAN (P27-28-29); 251 tests + 2 perf_smoke; 23 ACs demo'd |
| STORY-028 | Layout: shape: Block + Rich Inline | MERGED | #34 | 066d625f | 32-pass adversary, 3/3 CLEAN (P30-31-32); 309 layout tests; follow-ups STORY-072/073/074 |

## Wave 3 Batch 3 Story Status (COMPLETE — all 3 merged)

| Story | Title | Status | PR | Commit | Notes |
|-------|-------|--------|----|--------|-------|
| STORY-021 | DataSource: HTTP cache / E-DAT policy errors | MERGED | #35 | 362c4a1f | 23-pass adversary, 3/3 CLEAN (P21-22-23); ~80 findings, 20 fix-burst commits; PolicyRejected + label-driven routing; Display-vs-protocol structural fix (P15); 2282 workspace tests post-merge |
| STORY-024 | Brand Extraction CLI (library-only, BC-2.01.003) | MERGED | #36 | 924cdc04 | 11-pass adversary 3/3 strict-CLEAN (P9-10-11); library-only; 211 slideforge-brand tests; 11 ACs demo'd; E-BRD-006, VP-051/052, BC v1.8; follow-ups STORY-075 (footer detection) + STORY-076 (transform-aware extraction). AI PR-diff review PR-merge-CLEAN (2 LOW/OBS non-blocking). Security CLEAN. |
| STORY-025 | Per-Slide brand_overlay invariant (BC-2.02.001+002) | MERGED | #37 | 584cbc6f | 11-pass adversary 3/3 strict-CLEAN (P9-10-11); 455 slideforge-brand+types tests; 10 ACs demo'd; HIGH path-traversal guard (E-BRD-007); BC-2.02.001 v1.4 / BC-2.02.002 v1.2 / error-taxonomy v2.2. AI PR-diff review PR-merge-CLEAN (1 LOW + 4 OBS non-blocking). Security CLEAN. |

## What to Do Next

- **STORY-035**: Await adversary Pass 2 verdict (streak 0/3). If CLEAN: continue cascade. If findings: fix-burst + Pass 3. Need 3 consecutive strict-CLEAN for closure.
- **Continue Wave 4 Batch A**: Deliver remaining parallel stories (STORY-036, STORY-043→044→045, STORY-073, STORY-075, STORY-076, STORY-077). STORY-043 starts with slideforge-pdf workspace scaffolding.
- **Process configuration**: Add `-D clippy::pedantic -D clippy::unwrap_used` to `just check` target in Justfile. Add cross-crate compile check for stories touching slideforge-brand or slideforge-types.
- **Holdout caveat (standing)**: Full mean-satisfaction holdout (≥ 0.85) deferred to post-exporter waves. Re-run after CLI + PPTX/HTML exporters ship.

**Key file references:**
- Wave schedule + batching: `.factory/stories/wave-schedule.md`
- Story specs: `.factory/stories/stories/STORY-NNN-*.md`
- Story index: `.factory/stories/STORY-INDEX.md`
- Dependency graph: `.factory/stories/dependency-graph.md`
- Sprint state: `.factory/stories/sprint-state.yaml`

### Per-story delivery flow

1. Create worktree: `git worktree add .worktrees/STORY-NNN -b feature/S-NNN develop`
2. test-writer: stubs + failing tests (Red Gate — tests must FAIL before implementer starts)
3. implementer: TDD (make tests pass, zero `.unwrap()`, clippy::pedantic clean)
4. adversary: 3 consecutive clean passes (BC-5.39.001 — CLEAN strict = zero findings any severity)
5. `git push origin feature/S-NNN` → PR targeting `develop` → CI (17 checks) → squash-merge → state update
6. Remove worktree: `git worktree remove .worktrees/STORY-NNN`

### After all Wave 3 stories merge: Wave 3 Gate

- Full `cargo test --workspace --no-fail-fast` on develop
- Adversarial wave-gate review (3-CLEAN required, findings reset per wave)
- Verify no regressions from Wave 1/2 tests
- vsdd-factory:wave-gate skill

### Sync develop before starting

```bash
git fetch origin develop && git pull origin develop
```

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight (2026-05-23) |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 decision docs + 14 research threads + 7/7 spikes resolved |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + architecture (14 ADRs, 15 VPs, 20 crates) + UX spec (10 screens, 5 flows) + L2 domain spec (12 files). 17 passes, 69 findings, 3/3 clean. |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 77 stories, 21 epics, 6 waves, 462 pts (updated 2026-05-31: STORY-077 added per architect directive). 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Wave 1: COMPLETE + GATE PASSED. Wave 2: COMPLETE + GATE PASSED. Wave 3: COMPLETE + GATE PASSED 2026-05-31 (22 stories; 2584 tests GREEN; adversary 3-CLEAN passes 6/7/8; fix-PR #38 @ 7d266ad7). **Wave 4 STARTED 2026-05-31 — 17 stories / 104 pts, Batch A in progress.** STORY-035: 743/743 GREEN, adversary Pass 2 in flight (streak 0/3). | Per-story delivery |
| Phase 4: Holdout Evaluation | NOT STARTED | Per-wave holdout gates |
| Phase 5: Adversarial Refinement | NOT STARTED | Post-implementation cascade |
| Phase 6: Formal Hardening | NOT STARTED | Kani + fuzz + mutants + semgrep |
| Phase 7: Convergence | NOT STARTED | 7-dimension convergence assessment |

## Wave 1 Story Status (ALL MERGED — Gate PASSED)

14 stories: STORY-001 through STORY-010 + STORY-051/052/053/054 (CI). All merged to develop. Wave 1 gate: 3 adversarial passes (7→4→0 findings), keyword + field name sync (PRs #12 #13), `#![forbid(unsafe_code)]` + `clippy::pedantic` on all 9 crates.

## Wave 2 Story Status (ALL MERGED — Gate PASSED)

| Story | Title | Crate | Tests | Adversary | PR | Commit |
|-------|-------|-------|-------|-----------|-----|--------|
| STORY-011 | Expression Evaluator Core | slideforge-eval | 75 | 5 passes (5→1→0→0→0), 3/3 | #14 | 8c0f4915 |
| STORY-012 | @for Evaluation + Scoping + Set-Rules + Variants | slideforge-eval | 124 | 6 passes (8→6→4→0→0→0), 3/3 | #16 | 5eaa81b2 |
| STORY-013 | @if/@elif/@else + @include Cycle Detection | slideforge-eval | 157 | 6 passes (10→5→3→0→0→0), 3/3 | #18 | dbcb694c |
| STORY-014 | No-Implicit-Coercion + ${{ seq }} Disambiguation | slideforge-eval | 155 | 7 passes (7→5→2→1→0→0→0), 3/3 | #19 | b17819aa |
| STORY-015 | Alt Text Enforcement | slideforge-validate | 38 | 6 passes (9→2→1→0→0→0), 3/3 | #15 | 5d043eff |
| STORY-016 | Canvas Overflow + Zero-Slide + Validation Mode | slideforge-validate | 84 | 5 passes (10→3→0→0→0), 3/3 | #17 | 6022075e |
| STORY-017 | Color-Coded Label + WCAG Contrast + Lang | slideforge-validate | 138 | 9 passes (5→3→2→1→0→1→0→0→0), 3/3 | #20 | e8e31bab |

Wave 2 gate: 11 passes, 19 findings fixed, 3/3 clean (passes 9-10-11). Gate fix commits: 8d9b8952, 9dbb9592, e9c33fdc, 93ff4e5d, e93432c3. Key fixes: E-PAR-004 collision resolved, cross-crate integration tests added, EMU overflow capped, DivisionByZero reverted to E-EVL-003 per BC-1.02.001, DSL version propagated from AST.

## Wave 3 Batch 1 Story Status (ALL MERGED — Batch 2 NEXT)

6 root stories (no intra-wave dependencies), 494 new tests across 6 new crates. All 3/3 clean adversary convergence.

| Story | Title | Crate | Tests | Adversary | PR | Commit |
|-------|-------|-------|-------|-----------|-----|--------|
| STORY-018 | DataSource: JSON/CSV/YAML/TOML File Loading | slideforge-data | 79 | 9 passes, 3/3 clean | #21 | 6d4d7c02 |
| STORY-022 | Brand Loading: .pptx/.docx Template Extraction | slideforge-brand | 71 | 9 passes, 3/3 clean | #22 | de04e29a |
| STORY-026 | Core Layout: Deck → LaidOutDeck, EMU System | slideforge-layout | 85 | 8 passes, 3/3 clean | #23 | 22d448c6 |
| STORY-029 | Math Parser: $...$ / $$...$$ + @{var} + OMML Output | slideforge-math | 77 | 16 passes, 3/3 clean | #24 | 98c696f8 |
| STORY-031 | Chart Renderer: bar/line/pie/scatter/area/histogram/stacked-bar | slideforge-charts | 108 | 7 passes, 3/3 clean | #25 | fece5bf2 |
| STORY-033 | Diagram Renderer: Mermaid → PPTX-Safe SVG | slideforge-diagrams | 74 | 7 passes, 3/3 clean | #26 | 218334f1 |

New crates added by Batch 1 (total workspace now 13 crates): slideforge-data, slideforge-brand, slideforge-layout, slideforge-math, slideforge-charts, slideforge-diagrams.

## Wave 3 Batch 2 Story Status (COMPLETE — All 13 stories merged, PRs #27–#34)

| Story | Title | Crate | Tests | Adversary | PR | Commit |
|-------|-------|-------|-------|-----------|-----|--------|
| STORY-019 | DataSource: HTTP/HTTPS + SSRF Allowlist | slideforge-data | 143 | 9 passes (20→8→5→6→3→6→0→0→0), 3/3 clean | #27 | 7bc71f9c |
| STORY-032 | Chart Empty Data Placeholder | slideforge-charts | 146 | 7 passes (passes 5,6,7 CLEAN), 3/3 | #28 | 5ad267be |
| STORY-027 | Layout: DOCX Section Generation | slideforge-layout | 48 | 8 passes, 3/3 clean | #29 | 7641d4ea |
| STORY-034 | SVG Normalization via usvg | slideforge-diagrams | +71 | 9 passes, 3/3 clean (pre-merge adversary); 4 CI iterations post-3/3: rebase + clippy + cold_budget budget + Linux font fallback | #30 | 0cb4b982 |
| STORY-030 | Math: MathML + PDF Paths | slideforge-math | +170 +2 traced_test (~172 net) | 9 iterations (Pass 9-17): font-engine refactor + Linux font_resolver insights from STORY-034 + 2 paper-fix corrections (TD-VSDD-059); 3/3 CLEAN Pass 15/16/17 | #31 | 19e79696 |
| STORY-023 | Brand Synthesis: brand.toml → 31 Layouts | slideforge-brand | +113 net | 20 passes, 3/3 CLEAN (P18-20); 5 fix bursts; 3 CRIT spec drifts (E-BRD-002/005/007); PR-level 2 cycles, cycle 2 CLEAN; F1 BrandPalette slot mismatch + F2 debug_assert fixed in 4f78aa1c | #32 | dd6054c1 |
| STORY-020 | DataSource: Excel + SQLite | slideforge-data | 251 + 2 perf_smoke | 29 passes, 3/3 CLEAN (P27-28-29); AKM compounding-novelty; implementer-overclaim correction (P27); sibling-sweep recurrence (BC version bumps); 23 ACs demo'd. PR-level 1 cycle CLEAN; security clean. | #33 | 143f1b78 |
| STORY-028 | Layout: shape: Block + Rich Inline | slideforge-layout | 309 layout + 1764 workspace | 32 passes, 3/3 CLEAN (P30-31-32); 18 ACs demo'd; CI fix burst 1 cycle (fmt + clippy); follow-ups: STORY-072/073/074; PR-level 1 cycle CLEAN; 0 CRIT/HIGH/MED security | #34 | 066d625f |

## Decisions Log (milestones)

- 2026-05-23 — Workspace resolved, mode: greenfield
- 2026-05-23 — Market intelligence: GO
- 2026-05-24 — ALL 25 DSL DESIGN QUESTIONS COMPLETE
- 2026-05-24 — ALL 7/7 SPIKES RESOLVED
- 2026-05-24 — PRD COMPLETE (109 BCs)
- 2026-05-24 — ARCHITECTURE COMPLETE (14 ADRs, 15 VPs, 20 crates)
- 2026-05-24 — UX SPEC COMPLETE (10 screens, 5 flows)
- 2026-05-25 — PHASE 1 CONVERGED (17 passes, 69 findings, 3/3 clean) + APPROVED — Phase 2 authorized
- 2026-05-25 — PHASE 2 CONVERGED (22 passes, 96+ findings, 3/3 clean) + APPROVED — Phase 3 authorized
- 2026-05-25 — PHASE 3 STARTED — Wave 1, STORY-001 in progress
- 2026-05-26 — STORIES 001–010 + 051/052/053/054 MERGED (Wave 1, 14 stories)
- 2026-05-27 — WAVE 1 GATE PASSED — 3 gate passes (7→4→0), 11 findings fixed
- 2026-05-27 — STORY-011 MERGED (PR #14, 8c0f4915) — Expression Evaluator Core
- 2026-05-27 — STORY-015 MERGED (PR #15, 5d043eff) — Alt Text Enforcement
- 2026-05-27 — STORY-012 MERGED (PR #16, 5eaa81b2) — @for Evaluation + Scoping
- 2026-05-27 — STORY-016 MERGED (PR #17, 6022075e) — Canvas Overflow + Zero-Slide
- 2026-05-27 — STORY-013 MERGED (PR #18, dbcb694c) — @if/@elif/@else + @include Cycle Detection
- 2026-05-27 — STORY-014 MERGED (PR #19, b17819aa) — No-Implicit-Coercion + ${{ seq }}
- 2026-05-27 — STORY-017 MERGED (PR #20, e8e31bab) — Color-Coded Label + WCAG Contrast + Lang
- 2026-05-27 — WAVE 2 GATE PASSED — 11 gate passes, 19 findings fixed, 3/3 clean (passes 9-10-11)
- 2026-05-27/28 — WAVE 3 BATCH 1 COMPLETE — 6 new crates (data, brand, layout, math, charts, diagrams), 494 new tests, 6 PRs merged (#21-#26, commits 6d4d7c02 → 218334f1)
- 2026-05-28 — STORY-019 MERGED (PR #27, 7bc71f9c) — HTTP/HTTPS DataSource + SSRF allowlist (9-pass adversary, 49 findings fixed, defense-in-depth: allowlist before DNS + redirects(0) + body cap + scheme normalization)
- 2026-05-28 — STORY-032 MERGED (PR #28, 5ad267be) — Chart empty-data placeholder, 7-pass adversary convergence
- 2026-05-28 — STORY-027 MERGED (PR #29, 7641d4ea) — DOCX section generation (executive_summary + risk_register), 8-pass adversary convergence
- 2026-05-28 — STORY-030 font engine refactor: ab_glyph 0.2.31 + embedded Latin Modern Math 733KB OTF (GFL/LPPL) replacing synthetic glyph match — authorized by user to fix Pass 9 HIGH findings; now at pass 10 threshold
- 2026-05-28 — STORY-034 adversary 3/3 CONVERGED (9 passes); branch rebased onto develop (picks up STORY-019/027/032); PR #30 OPEN (CONFLICTING, needs rebase)
- 2026-05-28 — STORY-034 PR #30 force-push (local rebase to 92221f3d had never reached remote) + 4 CI iterations:
  - iter-1: implementer fontdb-empty hypothesis (apt-get fonts-dejavu-core + fonts-noto-core + fc-cache + Linux fontdb fallback dir scan) — INSUFFICIENT
  - iter-2: clippy len_zero fix + cold_budget budget 200→300ms + diagnostic eprintlns — DIAGNOSTIC REVEALED fontdb=324 but SVG <text>=false
  - iter-3: fonts-liberation added to apt-get + diagnostic removed + cargo fmt — INSUFFICIENT (Liberation Sans Arial-compatible but not Trebuchet-compatible)
  - iter-4: ROOT CAUSE — Mermaid emits font-family="'trebuchet ms', verdana, arial, sans-serif"; usvg 0.47 lookup is case-sensitive byte-compare; doesn't implement CSS fallback chain. Fix: custom usvg::Options::font_resolver with fallback chain (Liberation Sans → DejaVu Sans → Noto Sans → FreeSans). + fonts-freefont-ttf for defense-in-depth. All 17 CI checks pass on c85e4a32.
- 2026-05-28 — STORY-034 MERGED (PR #30 → squash → 0cb4b982). slideforge-diagrams now has post-rendering usvg normalization with PPTX-safe text preservation across all 4 platforms (macOS arm64, Linux x86_64, Linux arm64, Windows x86_64).
- 2026-05-28 — STORY-030 (Math: MathML + PDF Paths) MERGED (PR #31 → squash → 19e79696). slideforge-math crate ships with: LaTeX → MathML for HTML output target; LaTeX → SvgPaths newtype (PDF/SVG vector glyphs via ab_glyph 0.2.31 + embedded Latin Modern Math 733KB OTF); GUST Font License v1.0 verbatim + MANIFEST.toml SHA-256 audit + LICENSE-LatinModernMath.txt; @{var} math-mode interpolation; tracing #[instrument] on MathRendererImpl::render, render_mathml, render_pdf_paths; 170 tests in slideforge-math + 2 traced_test load-bearing assertions. Convergence: 9 adversary iterations, 2 paper-fix corrections (TD-VSDD-059): Pass 10 SvgPaths newtype + ab_glyph integration; Pass 11 OnceLock cache was dead code (only used for units_per_em — font field still reparsed; corrected to true &'static GlyphEngine); 3/3 CLEAN Pass 15/16/17. PR #31 required 1 CI iteration (clippy missing_docs on DISPLAY_SCALE_DEN + broken intra-doc link slideforge_plugin_api → slideforge_math::MathAst, both 1-line fixes). Per-AC demo evidence in docs/demo-evidence/STORY-030/ (AC-001/002/003/004 .tape + .gif + .webm + evidence-report.md; AC-005 deferred to STORY-047).
- 2026-05-28 — STORY-023 CONVERGED 3/3 CLEAN (Pass 18, 19, 20) + demos done (15 ACs at d771099e). 20-pass adversary trail, 5 fix bursts, 1 scope-discipline correction (non_exhaustive over-application caught Pass 14), 3 spec-code drift corrections (E-BRD-002 PPTX/TOML→PPTX/DOCX, E-BRD-005 un-retired with new semantic, E-BRD-007 documented in error-taxonomy.md).
- 2026-05-28 — Session handoff: STATE.md is the resume document. Top 3 next actions captured in Session Resume Brief. STORY-023 demo evidence pushed at d771099e (15 ACs covered in docs/demo-evidence/STORY-023/).
- 2026-05-29 — STORY-023 MERGED (PR #32, dd6054c1) — Brand Synthesis: brand.toml → 31 Layouts. Convergence: 20 LOCAL adversary passes, 3/3 CLEAN (P18-20). PR-level review: 2 cycles, cycle 1 (2 findings: F1 HIGH BrandPalette slot mismatch, F2 SUGGEST debug_assert gap), cycle 2 CLEAN (PR-merge). F1/F2 fixed in 4f78aa1c: shared color_by_name slot-name mapping + load-bearing regression test (test_f1_regression_brand_palette_primary_maps_to_dk2_not_dk1) + debug_assert in inference.rs. Wave 3 Batch 2 now 6/8 merged; STORY-020 + STORY-028 remaining.
- 2026-05-30 — STORY-020 MERGED (PR #33, 143f1b78) — DataSource: Excel + SQLite. Convergence: 29 LOCAL adversary passes, 3/3 CLEAN (P27-28-29). AKM compounding-novelty: paper-fixes, sibling-sweep gaps, semantic anchoring drift, BC version propagation, doc-vs-code precision all found across the run. Implementer-overclaim correction at P27 (claimed 33 renames in 4 files; adversary verified 8 in 1 file). 23 ACs demo'd, 251 unit tests + 2 perf_smoke. PR-level review: 1 cycle CLEAN; security: 0 findings. Wave 3 Batch 2 now 7/8 merged; STORY-028 remaining.
- 2026-05-30 — STORY-028 MERGED (PR #34, 066d625f) — Layout: shape: Block + Rich Inline. Convergence: 32 LOCAL adversary passes, 3/3 CLEAN (P30-31-32) per BC-5.39.001. 18 ACs demo'd, 309 slideforge-layout tests + 1764 workspace tests. PR-level review: 1 cycle CLEAN (zero CRIT/HIGH/MED); security: 0 CRIT/HIGH/MED + 3 LOW informational. CI fix burst: 1 cycle (fmt + clippy). Follow-up stories created during cascade: STORY-072 (gradient fills), STORY-073 (bullets layout), STORY-074 (brand-em-sizing). Wave 3 Batch 2 COMPLETE — all 13 stories merged (PRs #27–#34). Ready for Batch 3 (STORY-021, 024, 025).
- 2026-05-30 — STORY-021 MERGED (PR #35, 362c4a1f) — DataSource HTTP cache + E-DAT policy errors. Convergence: 23 LOCAL adversary passes, 3/3 CLEAN (P21-22-23) per BC-5.39.001. ~80 findings closed across 20 fix-burst commits. 10 ACs demo'd (30 evidence files at docs/demo-evidence/STORY-021/). Key structural changes: PolicyRejected variant for E-DAT-006 body-cap; label-driven IoError vs FileNotFound routing (P4); #[non_exhaustive] on DataError + DataSourceError (P5); UTF-8 panic fix in strip_bracket_prefix (P9); parse_e_dat_code for E-DAT-007..014 granularity (P10); E-DAT-014 retired at dispatcher boundary (P13); --offline hint added per spec (P14); STRUCTURAL FIX — http.rs Display decoupled from inter-layer protocol (P15). Workspace tests: 2282 (up from 1764). Wave 3 Batch 3: 1/3 merged.
- 2026-05-30 — STORY-024 RED GATE COMPLETE (feature/S-024 @ b61b0d04) — Brand Extraction CLI (BC-2.01.003). 15 failing tests covering 10 ACs using test_bc_2_01_003_* prefix. BrandExtractor stub + E_BRD_006 variant created. LIBRARY-ONLY: CLI wiring deferred to STORY-057. Implementer notes captured in Session Resume Brief (BrandConfig field order, [ColorSlot; 12] fixed array, LogoAsset ZIP-internal path extension, EC-003 coverage gap).
- 2026-05-30 — STORY-025 RED GATE COMPLETE (feature/S-025 @ 27d67613) — Per-Slide brand_overlay invariant (BC-2.02.001+002). 9 should_panic + 12 structural failing tests. SlideOverlay fully implemented; resolve_overlay + infer_media_type stubbed. TD-VSDD-060 sibling sweep: 42 Slide constructions updated with overlay: None across 11 files. Implementer must convert should_panic to direct assert! on green phase. AC-007/009/012 (parser rejection) deferred to STORY-008/009 per Architecture Compliance Rule 4.
- 2026-05-30 — STORY-024 LOCAL ADVERSARY 3/3 STRICT-CLEAN CONVERGED (11 passes). Key hardening during cascade: scheme-ref hex resolution (schemeClr must resolve against theme table, not passthrough symbolic name), TOML string escaping for special chars in brand names/fonts, EC-005 logo format detection emits warn not error (E-BRD-006), AC-007 color slot values quoted as strings in TOML, end-to-end round-trip test added. Spec parity achieved: E-BRD-006 row added to error-taxonomy.md, CAP-018 registered, VP-051 (brand round-trip extraction integration test) + VP-052 (source file read-only invariant) added to VP-INDEX (50→52), verification-architecture.md + verification-coverage-matrix.md updated. BC-2.01.003 v1.1→v1.8 (8 incremental versions during cascade). Follow-up stories created: STORY-075 (footer detection — loader.rs:214 hardcodes footer_text: None, dormant [footer] writer in extractor) + STORY-076 (srgbClr lumMod/tint/shade transform-aware extraction — silent drop in color.rs). STORY-INDEX 74→76. 211 slideforge-brand tests passing.
- 2026-05-30 — STORY-024 MERGED (PR #36, 924cdc04) — Brand Extraction library (BC-2.01.003). 11-pass adversary 3/3 strict-CLEAN (P9-10-11); library-only (CLI wiring deferred to STORY-057); 11 ACs demo'd. AI PR-diff review PR-merge-CLEAN (2 LOW/OBS non-blocking: BrandError::ParseError reuse for I/O — naming nit; .unwrap_or("bin") logo-ext fallback — documented). Security review CLEAN. All 17 CI checks passed. Wave 3 Batch 3: 2/3 merged.
- 2026-05-30 — STORY-025 LOCAL ADVERSARY 3/3 STRICT-CLEAN CONVERGED (11 passes). Key hardening during cascade: HIGH path-traversal containment guard in resolve_overlay (E-BRD-007, reuses strip_unc_prefix); media_type unified to shared logo::media_type_from_extension (eliminates divergence risk); EC-007 unknown-ext emits warn not error; EC-008 empty-path returns LogoRequired; LogoRequired message context-neutral (cross-context reuse). Spec parity: E-BRD-007 widened to overlay coverage (error-taxonomy v2.2); shared-variant rows for E-BRD-001/002; TomlReadError/TomlParseError variant-name corrections; BC-2.02.001 v1.1→v1.4 (EC-006 path-containment, EC-007, EC-008, "brand root directory" wording, subsystem SS-04); BC-2.02.002 v1.1→v1.2 (subsystem SS-06). 455 slideforge-brand+types tests passing. 10 ACs demo'd. Two [process-gap] lessons captured in .factory/cycles/STORY-025/lessons.md: (a) taxonomy/BC widen requires same-burst sweep of #[error] string + variant docs + field docs + sibling doc-tables (recurred 3x); (b) new error return arm requires `# Errors` rustdoc completeness check against all reachable arms (recurred 2x).
- 2026-05-30 — STORY-025 MERGED (PR #37, 584cbc6f) — Per-Slide brand_overlay invariant (BC-2.02.001+002). 11-pass adversary 3/3 strict-CLEAN (P9-10-11); 10 ACs demo'd; HIGH path-traversal guard confirmed. AI PR-diff review PR-merge-CLEAN (1 LOW + 4 OBS non-blocking). Security CLEAN. All 17 CI checks passed. Wave 3 Batch 3 COMPLETE — all 3 stories merged (STORY-021 PR #35, STORY-024 PR #36, STORY-025 PR #37). NEXT: Wave 3 Gate.
- 2026-05-31 — WAVE 3 GATE: Full test suite GREEN (2584 tests, 0 failures) on develop @ 584cbc6f. Holdout evaluation: must-pass PASS (5/5 must-pass ≥0.60; SSRF/HS-013 0.90; mean 0.767 over 6 evaluable scenarios — 9 blocked scenarios are structural mid-build artifact, not a quality defect; full mean-satisfaction holdout deferred to post-exporter waves). Adversarial wave-diff review CONVERGED: 8 passes, strict-CLEAN at passes 6/7/8. Integration findings fixed on branch fix/wave3-gate: slideforge-brand → [workspace.dependencies]; #[non_exhaustive] hardening across all public error enums + PATH-A plugin-api data enums with OBS-1 wildcard-error arms; conventions.md v1.3 policy codified. Fix-PR in flight.
- 2026-05-31 — CONVENTIONS.md v1.3: #[non_exhaustive] Policy codified (PATH A variant-growth / PATH B closed-domain framework; exhaustive plugin-api classification table; scope statement; wildcard-arm rule; checklist). Policy artifact is POLICY-only — conformance verified by adversary + clippy, not recorded in spec. Stale-checkout code-state column removed (OBS-2 correction; was written from develop checkout lacking fix commits).
- 2026-05-31 — WAVE 3 GATE FIX-PR #38 MERGED (squash commit 7d266ad7). Integration findings fixed: slideforge-brand → [workspace.dependencies]; #[non_exhaustive] hardening across all public error enums + PATH-A plugin-api data enums with OBS-1 wildcard-error arms; conventions.md v1.3. AI review strict-CLEAN; security CLEAN; 17/17 CI checks pass. Adversary pass 8 strict-CLEAN.
- 2026-05-31 — WAVE 3 COMPLETE. All 22 stories merged (Batch 1: 6, Batch 2: 13, Batch 3: 3) + gate PASSED. develop @ 7d266ad7 (38 PRs, ~2584 tests, 0 failures). 0 active worktrees. NEXT: Wave 4.
- 2026-05-31 — WAVE 4 STARTED. Human-approved: 16 stories / 96 pts (expanded from 13/85). STORY-073/075/076 pulled in as P1 follow-ups; STORY-072/074 deferred to Wave 5 as P2. BC-2.01.001 v1.2, BC-2.01.003 v1.9, error-taxonomy v2.3 landed. Batch A dispatch next.
- 2026-05-31 — STORY-035 IN PROGRESS (feature/S-035 @ 1b4f654e): Red Gate + Green + adversary LOCAL Pass 1 (2 HIGH/2 MED/1 LOW) + fix-burst. 743/743 GREEN, clippy/fmt clean. Adversary Pass 2 in flight (streak 0/3). Architect directive issued: F-001 Option D (eval populates Slide.register_content; layout clones; delete slideforge-types duplicate); F-002 AC-005/EC-003 descoped to STORY-077. BC-1.14.004 added to STORY-035 spec.
- 2026-05-31 — STORY-077 CREATED: SectionBlock IR Extension (architect-directed spin-out from STORY-035 F-002 descope). EPIC-18, P0, 8pts, Wave 4 Batch A. Anchors BC-3.02.002 + BC-1.14.003. Blocks STORY-041 + STORY-042. Wave 4 now 17 stories / 104 pts; project total 77 stories / 462 pts.

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-05-31 |
| **Position** | Phase 3, Wave 4 STARTED. Wave 3 Gate PASSED (PR #38, 7d266ad7). STORY-035 Batch A in progress — adversary Pass 2 running (streak 0/3). |
| **develop SHA** | 7d266ad7 (38 merged PRs) |
| **Workspace tests** | ~2584 (full-suite run 2026-05-31 @ 584cbc6f; fix-PR #38 is additive config+attrs only) |
| **Workspace crates** | 13 (7 Wave 1 + 6 Batch 1: data, brand, layout, math, charts, diagrams) |
| **Active worktrees** | 1: .worktrees/STORY-035 (feature/S-035, 743/743 GREEN) |
| **Open PRs** | 0 |
| **Wave 4 batch plan** | Batch A: STORY-035→036, STORY-043→044→045 (slideforge-pdf NEW), STORY-073, STORY-075, STORY-076, STORY-077. Batch B: STORY-037→040 (PPTX), STORY-041→042 (DOCX; blocked on STORY-077). Batch C: STORY-049→050. |
| **STORY-035 state** | Red Gate 3392b598 + Green 4f8a9487 + Pass 1 fix-burst 1b4f654e. Adversary Pass 2 in flight. Architect directive: F-001 Option D + F-002 descope to STORY-077. BC-1.14.004 added. |
| **STORY-077** | Created 2026-05-31. SectionBlock IR Extension, EPIC-18, P0, 8pts, Wave 4 Batch A. Blocks STORY-041/042. |
| **factory-artifacts** | Local only (not pushed to remote). Push requires explicit human authorization per CLAUDE.md. |
| **Highest priority next** | 1. Complete STORY-035 adversary cascade (need 3-CLEAN; Pass 2 in flight). 2. Continue remaining Batch A stories in parallel (STORY-036, STORY-043→044→045, STORY-073, STORY-075, STORY-076, STORY-077). 3. slideforge-pdf workspace scaffolding (STORY-043 first task). |
| **Holdout caveat** | Full mean-satisfaction holdout (≥0.85) deferred to post-exporter waves; 9/15 scenarios blocked by missing CLI + exporters. Re-run after PPTX/HTML exporters ship. |
| **Process improvements to apply** | Before accepting any "clippy clean": run `rustup update stable && cargo clippy --workspace --all-targets --all-features -- -D clippy::pedantic -D clippy::unwrap_used`. Add cross-crate compile check for stories touching slideforge-brand or slideforge-types. |

## Quality Bar (Non-Negotiable)

Production-grade from day 1. Key enforced gates:
- `#![forbid(unsafe_code)]` on all crates; zero `.unwrap()` outside tests; `clippy::pedantic` clean; `#![warn(missing_docs)]` on public APIs
- Kani proofs for pure-core functions (Phase 6); `cargo-fuzz` harness; `cargo-mutants` with documented kill-rate budget
- WCAG AA: web preview + HTML; PDF/UA-1 on PDF; OOXML accessibility linter on PPTX
- Multi-renderer visual parity: PowerPoint, Keynote, Google Slides, LibreOffice
- < 500ms cold build, < 50ms incremental (25-slide deck) — CI benchmark gate
- Signed releases + SBOM; `cargo audit` + `cargo deny` in CI; semgrep/CodeQL per PR
- Cross-platform: macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64
- Full quality bar table: CLAUDE.md

## Key Spec References

| Document | Scope |
|----------|-------|
| .factory/specs/behavioral-contracts/ | 109 BCs organized by section |
| .factory/specs/architecture/ARCH-INDEX.md | 12 architecture sections, 14 ADRs |
| .factory/specs/prd.md | PRD + 109 BCs + 15 holdout scenarios |
| .factory/stories/wave-schedule.md | 6 waves, batching, dependency order |
| .factory/stories/dependency-graph.md | Full story dependency graph |
| .factory/stories/STORY-INDEX.md | 77 stories with status |
| .factory/stories/sprint-state.yaml | Current sprint/wave state |
| .factory/planning/q1-decision-final.md | Computation, formats, registers, charts, math, brand, roadmap |
| .factory/planning/q2-decision-final.md | 31 types, aliases, components, DSL syntax per type |
| .factory/planning/q3-decision-final.md | Plugin-first architecture, 10 surfaces, trait signatures |
| .factory/planning/q4-q15-decisions.md | Template binding, output, a11y, shape DSL, inline, variants |
| .factory/planning/q16-q25-decisions.md | Versioning, strict mode, i18n, packages, workspace, errors, merge |

## Drift Items

| Date | Item | Severity | Notes |
|------|------|----------|-------|
| 2026-05-28 | LOCAL adversary 3-CLEAN convergence on STORY-034 ran exclusively on macOS, missed Linux-only Trebuchet MS substitution failure in usvg normalization (BC-1.12.001 violation) | LOW | Required 4 CI iterations to remediate. Process-gap candidate: should LOCAL adversary spawn a Linux-container-based test pass for any story touching font/text/SVG/rendering code paths? Surface to user for codification decision. |
