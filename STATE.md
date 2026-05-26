---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-3-tdd-implementation
status: IN_PROGRESS
last_updated: 2026-05-26
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
---

# Slideforge — Factory State

## What Is This Project?

slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. It generates branded .pptx, .docx, .pdf, .html, and a live web preview from a single indentation-significant DSL (.sf files) with data binding, iteration, conditionals, and a plugin-first architecture.

**Tagline:** "Branded documents from structured data — one source, every format."

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

## Current Status: Phase 3 IN PROGRESS — Wave 1: 7/14 merged (001/002/004/005/006/007), STORY-003/008/009/010 next

Phase 2 Story Decomposition: COMPLETE and APPROVED (2026-05-25).
- 71 stories decomposed from 112 BCs across 21 epics, 6 waves, 437 total points
- remove-uncertainty skill applied to all 71 stories (library versions validated via Perplexity/Context7; 50+ version fixes)
- Consistency validation: 22 findings found and fixed
- 22 adversarial passes, 96+ findings fixed, 3 consecutive clean passes (BC-5.39.001 satisfied)

Phase 1 Spec Crystallization: COMPLETE and APPROVED (2026-05-25).
- 17 adversarial passes, 69 findings fixed, 3 consecutive clean passes
- Convergence trajectory archived: .factory/cycles/v0.1.0-phase-1-spec/convergence-trajectory.md

## What to Do Next (Phase 3 TDD Implementation)

### Immediate first action

Start Wave 1 delivery. The per-story delivery flow is:
1. test-writer: stubs → 2. test-writer: failing tests → 3. implementer: TDD →
4. adversary: 3-CLEAN per story → 5. demo-recorder: per-AC demos → 6. push →
7. pr-manager: full 9-step PR process → 8. worktree cleanup

### Wave 1 Stories (14 stories, 85 points)

Internal sequencing:
- STORY-001 first (IR core types)
- STORY-002 after 001 (plugin trait API)
- STORY-003 + STORY-004 + STORY-005 after 001+002
- STORY-006 after 005
- STORY-007-010 after 006 (parser chain)
- STORY-051-054 independent CI stories (parallel with all)

### Key inputs for Phase 3

| Input | Location |
|-------|----------|
| Story files | .factory/stories/stories/STORY-NNN-*.md |
| Story index | .factory/stories/STORY-INDEX.md |
| Wave schedule | .factory/stories/wave-schedule.md |
| Dependency graph | .factory/stories/dependency-graph.md |
| BC specs | .factory/specs/behavioral-contracts/ |
| Architecture | .factory/specs/architecture/ |
| Sprint state | .factory/stories/sprint-state.yaml (create on first story start) |

### Phase 3 full sequence

| Step | Agent | Output | Depends on |
|------|-------|--------|-----------|
| Per-story | test-writer → implementer → adversary → demo-recorder → pr-manager | Merged story branches | Story file + deps complete |
| Wave gate | wave-gate skill | Integration validation | All stories in wave merged |
| Repeat | Next wave | Until Wave 5 (Wave 6 is Phase 6) | Prior wave gate PASS |

## Phase Progress

| Phase | Status | Key Output |
|-------|--------|-----------|
| Pre-pipeline | DONE | Toolchain + LLM + MCP preflight |
| Market intelligence | DONE 2026-05-23 | GO with medium confidence |
| Planning (25 DSL decisions) | DONE 2026-05-24 | q1–q25 decision docs + 14 research threads |
| Phase 1: Spec Crystallization | DONE — APPROVED 2026-05-25 | PRD (109 BCs, 15 HS, 4 supplements) + architecture (14 ADRs, 15 VPs, 20 crates) + UX spec (10 screens, 5 flows) + L2 domain spec (12 files) |
| Phase 2: Story Decomposition | DONE — APPROVED 2026-05-25 | 71 stories, 21 epics, 6 waves, 437 pts. 22 adversarial passes, 3/3 clean. |
| Phase 3: TDD Implementation | IN PROGRESS — Wave 1: STORY-001/002/004/005 merged (4/14), STORY-003/006 next | Per-story delivery |
| Phase 4: Holdout Evaluation | NOT STARTED | Per-wave holdout gates |
| Phase 5: Adversarial Refinement | NOT STARTED | Post-implementation cascade |
| Phase 6: Formal Hardening | NOT STARTED | Kani + fuzz + mutants + semgrep |
| Phase 7: Convergence | NOT STARTED | 7-dimension convergence assessment |

## Phase 1 Step Status (DONE)

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| P1-00 | devops-engineer | DONE | MSRV 1.85→1.88; rust-toolchain.toml + Cargo.toml + ci.yml |
| P1-01 | business-analyst | DONE | .factory/specs/domain-spec/ (12 files, L2-INDEX.md) |
| P1-02 | product-owner | DONE | prd.md + 4 supplements + 109 BCs + 15 HS |
| P1-03a | architect | DONE | 7/7 spikes resolved (S1 S2 S3 S4 S5 S6 S14) |
| P1-03b | architect | DONE | ARCH-INDEX + 12 sections + 14 ADRs + module-criticality |
| P1-04 | product-owner | DONE | PRD complete — 109 BCs, 15 HS, 4 supplements |
| P1-05 | architect | DONE | Feasibility: PASS-WITH-NOTES (5 notes addressed, BC-1.03.005 added) |
| P1-06 | ux-designer | DONE | UX-INDEX + 10 screens + 5 flows; VP-INDEX + 15 VPs |
| P1-07 | devops-engineer | DONE | DTU: not required; gene-transfusion: behavioral-only (75 behaviors); CI/CD: 3 workflows, 5-platform matrix |
| P1-09 | adversary | DONE — CONVERGED | 17 passes, 69 findings fixed. Trajectory: 17→12→10→6→2→2→0→2→1→3→3→1→3→1→0→0→0. Streak 3/3. |
| P1-10 | consistency-validator | DONE | 6 findings (2H, 2M, 2L) — all fixed |
| P1-11 | HUMAN | APPROVED 2026-05-25 | Phase 2 authorized |

## Phase 2 Step Status (DONE — AWAITING APPROVAL)

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| P2-01 | story-writer | DONE | epics.md, STORY-INDEX.md, dependency-graph.md, wave-schedule.md + 71 story files |
| P2-01b | story-writer | DONE | All 71 individual story specs written (11 bursts) |
| P2-01c | research-agent | DONE | remove-uncertainty on all 71 stories (6 wave batches, 50+ version fixes) |
| P2-02 | consistency-validator | DONE | 22 findings found and fixed |
| P2-03 | adversary | DONE — CONVERGED | 22 passes, 96+ findings fixed, 3/3 clean (passes 20-21-22) |
| P2-04 | state-manager | DONE | This commit |
| P2-05 | HUMAN | APPROVED 2026-05-25 | Phase 3 authorized |

## Decisions Log (milestones only — full log in cycles/v0.1.0-phase-1-spec/decisions-log.md)

- 2026-05-23 — Workspace resolved, mode: greenfield
- 2026-05-23 — Market intelligence: GO
- 2026-05-24 — ALL 25 DSL DESIGN QUESTIONS COMPLETE
- 2026-05-24 — PHASE 1 STARTED
- 2026-05-24 — ALL 7/7 SPIKES RESOLVED
- 2026-05-24 — PRD COMPLETE (109 BCs)
- 2026-05-24 — ARCHITECTURE COMPLETE (14 ADRs, 15 VPs, 20 crates)
- 2026-05-24 — UX SPEC COMPLETE (10 screens, 5 flows)
- 2026-05-25 — ADVERSARIAL CONVERGENCE (17 passes, 69 findings, 3/3 clean)
- 2026-05-25 — PHASE 1 APPROVED — Phase 2 authorized
- 2026-05-25 — PHASE 2 CONVERGED (22 passes, 96+ findings, 3/3 clean — passes 20-21-22)
- 2026-05-25 — PHASE 2 COMPLETE — Awaiting human approval for Phase 3
- 2026-05-25 — PHASE 2 APPROVED — Phase 3 authorized
- 2026-05-25 — PHASE 3 STARTED — Wave 1, STORY-001 (IR Core Types) in progress
- 2026-05-26 — STORY-001 MERGED (PR #1, b46f9929) — IR Core Types, 157 tests, 6 adversarial passes (20→6→1→0→0→0), 3/3 clean
- 2026-05-26 — STORY-002 MERGED (PR #2, ae344a83) — Plugin Trait API, 10 surfaces, 106 tests, 4 adversarial passes (6→0→0→0), 3/3 clean
- 2026-05-26 — STORY-004 MERGED (PR #3, 2489e8d8) — Value System + EMU Types, 212 tests, 4 adversarial passes (3→0→0→0), 3/3 clean
- 2026-05-26 — STORY-005 MERGED (PR #4, cc04ba50) — Lexer + Mode-Based Tokenization, 45 tests, 6 adversarial passes (7→1→0→0→0→0), 3/3 clean
- 2026-05-26 — STORY-006 MERGED (PR #5, 769bf8e3) — Parser Core (deck/slide/fields/indentation), 101 tests, 5 adversarial passes (7→4→0→0→0), 3/3 clean
- 2026-05-26 — STORY-007 MERGED (PR #6, f178c179) — @for, @if/@elif/@else, expression parser, template interpolation, 142 tests, 4 adversarial passes (2→0→0→0), 3/3 clean

## Decision Documents (canonical references)

| Document | Scope |
|----------|-------|
| planning/q1-decision-final.md | Computation, formats, registers, charts, math, brand, roadmap |
| planning/q2-decision-final.md | 31 types, aliases, components, DSL syntax per type |
| planning/q3-decision-final.md | Plugin-first architecture, 10 surfaces, trait signatures, registry |
| planning/q4-q15-decisions.md | Template binding, output, a11y, shape DSL, inline, variants, includes, comments, conditionals, assets |
| planning/q16-q25-decisions.md | Versioning, strict mode, i18n, packages, workspace, defaults, keywords, errors, mixins, merge |

## Quality Bar (Non-Negotiable)

Production-grade-from-day-1. Key gates:
- `#![forbid(unsafe_code)]` (except FFI if added); zero `.unwrap()` outside tests
- `clippy::pedantic` clean; `#![warn(missing_docs)]` on public APIs
- Kani proofs for pure-core functions; cargo-fuzz in CI; cargo-mutants with documented kill-rate budget
- WCAG AA on web preview + HTML; PDF/UA-1 on PDF
- Multi-renderer visual parity (PowerPoint, Keynote, Google Slides, LibreOffice)
- < 500ms cold build, < 50ms incremental (25-slide deck)
- Signed releases + SBOM; cargo audit + cargo deny in CI
- Cross-platform binaries (macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64)
- Full quality bar table: see CLAUDE.md

## Version Roadmap (summary)

- **v1.0**: Data-reactive DSL (rungs 1-9), 31 slide types, 5 output formats, 3 writing registers, plugin-first (10 surfaces, 20 crates), charts (plotters), math ($...$+@{var}), Mermaid diagrams, full package model, workspace, shape DSL, brand bridge, WCAG AA, production-grade
- **v1.x**: More chart types, figure auto-numbering, DOCX Level 2, comemo incremental, slideforge fmt
- **v2**: User-defined functions, SmartArt, animations, native OOXML charts, plugin connectors, mixins, components, LaTeX/Beamer output
- **v3**: Morph transitions, video export, multi-document pipeline, marketplace, collaboration

## Drift Items

_(None)_
