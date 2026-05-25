---
document_type: architecture-index
level: L4
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
phase: 1b
deployment_topology: single-service
traces_to: .factory/specs/prd.md
inputs:
  - .factory/specs/architecture-feasibility-report.md
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/prd-supplements/nfr-catalog.md
  - .factory/specs/domain-spec/invariants.md
  - .factory/planning/spikes/S1-ooxmlsdk-pptx-coverage.md
  - .factory/planning/spikes/S2-pdf-backend-evaluation.md
  - .factory/planning/spikes/S3-wcag-tooling-choice.md
  - .factory/planning/spikes/S4-chumsky-indentation-parser.md
  - .factory/planning/spikes/S5-brand-synthesis-layout-taxonomy.md
  - .factory/planning/spikes/S6-multi-renderer-parity.md
  - .factory/planning/spikes/S14-mermaid-diagram-engine.md
  - .factory/planning/q3-decision-final.md
input-hash: "[pending compute-input-hash]"
---

# ARCH-INDEX — slideforge v1.0 Architecture

## Architecture Verdict

APPROVED. Feasibility review PASS-WITH-NOTES (2026-05-24). All five notes are
incorporated into the section files listed below.

---

## Deployment Topology

`single-service` — one CLI binary, one tech stack (Rust stable), no network services
required at runtime. The web preview server (`slideforge serve`) launches an embedded
axum process but this is a local development feature, not a deployed service.

---

## Subsystem Registry

| SS-ID | Subsystem Name | Primary Crates | Notes |
|-------|---------------|----------------|-------|
| SS-01 | DSL Parser | slideforge-syntax | Pure core. Kani-amenable. |
| SS-02 | Evaluator | slideforge-eval | Pure core. Kani-amenable. |
| SS-03 | Validation | slideforge-validate | Pure core. Owns ALL compile-time checks. |
| SS-04 | Brand | slideforge-brand | Effectful shell. Reads file I/O; synthesis is pure. |
| SS-05 | Layout Engine | slideforge-layout | Pure core. Deck → LaidOutDeck. |
| SS-06 | PPTX Export | slideforge-pptx | Effectful shell. ooxmlsdk 0.6.1. |
| SS-07 | PDF Export | slideforge-pdf | Effectful shell. pdf-writer + krilla. Phase 4. |
| SS-08 | DOCX Export | slideforge-docx | Effectful shell. Phase 3. |
| SS-09 | HTML/Preview | slideforge-preview | Effectful shell. axum + WebSocket + SVG. |
| SS-10 | Data Sources | slideforge-data | Effectful shell. All external I/O. |
| SS-11 | Diagrams | slideforge-diagrams | Effectful shell (font DB scan). Mostly pure. |
| SS-12 | Charts | slideforge-charts | Pure core. plotters SVG generation. |
| SS-13 | Math | slideforge-math | Pure core. LaTeX transformation. |
| SS-14 | Plugin API | slideforge-plugin-api | Pure types. 10 trait surfaces. |
| SS-15 | IR Types | slideforge-types | Pure types. Hash + Eq + Clone on all. |
| SS-16 | Package Mgmt | slideforge-package | Effectful shell. Git + sf.lock. |
| SS-17 | Workspace Config | slideforge-config | Effectful shell. File I/O only. |
| SS-18 | CLI Orchestrator | slideforge-cli | Effectful shell. All lifecycle I/O. |
| SS-19 | Slide Type Impls | slideforge-types | 31 SlideType implementations. |

---

## Document Map

| Section | File | Primary Consumer | Purpose |
|---------|------|-----------------|---------|
| System Overview | system-overview.md | All | Four-stage pipeline, bounded contexts, data flow |
| Crate Architecture | crate-architecture.md | implementer, devops | 19-crate workspace, dependency graph, purity boundaries |
| Plugin Architecture | plugin-architecture.md | implementer, story-writer | 10 surfaces, trait signatures, registry pattern |
| IR Design | ir-design.md | implementer | Deck + LaidOutDeck, type constraints, EMU system |
| Error Architecture | error-architecture.md | implementer, test-writer | Error propagation, span threading, miette rendering |
| Brand Architecture | brand-architecture.md | implementer | Brand synthesis, 31-layout taxonomy, dark-layout invariant |
| Export Architecture | export-architecture.md | implementer | Per-format export strategy, fallback paths |
| Dependency Graph | dependency-graph.md | devops, security | External dep manifest, pinned versions, cargo deny policy |
| Verification Architecture | verification-architecture.md | formal-verifier, test-writer | Provable properties, Kani proofs, coverage matrix |
| Purity Boundary Map | purity-boundary-map.md | implementer, formal-verifier | Pure core vs effectful shell classification |
| Tooling Selection | tooling-selection.md | implementer, devops | Technology choices with spike references |
| Verification Coverage Matrix | verification-coverage-matrix.md | formal-verifier | VP-to-module mapping |

---

## ADR Registry

| ADR | Title | Status |
|-----|-------|--------|
| ADR-001 | PPTX generation via ooxmlsdk 0.6.1 | Accepted |
| ADR-002 | Visual parity testing with LibreOffice headless | Accepted |
| ADR-003 | PDF generation via pdf-writer + krilla | Accepted |
| ADR-004 | Include/import resolution semantics | Accepted |
| ADR-005 | Two-IR model (Deck + LaidOutDeck) | Accepted |
| ADR-006 | Plugin-first architecture with 10 surfaces | Accepted |
| ADR-007 | Data-reactive computation model (rungs 1-9) | Accepted |
| ADR-008 | Web preview via SVG + axum + WebSocket | Accepted |
| ADR-009 | Parser via chumsky 0.10 hybrid approach | Accepted |
| ADR-010 | Brand bidirectional bridge | Accepted |
| ADR-011 | WCAG AA toolchain | Accepted |
| ADR-012 | Error accumulation and miette rendering | Accepted |
| ADR-013 | Integer EMU coordinate system | Accepted |
| ADR-014 | Mermaid via mermaid-rs-renderer | Accepted |

---

## Feasibility Notes Resolved

| Note | Summary | Resolution Location |
|------|---------|-------------------|
| Note 1 | krilla tagged PDF fallback path | export-architecture.md §SlideTagEngine |
| Note 2 | All compile-time validation to slideforge-validate | crate-architecture.md, verification-architecture.md |
| Note 3 | LaTeX-to-OMML is spike S-MATH-01 dependency | ir-design.md §Math IR |
| Note 4 | clrMapOvr + explicit white runs as tested invariant | brand-architecture.md §Dark Layout Invariant |
| Note 5 | BC-1.03.005 HTTP allowlist (product-owner addition) | behavioral-contracts/BC-1.03.005.md |
