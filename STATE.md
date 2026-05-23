---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-1-spec-crystallization
status: PENDING_HUMAN_INPUT
last_updated: 2026-05-23
---

# Slideforge Factory State

## Pipeline Mode
**Greenfield** — new Rust + DSL PowerPoint generator project. Scaffolding pre-applied (Cargo workspace + 7 crate stubs already committed on main @ e44aacd).

## Workspace
- Repo: `/Users/jmagady/Dev/slideforge`
- Factory worktree: `.factory/` on branch `factory-artifacts`
- Brief: `.factory/specs/product-brief.md` (canonical) — sourced from `.factory/seed/PROJECT-SEED.md`
- Reference materials: `.factory/seed/reference/` (Python implementation — IMMUTABLE source-of-truth for visual behavior)

## Phase Status

| Phase | Status | Gate |
|-------|--------|------|
| Pre-Pipeline (toolchain preflight) | PENDING | — |
| Market Intelligence | PENDING | Human reviews GO/CAUTION/STOP |
| Phase 0 (Codebase Ingestion) | N/A — scaffolding is empty stubs, brief is canonical | — |
| Phase 1 (Spec Crystallization) | BLOCKED | Awaiting human answers to 7 Open Questions (seed §11) |
| Phase 2 (Story Decomposition) | PENDING | — |
| Phase 3 (TDD Implementation) | PENDING | — |
| Phase 4 (Holdout Eval) | PENDING | — |
| Phase 5 (Adversarial) | PENDING | — |
| Phase 6 (Hardening) | PENDING | — |
| Phase 7 (Convergence) | PENDING | — |

## Open Questions (BLOCKING Phase 1)
Per PROJECT-SEED.md §11, the factory MUST get human input on:
1. **Q1 — Project name** (seed proposes `slideforge`; alternatives: Slidesmith, Decktype, Brandeck, Forgedeck)
2. **Q2 — DSL syntax style** (indentation-significant vs. brace-delimited)
3. **Q3 — Multi-file project support** (single-file deck vs. @include composition)
4. **Q4 — Brand template input format** (.pptx file vs. .toml synthesized)
5. **Q5 — PDF/HTML exporters in v1.0 or post-v1.0** (currently optional in Phase 4)
6. **Q6 — Python binding API style** (dict-based for migration vs. DSL-only)
7. **Q7 — Live preview architecture** (PPTX rebuild / HTML / GUI / Typst-style)

## Decisions Log
- 2026-05-23 — Workspace resolved to `/Users/jmagady/Dev/slideforge`
- 2026-05-23 — Mode: greenfield (scaffolding pre-applied counts as Phase 0 stub)
- 2026-05-23 — `factory-artifacts` orphan branch + worktree initialized (commit 562ccab)
- 2026-05-23 — Seed bundle relocated from `main:seed/` to `factory-artifacts:.factory/seed/`
- 2026-05-23 — Canonical brief established at `.factory/specs/product-brief.md`

## Drift Items
_(None yet)_

## Next Action
Orchestrator to surface the 7 Open Questions to the human via AskUserQuestion. Upon answers, run market-intelligence-assessment, then validate-brief, then phase-1-spec-crystallization.
