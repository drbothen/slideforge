---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-05-28
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
total_stories: 71
total_points: 437
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
develop_sha: "19e79696"
develop_pr_count: 31
workspace_tests: 1672
workspace_test_failures: 0
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. It generates branded .pptx, .docx, .pdf, .html, and a live web preview from a single indentation-significant DSL (.sf files) with data binding, iteration, conditionals, and a plugin-first architecture.

**Tagline:** "Branded documents from structured data — one source, every format."

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

## Current Status

Phase 3 IN PROGRESS. Wave 1 COMPLETE (14/14 stories, gate PASSED). Wave 2 COMPLETE (7/7 stories, gate PASSED). Wave 3 Batch 1 COMPLETE (6 stories merged, PRs #21-#26). **Wave 3 Batch 2 IN PROGRESS (7/8 merged: STORY-019, 027, 030, 032, 034 + 2 earlier). 1 in-flight (STORY-023, streak 2/3), 2 not started (STORY-020, STORY-028).**

develop branch: `19e79696` (31 merged PRs, 1672 tests, 0 failures). 1 active worktree (STORY-023). 0 open PRs.

## Wave 3 Batch 2 Story Status

| Story | Title | Status | PR | Commit | Notes |
|-------|-------|--------|----|--------|-------|
| STORY-019 | HTTP/HTTPS DataSource + SSRF | MERGED | #27 | 7bc71f9c | 9-pass adversary, 3/3 clean |
| STORY-032 | Chart Empty Data Placeholder | MERGED | #28 | 5ad267be | 7-pass adversary, 3/3 clean |
| STORY-027 | Layout: DOCX Section Generation | MERGED | #29 | 7641d4ea | 8-pass adversary, 3/3 clean |
| STORY-023 | Brand Synthesis: brand.toml → 31 Layouts | IN PROGRESS | — | 800b11d8 | Streak 2/3 CLEAN (passes 9,10) |
| STORY-030 | Math MathML + PDF Paths | MERGED | #31 | 19e79696 | 9 adversary iterations (Pass 9-17) incl. 2 paper-fix corrections (TD-VSDD-059): font-engine refactor (ab_glyph 0.2.31 + embedded LM Math 733KB OTF); paper-fix detection Pass 11 cache (OnceLock dead code); 3/3 CLEAN Pass 15/16/17 |
| STORY-034 | SVG Normalization via usvg | MERGED | #30 | 0cb4b982 | 4 CI iterations (force-push rebase + usvg font_resolver fix for Linux Trebuchet MS substitution) |
| STORY-020 | DataSource: Excel + SQLite | NOT STARTED | — | — | Depends on STORY-019 (merged) |
| STORY-028 | Layout: shape: Block + Rich Inline | NOT STARTED | — | — | Depends on STORY-027 (merged) |

## Resume Instructions for Each In-Flight Story

### STORY-023 — Brand Synthesis (slideforge-brand, 13pts XL)
- Worktree: `.worktrees/STORY-023/`  Branch: `feature/S-023`  SHA: `800b11d8`
- Convergence: 10 passes, streak **2/3 CLEAN** (passes 9, 10); need 1 more clean strict pass
- All known findings closed; tests pass 160/160 in slideforge-brand
- Key implementation: 31 OOXML layouts, 12 color slots with deterministic inference, brand.toml synthesis, ECMA-376-compliant XML serialization, dark layout defRPr defaults, hex validation, logo path resolution
- **Next: Run Pass 11 adversary review. If CLEAN strict → push + demos + PR. If findings → fix burst → Pass 12.**

## What to Do After In-Flight Stories Complete

**After STORY-023 merge:**
- Start STORY-020 (DataSource: Excel + SQLite, depends on STORY-019 — merged)
- Start STORY-028 (Layout: shape: Block + Rich Inline, depends on STORY-027 — merged)
- After Batch 2 fully merged: Batch 3 (STORY-021, 024, 025)

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
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 71 stories, 21 epics, 6 waves, 437 pts. 22 passes, 96+ findings, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Wave 1: COMPLETE + GATE PASSED. Wave 2: COMPLETE + GATE PASSED. Wave 3: Batch 1 COMPLETE (6/17 stories, PRs #21-#26, 494 new tests). Batch 2: 7/8 merged: STORY-019 (#27), STORY-027 (#29), STORY-030 (#31), STORY-032 (#28), STORY-034 (#30) + 2 Batch 1. 1 in-flight (STORY-023 streak 2/3). 2 not started (STORY-020, STORY-028). | Per-story delivery |
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

## Wave 3 Batch 2 Story Status (IN PROGRESS — STORY-019 + STORY-032 + STORY-027 + STORY-034 MERGED; 2 in-flight, 2 not started)

| Story | Title | Crate | Tests | Adversary | PR | Commit |
|-------|-------|-------|-------|-----------|-----|--------|
| STORY-019 | DataSource: HTTP/HTTPS + SSRF Allowlist | slideforge-data | 143 | 9 passes (20→8→5→6→3→6→0→0→0), 3/3 clean | #27 | 7bc71f9c |
| STORY-032 | Chart Empty Data Placeholder | slideforge-charts | 146 | 7 passes (passes 5,6,7 CLEAN), 3/3 | #28 | 5ad267be |
| STORY-027 | Layout: DOCX Section Generation | slideforge-layout | 48 | 8 passes, 3/3 clean | #29 | 7641d4ea |
| STORY-034 | SVG Normalization via usvg | slideforge-diagrams | +71 | 9 passes, 3/3 clean (pre-merge adversary); 4 CI iterations post-3/3: rebase + clippy + cold_budget budget + Linux font fallback | #30 | 0cb4b982 |
| STORY-030 | Math: MathML + PDF Paths | slideforge-math | +170 +2 traced_test (~172 net) | 9 iterations (Pass 9-17): font-engine refactor + Linux font_resolver insights from STORY-034 + 2 paper-fix corrections (TD-VSDD-059); 3/3 CLEAN Pass 15/16/17 | #31 | 19e79696 |

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

## Session Resume Checkpoint

| Field | Value |
|-------|-------|
| **Date** | 2026-05-28 |
| **Position** | Phase 3, Wave 3 — Batch 2 IN PROGRESS (7/8 merged: STORY-019, 027, 030, 032, 034 + 2 from Batch 1). 1 in-flight, 2 not-started. |
| **develop SHA** | 19e79696 |
| **Workspace tests** | 1672 passing, 0 failures |
| **Workspace crates** | 13 (7 from Wave 1 + 6 new from Batch 1: data, brand, layout, math, charts, diagrams) |
| **Active worktrees** | `.worktrees/STORY-023` (feature/S-023, 800b11d8) |
| **Open PRs** | 0 |
| **In-flight stories** | STORY-023: streak 2/3 CLEAN, need 1 more pass. |
| **Not-started stories** | STORY-020 (Excel+SQLite, dep STORY-019 merged), STORY-028 (shape:, dep STORY-027 merged) |
| **Highest priority next actions** | 1. STORY-023: adversary Pass 20 (streak 2/3 CLEAN; one more clean strict pass for 3/3 convergence) → push + per-AC demos + rebase + PR + merge. 2. After STORY-023 merge: start STORY-020 (Excel + SQLite DataSource, dep STORY-019 merged) + STORY-028 (shape: Block + Rich Inline, dep STORY-027 merged). 3. After Batch 2 fully merged: Batch 3 (STORY-021, 024, 025). |

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
| .factory/stories/STORY-INDEX.md | 71 stories with status |
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
