---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-1-spec-crystallization
status: IN_PROGRESS_PHASE_1
last_updated: 2026-05-24
---

# Slideforge — Factory State

## What Is This Project?
slideforge is a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. It generates branded .pptx, .docx, .pdf, .html, and a live web preview from a single indentation-significant DSL (.sf files) with data binding, iteration, conditionals, and a plugin-first architecture.

**Tagline:** "Branded documents from structured data — one source, every format."

**Repository:** https://github.com/drbothen/slideforge
**Workspace:** /Users/jmagady/Dev/slideforge
**Factory worktree:** .factory/ on branch `factory-artifacts`

## Current Status: PHASE 1 IN PROGRESS (2026-05-24)

Pre-Phase-1 work complete. Phase 1 actively running. Resume from "Phase 1 Progress" section below.

## Phase 1 Progress (as of 2026-05-24)

### Phase 1 Step Status

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| P1-00 | devops-engineer | DONE | MSRV 1.85→1.88 in rust-toolchain.toml + Cargo.toml + ci.yml |
| P1-01 | business-analyst | DONE | .factory/specs/domain-spec/ (12 files, L2-INDEX.md) |
| P1-02 | product-owner | IN_PROGRESS | L3 PRD + BCs — LAUNCHING NOW (L2 ready) |
| P1-03a-S1 | architect | IN_PROGRESS | S1 code exists (.factory/planning/spikes/S1-code/). No report. Re-running. |
| P1-03a-S2 | architect | DONE | .factory/planning/spikes/S2-pdf-backend-evaluation.md + S2-code/ |
| P1-03a-S3 | architect | DONE | .factory/planning/spikes/S3-wcag-tooling-choice.md + S3-code/ |
| P1-03a-S4 | architect | IN_PROGRESS | S4 code exists (.factory/planning/spikes/S4-code/). No report. Re-running. |
| P1-03a-S5 | architect | DONE | .factory/planning/spikes/S5-brand-synthesis-layout-taxonomy.md + S5-code/ |
| P1-03a-S6 | architect | DONE | .factory/planning/spikes/S6-multi-renderer-parity.md + S6-code/ |
| P1-03a-S14 | architect | DONE | .factory/planning/spikes/S14-mermaid-diagram-engine.md + S14-code/ |
| P1-03b | architect | NOT_STARTED | Architecture + ADRs — blocked on P1-02 + all spikes DONE |
| P1-04 | product-owner | NOT_STARTED | PRD revision — blocked on P1-03b |
| P1-05 | ux-designer | NOT_STARTED | UX spec — blocked on P1-02 |
| P1-06 | devops-engineer | NOT_STARTED | CI/CD matrix expansion — blocked on P1-03b |
| P1-07 | adversary | NOT_STARTED | 3 clean passes — blocked on P1-03b+P1-04+P1-05+P1-06 |
| P1-08 | consistency-validator | NOT_STARTED | Cross-doc audit — blocked on P1-07 |
| P1-09 | HUMAN | NOT_STARTED | Approval gate — blocked on P1-08 |

### Spike Verdicts (resolved)

| ID | Verdict | Key decision |
|----|---------|-------------|
| S2 | ADOPT pdf-writer + krilla | PDF/UA-1 decisive. Custom SlideTagEngine on pdf-writer/krilla. |
| S3 | axe-core/playwright + veraPDF + custom OOXML linter | Web preview MUST use SVG not canvas. chumsky validate() confirmed for error accumulation. |
| S5 | 31 layouts (11 standard + 20 custom) | 31 types → 20 custom layouts (many-to-one). Dark: clrMapOvr + explicit solidFill fallback. |
| S6 | SSIM≥0.99 + PSNR≥35dB dual gate | visual-parity-contract.md SSIM 0.97 is too lenient — must update to ≥0.99. Render: PPTX→PDF(LO Still)→PNG(300 DPI). |
| S14 | ADOPT mermaid-rs-renderer v0.2.2 | 65µs–3ms/diagram, pure SVG, 8/8 diagram types pass, PPTX-safe without post-processing. |

### Spikes still in-progress

| ID | Code status | Missing | Action |
|----|------------|---------|--------|
| S1 | .factory/planning/spikes/S1-code/src/main.rs (1,454 lines). Did NOT compile. | Spike report | Re-running architect: fix compile + write report. |
| S4 | .factory/planning/spikes/S4-code/src/ (6 files, 1,539 lines). Compiled (s4_bench, s4_demo). | Spike report | Re-running architect: test binaries + write report. |

### Domain Spec (L2) outputs

- .factory/specs/domain-spec/L2-INDEX.md — 30 CAPs, 22 DIs, 20 DECs, 14 ASMs, 16 Rs, 18 FMs
- .factory/specs/domain-spec/capabilities.md, entities.md, invariants.md, events.md, edge-cases.md, assumptions.md, risks.md, failure-modes.md, differentiators.md, ubiquitous-language.md
- .factory/planning/domain-research.md (34KB) — 4 bounded contexts (Authoring/Branding/Layout/Export), PDF/UA gap confirmed, font metric divergence is #1 multi-format challenge

### What to do next (resume instructions)

1. **S1 re-run**: dispatch architect with S1-code/ to fix compile errors and write S1 spike report
2. **S4 re-run**: dispatch architect with S4-code/ to run binaries and write S4 spike report
3. **PRD launch**: dispatch product-owner with L2 domain spec + all Q decisions to produce L3 PRD + BCs
4. **Architecture + ADRs**: dispatch architect AFTER (PRD complete) AND (S1+S4 reports done)
5. **UX spec**: dispatch ux-designer after PRD draft exists
6. **CI/CD expansion**: dispatch devops-engineer after architecture is done

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
- 2026-05-24 — PHASE 1 STARTED: devops-engineer bumped MSRV to 1.88 (clean build verified)
- 2026-05-24 — Domain research completed (34KB); 4 bounded contexts confirmed; font metric divergence = #1 multi-format risk
- 2026-05-24 — Spike S2 RESOLVED: ADOPT pdf-writer + krilla (PDF/UA-1 decisive axis)
- 2026-05-24 — Spike S3 RESOLVED: axe-core/playwright + veraPDF + custom OOXML linter
- 2026-05-24 — Spike S5 RESOLVED: 31 layouts (11 standard + 20 custom); dark: clrMapOvr + solidFill fallback
- 2026-05-24 — Spike S6 RESOLVED: SSIM≥0.99 + PSNR≥35dB dual gate; visual-parity-contract.md SSIM 0.97 threshold is stale — must be updated to ≥0.99 when architecture agent runs
- 2026-05-24 — Spike S14 RESOLVED: ADOPT mermaid-rs-renderer v0.2.2 (65µs–3ms/diagram, pure SVG, 8/8 types)
- 2026-05-24 — L2 Domain Spec COMPLETE (12 files; 30 CAPs, 22 DIs, 20 DECs, 14 ASMs, 16 Rs, 18 FMs)
- 2026-05-24 — Spike S1 IN_PROGRESS: code exists but did not compile; re-running architect
- 2026-05-24 — Spike S4 IN_PROGRESS: code compiled but no report; re-running architect
- 2026-05-24 — PRD LAUNCHING: dispatching product-owner (L2 domain spec ready)

## Drift Items
_(None)_
