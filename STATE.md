---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-1-spec-crystallization
status: READY_TO_START_PHASE_1
last_updated: 2026-05-24
---

# Slideforge — Factory State

## What Is This Project?
slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. It generates branded .pptx, .docx, .pdf, .html, and a live web preview from a single indentation-significant DSL (.sf files) with data binding, iteration, conditionals, and a plugin-first architecture.

**Tagline:** "Branded documents from structured data — one source, every format."

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

## Current Status: READY TO START PHASE 1

All pre-Phase-1 work is complete:
- All factory bootstrapped (worktree, STATE.md, seed ingested)
- All Market intelligence: GO (medium confidence)
- All Toolchain preflight: PASS-WITH-NOTES (MSRV bump needed: 1.85->1.88)
- All Brief validated and sharded (608-word slim core + 10 modular specs)
- All 25 DSL design questions decided (Q1-Q25)
- All 14 research threads completed (R1-R14)
- All Quality bar locked (production-grade-from-day-1)
- All decisions reconciled and consistent across documents

## What to Do Next (Phase 1 Spec Crystallization)

### Immediate first actions (dispatch in parallel):
1. **business-analyst** -> L2 domain spec (entities, relationships, processes, invariants)
   - Input: .factory/specs/product-brief.md + .factory/planning/q1-decision-final.md
2. **architect** -> Spike S1 (ooxmlsdk PPTX coverage validation — code + binaries, real .pptx generated)
   - Input: .factory/planning/ooxml-foundations.md + .factory/planning/brand-template-patterns.md
3. **dx-engineer/devops-engineer** -> Bump MSRV 1.85->1.88 in Cargo.toml + ci.yml

### Phase 1 full sequence:
| Step | Agent | Output | Depends on |
|------|-------|--------|-----------|
| P1-01 | business-analyst | L2 domain spec | — |
| P1-02 | product-owner | L3 PRD with BC-S.SS.NNN behavioral contracts | P1-01 |
| P1-03a | architect | Spikes S1-S6 (code + binaries) | — |
| P1-03b | architect | Architecture doc + ADR-001..014 | P1-02 + P1-03a |
| P1-04 | product-owner | PRD revision from architect feedback | P1-03b |
| P1-05 | ux-designer | UX spec (CLI + web preview wireframes) | P1-02 |
| P1-06 | devops-engineer | CI/CD matrix expansion | P1-03a (MSRV bump) |
| P1-07 | adversary | Adversarial spec review (3 clean passes) | P1-03b + P1-04 + P1-05 + P1-06 |
| P1-08 | consistency-validator | Cross-doc consistency audit | P1-07 |
| P1-09 | HUMAN | Approval gate | P1-08 |

### Key constraints for Phase 1 agents:
- Spike depth: CODE + BINARIES (not design-only). Real .pptx files generated. Real parsers built.
- Production-grade quality bar: see "Quality Bar" section below
- Plugin-first architecture: 10 extensibility surfaces, 19 crates
- 31 slide types (not 23 — the original seed is superseded)
- All 5 output formats (PPTX, DOCX, PDF, HTML, web preview) are v1.0 scope

## Decision Documents (canonical references)

| Document | Scope | Lines |
|----------|-------|-------|
| planning/q1-decision-final.md | Computation, formats, registers, charts, math, brand, roadmap | ~615 |
| planning/q2-decision-final.md | 31 types, aliases, components, DSL syntax per type | ~340 |
| planning/q3-decision-final.md | Plugin-first architecture, 10 surfaces, trait signatures, registry | ~220 |
| planning/q4-q15-decisions.md | Template binding, output, a11y, shape DSL, inline, variants, includes, comments, conditionals, assets | ~229 |
| planning/q16-q25-decisions.md | Versioning, strict mode, i18n, packages, workspace, defaults, keywords, errors, mixins, merge | ~162 |

## Research Documents (foundation for Phase 1)

| ID | File | Key finding |
|----|------|-------------|
| R1 | planning/python-reference-deep-read.md | 23 types catalogued, silent fallbacks, string-prefix bold = anti-pattern |
| R2 | planning/brand-template-patterns.md | 31 layouts needed, clrMap inheritance, element ordering strict |
| R3 | planning/dsl-competitor-analysis.md | 70+ citations, #1 differentiator = brand-template-first |
| R4 | planning/ooxml-foundations.md | ooxmlsdk = serializer only, element ordering enforced |
| R5 | planning/wcag-for-slides.md | 23 criteria, SVG > canvas for a11y, axe-core |
| R6 | planning/ir-prior-art.md | Two-IR model, Pandoc ADT + Typst Frame, Hash-stable |
| R7 | planning/composition-prior-art.md | set rules + variants, no user components v1.0 |
| R8 | planning/writing-register-research.md | notes != report (McKinsey/NIST/military confirm) |
| R9 | planning/document-content-vocabulary.md | 67 elements, 19 v1.0-core, charts SVG |
| R10 | planning/pptx-element-taxonomy.md | 160+ elements, 35 v1.0-core, charts biggest gap |
| R11 | planning/math-latex-research.md | $...$ + @{var}, pulldown-latex -> OMML |
| R12 | planning/template-binding-research.md | Overlay v1.0, multi-master v2, Google Slides degrades |
| R13 | planning/raw-escape-hatch-research.md | No raw to users, shape DSL instead, Typst's philosophy |
| R14 | planning/workspace-model-research.md | Cargo-style workspace + .sfconfig cascade |

## Spikes (architect must resolve in Phase 1)

| ID | Spike | Severity | Depth |
|----|-------|----------|-------|
| S1 | ooxmlsdk PPTX coverage validation | HIGH (blocking) | Code + binaries |
| S2 | PDF backend evaluation | HIGH | Code + binaries |
| S3 | WCAG AA tooling choice | MEDIUM | Code + binaries |
| S4 | chumsky 0.10 indentation parser | MEDIUM | Code + binaries |
| S5 | Brand synthesis layout taxonomy | HIGH | Code + binaries |
| S6 | Multi-renderer parity baseline | HIGH | Code + binaries |
| S14 | Mermaid diagram rendering engine | HIGH | Code + binaries |

## ADRs (architect must produce in Phase 1)

ADR-001 through ADR-014 (see planning/q1-decision-final.md Section 0 and specs/decisions-applied.md for the full list).

## Quality Bar (Non-Negotiable)

Production-grade-from-day-1. Key gates:
- `#![forbid(unsafe_code)]` (except FFI if added)
- Zero `.unwrap()` outside tests
- `clippy::pedantic` clean
- `#![warn(missing_docs)]` on public APIs
- Kani proofs for pure-core functions
- cargo-fuzz harness in CI
- cargo-mutants with documented kill-rate budget
- WCAG AA on web preview + HTML
- PDF/UA-1 on PDF
- Multi-renderer visual parity (PowerPoint, Keynote, Google Slides, LibreOffice)
- < 500ms cold build, < 50ms incremental (25-slide deck)
- Signed releases + SBOM
- cargo audit + cargo deny in CI
- Cross-platform binaries (macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64)

## Version Roadmap (summary)

- **v1.0**: Data-reactive DSL (rungs 1-9), 31 slide types, 5 output formats, 3 writing registers, plugin-first (10 surfaces, 19 crates), charts (plotters), math ($...$+@{var}), Mermaid diagrams, full package model, workspace, shape DSL, brand bridge (bidirectional PPTX+DOCX), WCAG AA, production-grade
- **v1.x**: More chart types, figure auto-numbering, DOCX Level 2, comemo incremental, slideforge fmt
- **v2**: User-defined functions, SmartArt, animations, native OOXML charts, plugin connectors, mixins, components, LaTeX/Beamer output, AI-assisted content, presentation mode
- **v3**: Morph transitions, video export, multi-document pipeline, marketplace, collaboration

## Decisions Log (chronological)

- 2026-05-23 — Workspace resolved, mode: greenfield
- 2026-05-23 — factory-artifacts branch + worktree initialized
- 2026-05-23 — Seed ingested, canonical brief at specs/product-brief.md
- 2026-05-23 — 7 Open Questions answered (slideforge / indented / @include / both+extract+full-synthesis / all-3-exporters / DSL-only Python / Typst-web-preview)
- 2026-05-23 — Production-grade-from-day-1 declared
- 2026-05-23 — Market intelligence: GO
- 2026-05-23 — Toolchain preflight: PASS-WITH-NOTES
- 2026-05-23 — Brief validated, sharded, parity contract locked
- 2026-05-24 — Q1 LOCKED (data-reactive, 5 formats, 3 registers, charts, math, plugin-first, brand bridge)
- 2026-05-24 — Q2 LOCKED (31 types, aliases, reserved components)
- 2026-05-24 — Q3 LOCKED (plugin-first, 10 surfaces, dog-food, Mermaid v1.0)
- 2026-05-24 — Q4-Q15 LOCKED (template overlay, a11y, shape DSL, interpolation, variants, fragments, etc.)
- 2026-05-24 — Q16-Q25 LOCKED (packages, workspace, defaults, keywords, errors, merge)
- 2026-05-24 — ALL 25 DSL DESIGN QUESTIONS COMPLETE
- 2026-05-24 — Reconciliation pass: all docs consistent

## Drift Items
_(None)_
