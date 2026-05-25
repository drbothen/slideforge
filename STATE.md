---
project: slideforge
mode: greenfield
created: 2026-05-23
current_phase: phase-1-spec-crystallization
status: READY_FOR_ADVERSARIAL_REVIEW
last_updated: 2026-05-24
prd_bcs: 101
prd_hs: 15
prd_supplements: 4
spikes_resolved: 7
spikes_total: 7
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

## Current Status: READY FOR ADVERSARIAL REVIEW (2026-05-24)

All pre-adversarial Phase 1 steps complete. DTU: not required (no external clones needed). Gene transfusion: behavioral-only (75 behaviors classified, no code ported). CI/CD: 3 workflows committed (ci.yml, release.yml, security.yml), 5-platform matrix, supply-chain audit, visual-diff gate. Next: P1-09 adversarial spec review — 3 consecutive clean passes required before Phase 2.

## Phase 1 Progress (as of 2026-05-24)

### Phase 1 Step Status

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| P1-00 | devops-engineer | DONE | MSRV 1.85→1.88 in rust-toolchain.toml + Cargo.toml + ci.yml |
| P1-01 | business-analyst | DONE | .factory/specs/domain-spec/ (12 files, L2-INDEX.md) |
| P1-02 | product-owner | DONE | .factory/specs/prd.md + 4 supplements + 101 BCs (BC-INDEX.md) + 15 holdout scenarios |
| P1-03a-S1 | architect | DONE | .factory/planning/spikes/S1-ooxmlsdk-pptx-coverage.md — ADOPT-WITH-WORKAROUNDS (55/57 PASS, 2 workarounds) |
| P1-03a-S2 | architect | DONE | .factory/planning/spikes/S2-pdf-backend-evaluation.md — ADOPT pdf-writer + krilla |
| P1-03a-S3 | architect | DONE | .factory/planning/spikes/S3-wcag-tooling-choice.md — axe-core/playwright + veraPDF + custom OOXML linter |
| P1-03a-S4 | architect | DONE | .factory/planning/spikes/S4-chumsky-indentation-parser.md — VIABLE-WITH-CAVEATS (hybrid lexer + chumsky 0.10) |
| P1-03a-S5 | architect | DONE | .factory/planning/spikes/S5-brand-synthesis-layout-taxonomy.md — 31 layouts (11 std + 20 custom) |
| P1-03a-S6 | architect | DONE | .factory/planning/spikes/S6-multi-renderer-parity.md — SSIM≥0.99 + PSNR≥35dB dual gate |
| P1-03a-S14 | architect | DONE | .factory/planning/spikes/S14-mermaid-diagram-engine.md — ADOPT mermaid-rs-renderer v0.2.2 |
| P1-03b | architect | DONE | .factory/specs/architecture/ (ARCH-INDEX + 12 section files, 14 ADRs, module-criticality, feasibility report) |
| P1-04 | product-owner | DONE | PRD complete — 101 BCs, 15 HS, 4 supplements, all spikes resolved |
| P1-05 | architect | DONE | P1-05 (feasibility): PASS-WITH-NOTES — all 5 notes addressed; BC-1.03.005 added |
| P1-06 | ux-designer | DONE | .factory/specs/ux-spec/ (UX-INDEX + 10 screens + 5 flows); .factory/specs/verification-properties/ (VP-INDEX + 7 VPs) |
| P1-07 | devops-engineer | DONE | DTU: not required; gene-transfusion: behavioral-only (75 behaviors); CI/CD: 3 workflows, 5-platform matrix |
| P1-09 | adversary | NOT_STARTED | 3 clean passes — READY TO START |
| P1-10 | consistency-validator | NOT_STARTED | Cross-doc audit — blocked on P1-09 (3-clean) |
| P1-11 | HUMAN | NOT_STARTED | Approval gate — blocked on P1-10 |

### Spike Verdicts (ALL 7 RESOLVED)

| ID | Verdict | Key decision |
|----|---------|-------------|
| S1 | ADOPT-WITH-WORKAROUNDS | ooxmlsdk 0.6.1; 55/57 PASS; 2 workarounds (table raw-XML embed, no Default in Content_Types) |
| S2 | ADOPT pdf-writer + krilla | PDF/UA-1 decisive. Custom SlideTagEngine on pdf-writer/krilla. |
| S3 | axe-core/playwright + veraPDF + custom OOXML linter | Web preview MUST use SVG not canvas. chumsky validate() confirmed for error accumulation. |
| S4 | VIABLE-WITH-CAVEATS | chumsky 0.10; hybrid hand-written lexer + chumsky parser via Stream |
| S5 | 31 layouts (11 standard + 20 custom) | 31 types → 20 custom layouts (many-to-one). Dark: clrMapOvr + explicit solidFill fallback. |
| S6 | SSIM≥0.99 + PSNR≥35dB dual gate | Render: PPTX→PDF(LO Still)→PNG(300 DPI). visual-parity-contract.md threshold updated to ≥0.99. |
| S14 | ADOPT mermaid-rs-renderer v0.2.2 | 65µs–3ms/diagram, pure SVG, 8/8 diagram types pass, PPTX-safe without post-processing. |

### Domain Spec (L2) outputs

- .factory/specs/domain-spec/L2-INDEX.md — 30 CAPs, 22 DIs, 20 DECs, 14 ASMs, 16 Rs, 18 FMs
- .factory/specs/domain-spec/capabilities.md, entities.md, invariants.md, events.md, edge-cases.md, assumptions.md, risks.md, failure-modes.md, differentiators.md, ubiquitous-language.md
- .factory/planning/domain-research.md (34KB) — 4 bounded contexts (Authoring/Branding/Layout/Export), PDF/UA gap confirmed, font metric divergence is #1 multi-format challenge

### Architecture outputs (P1-03b DONE)

- .factory/specs/architecture/ARCH-INDEX.md + 12 section files (system-overview, crate-architecture, plugin-architecture, ir-design, export-architecture, brand-architecture, error-architecture, tooling-selection, verification-architecture, verification-coverage-matrix, purity-boundary-map, dependency-graph)
- .factory/specs/architecture/adr/ — ADR-001 through ADR-014
- .factory/specs/module-criticality.md
- .factory/specs/architecture-feasibility-report.md — PASS-WITH-NOTES (5 notes, all addressed)
- .factory/specs/verification-properties/ — VP-INDEX.md + 7 VP files (VP-001 through VP-008, skipping VP-005)
- .factory/specs/ux-spec/ — UX-INDEX.md + 10 screens (SCR-001 through SCR-010) + 5 flows (FLOW-001 through FLOW-005)
- .factory/specs/behavioral-contracts/BC-1.03.005.md — new BC (feasibility note 5: mermaid-rs-renderer pinning)

### What to do next (resume instructions)

1. **P1-09 — Adversarial spec review**: dispatch adversary against ALL Phase 1 specs. Must achieve 3 consecutive clean passes (BC-5.39.001). Scope: domain-spec, PRD + supplements, architecture + ADRs, UX spec, VPs, cicd-setup, dtu-assessment, gene-transfusion-assessment.
2. **P1-10 — Consistency audit**: dispatch consistency-validator AFTER P1-09 passes 3-clean.
3. **P1-11 — Human approval gate**: human review AFTER P1-10 complete, then Phase 2 begins.

**Adversary scope checklist for P1-09:**
- .factory/specs/domain-spec/ (12 files)
- .factory/specs/prd.md + 4 supplements
- .factory/specs/behavioral-contracts/ (BC-INDEX + all BC files)
- .factory/specs/architecture/ (ARCH-INDEX + 12 sections + 14 ADRs + feasibility)
- .factory/specs/ux-spec/ (UX-INDEX + 10 screens + 5 flows)
- .factory/specs/verification-properties/ (VP-INDEX + 7 VPs)
- .factory/specs/dtu-assessment.md
- .factory/specs/gene-transfusion-assessment.md
- .factory/specs/cicd-setup.md
- .factory/planning/spikes/ (all 7 resolved spikes)

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

## Spikes (ALL RESOLVED 2026-05-24)

| ID | Spike | Severity | Verdict |
|----|-------|----------|---------|
| S1 | ooxmlsdk PPTX coverage validation | HIGH (blocking) | ADOPT-WITH-WORKAROUNDS |
| S2 | PDF backend evaluation | HIGH | ADOPT pdf-writer + krilla |
| S3 | WCAG AA tooling choice | MEDIUM | axe-core/playwright + veraPDF + OOXML linter |
| S4 | chumsky 0.10 indentation parser | MEDIUM | VIABLE-WITH-CAVEATS (hybrid lexer) |
| S5 | Brand synthesis layout taxonomy | HIGH | 31 layouts (11 std + 20 custom) |
| S6 | Multi-renderer parity baseline | HIGH | SSIM≥0.99 + PSNR≥35dB dual gate |
| S14 | Mermaid diagram rendering engine | HIGH | ADOPT mermaid-rs-renderer v0.2.2 |

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
- 2026-05-24 — Spike S1 RESOLVED: ADOPT-WITH-WORKAROUNDS — ooxmlsdk 0.6.1; 55/57 PASS; 2 workarounds identified
- 2026-05-24 — Spike S4 RESOLVED: VIABLE-WITH-CAVEATS — hybrid hand-written lexer + chumsky 0.10 via Stream
- 2026-05-24 — PRD COMPLETE: 101 BCs (BC-1.01 through BC-5.05), 15 holdout scenarios, 4 supplements (error-taxonomy, interface-definitions, nfr-catalog, test-vectors)
- 2026-05-24 — ALL 7/7 SPIKES RESOLVED: S1 S2 S3 S4 S5 S6 S14 — P1-03b (architecture) is unblocked
- 2026-05-24 — READY FOR ARCHITECTURE FEASIBILITY REVIEW: P1-03b + P1-05 can run in parallel
- 2026-05-24 — P1-05 ARCHITECTURE FEASIBILITY: PASS-WITH-NOTES — 5 notes identified and fully addressed; BC-1.03.005 added for mermaid-rs-renderer pinning
- 2026-05-24 — P1-03b ARCHITECTURE COMPLETE: ARCH-INDEX + 12 section files + 14 ADRs (ADR-001–ADR-014) + module-criticality.md + feasibility-report
- 2026-05-24 — P1-06 UX SPEC COMPLETE: UX-INDEX + 10 screens (SCR-001–SCR-010) + 5 flows (FLOW-001–FLOW-005); 7 VPs (VP-001–VP-008) + VP-INDEX
- 2026-05-24 — BC-1.03.005 ADDED: mermaid-rs-renderer version-pinning contract (feasibility note 5)
- 2026-05-24 — P1-07 DTU ASSESSMENT COMPLETE: DTU_REQUIRED=false — all external services (mermaid-rs-renderer, LibreOffice headless, axe-core) are pure crate deps or test tools; no clone infrastructure required
- 2026-05-24 — P1-07 GENE TRANSFUSION COMPLETE: behavioral-only — 75 behaviors classified from Python reference; 0 code ported; Rust implementation starts from scratch with spec as truth
- 2026-05-24 — P1-07 CI/CD SETUP COMPLETE: 3 workflows (ci.yml expanded, release.yml new, security.yml new); 5-platform matrix (macOS arm64+x86_64, Linux x86_64+musl, Windows x86_64); cargo-deny, nextest, criterion gate, visual-diff.py, SBOM stub — all committed to main
- 2026-05-24 — ALL PRE-ADVERSARIAL STEPS COMPLETE: STATUS → READY_FOR_ADVERSARIAL_REVIEW

## Drift Items
_(None)_
