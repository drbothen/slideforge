---
document_type: prd
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/specs/domain-spec/L2-INDEX.md
  - .factory/specs/domain-spec/capabilities.md
  - .factory/specs/domain-spec/invariants.md
  - .factory/specs/domain-spec/edge-cases.md
  - .factory/specs/domain-spec/assumptions.md
  - .factory/specs/domain-spec/risks.md
  - .factory/specs/domain-spec/failure-modes.md
  - .factory/specs/domain-spec/differentiators.md
  - .factory/specs/product-brief.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q2-decision-final.md
  - .factory/planning/q3-decision-final.md
  - .factory/planning/q4-q15-decisions.md
  - .factory/planning/q16-q25-decisions.md
  - .factory/planning/spikes/S2-pdf-backend-evaluation.md
  - .factory/planning/spikes/S5-brand-synthesis-layout-taxonomy.md
  - .factory/planning/spikes/S6-multi-renderer-parity.md
  - .factory/planning/spikes/S14-mermaid-diagram-engine.md
input-hash: "[pending compute-input-hash]"
traces_to: .factory/specs/domain-spec/L2-INDEX.md
supplements:
  - .factory/specs/prd-supplements/interface-definitions.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/prd-supplements/test-vectors.md
  - .factory/specs/prd-supplements/nfr-catalog.md
---

# Product Requirements Document — slideforge v1.0

---

## Section 1: Product Overview

### 1.1 Problem Statement

Engineering teams, incident responders, and internal tooling engineers must produce
branded, data-driven presentations and reports regularly. Current solutions require
either manual drag-and-drop (PowerPoint), Python dicts with no type safety or error
recovery (python-pptx), or tools that cannot produce PPTX output with real brand
templates (Typst, Marp, Slidev). The result: slow authoring, brittle scripts,
accessibility-ignorant output, and documents that cannot be generated deterministically
from version-controlled source.

### 1.2 Vision

slideforge is a data-reactive branded document platform that compiles a single
indentation-significant DSL (`.sf` files) into `.pptx`, `.docx`, `.pdf`, `.html`,
and a live web preview through a six-stage pipeline (Parse → Evaluate → Brand →
Validate → Layout → Export) with a plugin-first architecture and compile-time accessibility
enforcement.

A single `.sf` file is the only source. Brand configuration is loaded from existing
`.pptx` templates or synthesized from `brand.toml`. Output is deterministic,
version-controllable, and CI-runnable. Every visual element must declare its alt text
at compile time — accessibility is not optional or deferred.

### 1.3 Key Differentiators

| ID | Differentiator | BC Backing |
|----|---------------|-----------|
| D-001 | Live data reactivity — `@data` binds external sources; watch mode auto-refreshes | BC-1.03.001, BC-1.03.002, BC-5.05.001 |
| D-002 | Opinionated shared slide vocabulary — 31 named types with enforced semantics | BC-3.01.001, BC-3.01.002, BC-1.09.001 |
| D-003 | Production-grade multi-format from one source — PPTX+DOCX+PDF+HTML+preview | BC-4.01.001, BC-4.02.001, BC-4.03.001, BC-4.03.003 |
| D-004 | OOXML brand template bridge — load any .pptx; synthesize from brand.toml; extract back | BC-2.01.001, BC-2.01.002, BC-2.01.003 |
| D-005 | Compile-time accessibility enforcement — missing alt = compile error | BC-5.01.001, BC-5.01.003 |
| D-006 | Single binary, zero runtime dependencies | BC-1.12.001, BC-4.03.002 |
| D-007 | Plugin-first extensibility with dog-fooding guarantee | BC-5.02.001, BC-5.02.002 |

### 1.4 Target Users

- Engineering teams generating regular branded reports and briefs from data
- Incident response teams producing executive briefs from structured event data
- Internal tooling needing deterministic, version-controllable presentation output
- python-pptx users who want a typed DSL with error recovery and multi-format output

### 1.5 Out of Scope (v1.0)

- Manual authoring GUI / WYSIWYG editor
- User-defined functions (deferred to v2.0)
- Multi-master deck architecture (deferred to v2.0)
- DOCX Level 2 flowing document layout (deferred to v1.x)
- SmartArt generation (deferred to v2.0)
- Animation and transition effects
- Slide collaboration (real-time editing)
- Plugin marketplace / package registry (git-based distribution only in v1.0)
- WASM plugin system (v2.0)

---

## Section 2: Behavioral Contracts Index

> Full index lives at `.factory/specs/behavioral-contracts/BC-INDEX.md`.
> Individual contracts at `.factory/specs/behavioral-contracts/BC-S.SS.NNN.md`.
> Below are grouped summaries by bounded context.

### 2.1 Authoring Bounded Context (Section 1)

**Subsystems:** DSL Parsing, Variable Interpolation, Data Binding, Iteration, Conditionals,
Multi-File Composition, Variants, Set Rules, Aliases, Math, Charts, Diagrams, Versioning,
Writing Registers, Diagnostics.

| Subsection | BCs | P0 | P1 |
|-----------|-----|----|----|
| 1.01 DSL Source Parsing | 6 | 6 | 0 |
| 1.02 Variable Interpolation | 5 | 5 | 0 |
| 1.03 Data Binding | 7 | 3 | 4 |
| 1.04 Iteration | 3 | 3 | 0 |
| 1.05 Conditionals | 2 | 2 | 0 |
| 1.06 Multi-File Composition | 4 | 3 | 1 |
| 1.07 Variants | 5 | 5 | 0 |
| 1.08 Set Rules | 3 | 3 | 0 |
| 1.09 Aliases | 2 | 0 | 2 |
| 1.10 Math/LaTeX | 3 | 0 | 3 |
| 1.11 Charts | 2 | 0 | 2 |
| 1.12 Diagrams | 3 | 0 | 3 |
| 1.13 DSL Versioning | 1 | 0 | 1 |
| 1.14 Writing Registers | 4 | 4 | 0 |
| 1.15 Diagnostics | 3 | 3 | 0 |

### 2.2 Branding Bounded Context (Section 2)

| Subsection | BCs | P0 | P1 |
|-----------|-----|----|----|
| 2.01 Brand Loading/Synthesis | 6 | 6 | 0 |
| 2.02 Per-Slide Brand Overlay | 2 | 0 | 2 |

### 2.3 Layout Bounded Context (Section 3)

| Subsection | BCs | P0 | P1 |
|-----------|-----|----|----|
| 3.01 31 Built-in Slide Types | 3 | 3 | 0 |
| 3.02 Document Section Generation | 2 | 2 | 0 |
| 3.03 Compile-Time Validation | 4 | 4 | 0 |
| 3.04 Shape DSL | 2 | 0 | 2 |
| 3.05 Rich Inline Formatting | 1 | 0 | 1 |

### 2.4 Export Bounded Context (Section 4)

| Subsection | BCs | P0 | P1 |
|-----------|-----|----|----|
| 4.01 PPTX Export | 6 | 6 | 0 |
| 4.02 DOCX Export | 2 | 2 | 0 |
| 4.03 PDF/HTML/Web Preview | 5 | 4 | 1 |

### 2.5 Cross-Cutting Capabilities (Section 5)

| Subsection | BCs | P0 | P1 |
|-----------|-----|----|----|
| 5.01 Accessibility Validation | 5 | 5 | 0 |
| 5.02 Plugin Architecture | 2 | 2 | 0 |
| 5.03 Package Management | 6 | 0 | 6 |
| 5.04 Workspace Configuration | 3 | 0 | 3 |
| 5.05 Watch Mode | 5 | 0 | 5 |
| 5.06 Project Init / Scaffolding | 2 | 0 | 2 |

**Total: 109 BCs — 71 P0, 38 P1, 0 P2.**

---

## Section 3: Interface Definition

> Full detail in `.factory/specs/prd-supplements/interface-definitions.md`.

### 3.1 CLI Command Surface

```
slideforge <COMMAND> [OPTIONS]

Commands:
  build           Compile .sf source to output format(s)
  watch           Watch .sf source and data sources; live-reload web preview
  init            Scaffold a new slideforge project with starter files
  package         Manage content packages (install, list, remove, verify)
  config          Inspect and explain workspace configuration
  extract-brand   Extract brand.toml from an existing .pptx or .docx template

Options (build):
  <SOURCE>            Path to .sf entry file (required)
  --output <DIR>      Output directory [default: ./dist]
  --format <FMT>      Output format(s): pptx,docx,pdf,html,preview [default: pptx]
  --variant <NAME>    Build a named variant defined in the source
  --warn-only         Promote validation errors to warnings; render error-slide placeholders
  --offline           Skip HTTP data sources; fail on @import if sf.lock absent
  --template <FILE>   Override brand template (.pptx or brand.toml)
  --workspace         Build all workspace members declared in slideforge.toml
  -v, --verbose       Show per-stage timing and diagnostic details
```

### 3.2 Exit Code Semantics

| Code | Condition |
|------|-----------|
| 0 | Success — output written to --output |
| 1 | Parse error — no output produced |
| 2 | Validation error in strict mode — no output produced |
| 3 | Export error — serialization or I/O failure |
| 4 | Configuration error — brand not found, workspace misconfigured |
| 5 | Package error — package not found, lockfile mismatch |
| 64 | User error — invalid CLI arguments (clap convention) |
| 130 | Interrupted — SIGINT during watch mode |

### 3.3 JSON Output Schema (--format json diagnostic output)

```json
{
  "status": "error" | "warning" | "success",
  "diagnostics": [
    {
      "severity": "parse_error" | "validation_error" | "lint",
      "code": "E-PAR-001",
      "message": "string",
      "file": "string (absolute path)",
      "line": 42,
      "col": 7,
      "hint": "string (correction suggestion)"
    }
  ],
  "outputs": [
    {
      "format": "pptx" | "docx" | "pdf" | "html",
      "path": "string (absolute path)",
      "size_bytes": 102400
    }
  ],
  "build_ms": 342
}
```

---

## Section 4: Non-Functional Requirements

> **Authoritative NFR registry:** `.factory/specs/prd-supplements/nfr-catalog.md` (35 NFRs
> across 8 categories). The catalog supersedes any inline table here per CLAUDE.md precedence
> (PRD supplements supersede PRD prose for the same surface area).
>
> Key targets for quick reference (IDs are from nfr-catalog.md):
>
> | NFR-ID | Category | Requirement | Target |
> |--------|---------|-------------|--------|
> | NFR-001 | Performance | Cold build time for 25-slide deck | < 500ms |
> | NFR-002 | Performance | Incremental rebuild (watch mode, 1 slide change) | < 50ms |
> | NFR-007 | Visual Parity | PPTX SSIM vs reference PNG (LibreOffice) | ≥ 0.99 per slide |
> | NFR-008 | Visual Parity | PPTX PSNR vs reference PNG (LibreOffice) | ≥ 35dB per slide |
> | NFR-012 | Accessibility | PDF/UA-1 compliance (veraPDF) | Zero violations |
>
> For the full 35-NFR catalog with validation methods, risk sources, and per-category
> groupings, see `.factory/specs/prd-supplements/nfr-catalog.md`.

---

## Section 5: Error Taxonomy

> **Authoritative error catalog:** `.factory/specs/prd-supplements/error-taxonomy.md` (53 active
> error codes (3 retired) across 9 categories: E-PAR, E-EVL, E-DAT, E-LAY, E-EXP, E-BRD, E-PKG, E-CFG, E-A11).
> The supplement supersedes any inline table here per CLAUDE.md precedence (PRD supplements
> supersede PRD prose for the same surface area). Key error codes referenced in behavioral
> contracts trace to this catalog.
>
> Error code convention: `E-<CAT>-<NNN>` where CAT is the 3-char subsystem abbreviation and
> NNN is sequential within the category (001-999).

---

## Section 6: Competitive Differentiator Traceability

> Seeded from `domain-spec/differentiators.md` Section 9 (L2 Differentiators D-001 to D-007).

| Differentiator | Description | Backing BCs | Moat Strength |
|---------------|-------------|------------|--------------|
| D-001: Live Data Reactivity | @data at compile time + watch mode | BC-1.03.001, BC-1.03.002, BC-5.05.001, BC-5.05.002 | HIGH |
| D-002: Opinionated Slide Vocabulary | 31 named types with enforced semantics | BC-3.01.001, BC-3.01.002, BC-1.09.001 | HIGH |
| D-003: Multi-Format from One Source | PPTX+DOCX+PDF+HTML+preview | BC-4.01.001, BC-4.02.001, BC-4.03.001, BC-4.03.003, BC-4.03.004, BC-1.14.001–004 | MEDIUM |
| D-004: OOXML Brand Bridge | Load/synthesize/extract brand | BC-2.01.001, BC-2.01.002, BC-2.01.003 | HIGH |
| D-005: Compile-Time Accessibility | alt missing = compile error | BC-5.01.001, BC-5.01.002, BC-5.01.003, BC-5.01.005 | MEDIUM |
| D-006: Single Binary | No Node.js, no Python, no Chrome required | BC-1.12.001 (mmdr, no Node), BC-4.03.002 (pdf-writer, no Chrome) | LOW |
| D-007: Plugin-First Dog-Fooding | Trait API proven by bundled plugins | BC-5.02.001, BC-5.02.002 | LOW |

---

## Section 7: Requirements Traceability Matrix

| BC ID | Source L2 CAP | Priority | Category | Test Type |
|-------|-------------|----------|---------|-----------|
| BC-1.01.001 | CAP-001 | P0 | functional | unit + snapshot |
| BC-1.01.002 | CAP-001 | P0 | error-handling | unit |
| BC-1.01.003 | CAP-001 | P0 | error-handling | unit |
| BC-1.01.004 | CAP-001, CAP-006 | P0 | functional | unit |
| BC-1.01.005 | CAP-001 | P0 | error-handling | unit |
| BC-1.01.006 | CAP-001 | P0 | error-handling | unit |
| BC-1.02.001 | CAP-002 | P0 | functional | unit + proptest |
| BC-1.02.002 | CAP-002 | P0 | error-handling | unit |
| BC-1.02.003 | CAP-002 | P0 | invariant | unit + proptest |
| BC-1.02.004 | CAP-002, CAP-012 | P0 | edge-case | unit |
| BC-1.02.005 | CAP-002, CAP-004 | P0 | functional | unit |
| BC-1.03.001 | CAP-003 | P0 | functional | integration |
| BC-1.03.002 | CAP-003 | P0 | functional | integration |
| BC-1.03.003 | CAP-003 | P0 | error-handling | unit |
| BC-1.03.004 | CAP-003 | P1 | functional | integration |
| BC-1.03.005 | CAP-003 | P1 | security | integration |
| BC-1.03.006 | CAP-003 | P1 | functional | integration |
| BC-1.03.007 | CAP-003 | P1 | functional | integration |
| BC-1.04.001 | CAP-004 | P0 | functional | unit + integration |
| BC-1.04.002 | CAP-004 | P0 | edge-case | unit |
| BC-1.04.003 | CAP-004 | P0 | invariant | kani |
| BC-1.05.001 | CAP-005 | P0 | functional | unit |
| BC-1.05.002 | CAP-005 | P0 | functional | unit |
| BC-1.06.001 | CAP-006 | P0 | functional | integration |
| BC-1.06.002 | CAP-006 | P0 | invariant | unit |
| BC-1.06.003 | CAP-006 | P0 | edge-case | unit |
| BC-1.06.004 | CAP-006, CAP-025 | P1 | functional | integration |
| BC-1.07.001 | CAP-007 | P0 | functional | integration |
| BC-1.07.002 | CAP-007 | P0 | functional | unit |
| BC-1.07.003 | CAP-007 | P0 | invariant | unit |
| BC-1.07.004 | CAP-007 | P0 | error-handling | unit |
| BC-1.07.005 | CAP-007 | P0 | edge-case | unit |
| BC-1.08.001 | CAP-008 | P0 | functional | unit |
| BC-1.08.002 | CAP-008 | P0 | functional | unit |
| BC-1.08.003 | CAP-008 | P0 | edge-case | unit |
| BC-1.09.001 | CAP-009 | P1 | functional | unit |
| BC-1.09.002 | CAP-009 | P1 | invariant | unit |
| BC-1.10.001 | CAP-012 | P1 | functional | unit + snapshot |
| BC-1.10.002 | CAP-012 | P1 | error-handling | unit |
| BC-1.10.003 | CAP-012 | P1 | functional | integration |
| BC-1.11.001 | CAP-013 | P1 | functional | snapshot |
| BC-1.11.002 | CAP-013 | P1 | edge-case | unit |
| BC-1.12.001 | CAP-014 | P1 | functional | integration |
| BC-1.12.002 | CAP-014 | P1 | error-handling | unit |
| BC-1.12.003 | CAP-014 | P1 | functional | unit |
| BC-1.13.001 | CAP-028 | P1 | functional | unit |
| BC-1.14.001 | CAP-029 | P0 | functional | integration |
| BC-1.14.002 | CAP-029 | P0 | functional | integration |
| BC-1.14.003 | CAP-029 | P0 | functional | integration |
| BC-1.14.004 | CAP-029 | P0 | invariant | integration |
| BC-1.15.001 | CAP-030 | P0 | functional | unit |
| BC-1.15.002 | CAP-030 | P0 | functional | unit |
| BC-1.15.003 | CAP-030 | P0 | functional | unit |
| BC-2.01.001 | CAP-018 | P0 | functional | integration |
| BC-2.01.002 | CAP-018 | P0 | functional | snapshot |
| BC-2.01.003 | CAP-018 | P0 | functional | integration |
| BC-2.01.004 | CAP-018 | P0 | edge-case | unit |
| BC-2.01.005 | CAP-018 | P0 | functional | snapshot |
| BC-2.01.006 | CAP-018 | P0 | edge-case | unit |
| BC-2.02.001 | CAP-019 | P1 | functional | integration |
| BC-2.02.002 | CAP-019 | P1 | invariant | unit |
| BC-3.01.001 | CAP-010 | P0 | functional | unit + snapshot |
| BC-3.01.002 | CAP-010 | P0 | error-handling | unit |
| BC-3.01.003 | CAP-010 | P0 | error-handling | unit |
| BC-3.02.001 | CAP-011 | P0 | functional | integration |
| BC-3.02.002 | CAP-011 | P0 | functional | integration |
| BC-3.03.001 | CAP-022 | P0 | functional | unit |
| BC-3.03.002 | CAP-022 | P0 | invariant | integration |
| BC-3.03.003 | CAP-022 | P0 | functional | integration |
| BC-3.03.004 | CAP-022 | P0 | error-handling | unit |
| BC-3.04.001 | CAP-023 | P1 | functional | unit + snapshot |
| BC-3.04.002 | CAP-023 | P1 | error-handling | unit |
| BC-3.05.001 | CAP-024 | P1 | functional | snapshot |
| BC-4.01.001 | CAP-015 | P0 | functional | snapshot + visual |
| BC-4.01.002 | CAP-015 | P0 | visual-parity | visual regression |
| BC-4.01.003 | CAP-015 | P0 | functional | snapshot |
| BC-4.01.004 | CAP-015 | P0 | accessibility | manual + axe-core |
| BC-4.01.005 | CAP-015 | P0 | functional | snapshot |
| BC-4.01.006 | CAP-015 | P0 | functional | snapshot |
| BC-4.02.001 | CAP-016 | P0 | functional | snapshot |
| BC-4.02.002 | CAP-016 | P0 | functional | integration |
| BC-4.03.001 | CAP-017 | P0 | accessibility | veraPDF CI gate |
| BC-4.03.002 | CAP-017 | P0 | architecture | unit |
| BC-4.03.003 | CAP-017 | P0 | accessibility | axe-core CI |
| BC-4.03.004 | CAP-017 | P1 | functional | manual |
| BC-4.03.005 | CAP-017 | P0 | functional | kani + unit |
| BC-5.01.001 | CAP-020 | P0 | accessibility | unit |
| BC-5.01.002 | CAP-020 | P0 | accessibility | unit |
| BC-5.01.003 | CAP-020 | P0 | accessibility | unit |
| BC-5.01.004 | CAP-020 | P0 | accessibility | unit |
| BC-5.01.005 | CAP-020 | P0 | accessibility | integration |
| BC-5.02.001 | CAP-021 | P0 | architecture | unit |
| BC-5.02.002 | CAP-021 | P0 | invariant | integration |
| BC-5.03.001 | CAP-025 | P1 | functional | integration |
| BC-5.03.002 | CAP-025 | P1 | error-handling | unit |
| BC-5.03.003 | CAP-025 | P1 | functional | unit |
| BC-5.03.004 | CAP-025 | P1 | functional | unit |
| BC-5.03.005 | CAP-025 | P1 | functional | integration |
| BC-5.03.006 | CAP-025 | P1 | functional | integration |
| BC-5.04.001 | CAP-026 | P1 | functional | integration |
| BC-5.04.002 | CAP-026 | P1 | functional | unit |
| BC-5.04.003 | CAP-026 | P1 | functional | integration |
| BC-5.05.001 | CAP-027 | P1 | functional | integration |
| BC-5.05.002 | CAP-027 | P1 | edge-case | integration |
| BC-5.05.003 | CAP-027 | P1 | edge-case | integration |
| BC-5.05.004 | CAP-027 | P1 | failure-mode | integration |
| BC-5.05.005 | CAP-027 | P1 | failure-mode | manual |
| BC-5.06.001 | CAP-026 | P1 | functional | integration |
| BC-5.06.002 | CAP-026 | P1 | error-handling | unit |

---

## Section 8: Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Phase 4 holdout mean satisfaction | ≥ 0.85 | holdout-evaluator agent Phase 4 |
| Phase 4 holdout must-pass rate | ≥ 0.60 | holdout-evaluator agent Phase 4 |
| Cold build time (25-slide deck) | < 500ms | CI benchmark (criterion) |
| Incremental rebuild (watch mode) | < 50ms | CI benchmark |
| PPTX LibreOffice SSIM vs reference | ≥ 0.99 per slide | CI visual regression job |
| PPTX LibreOffice PSNR vs reference | ≥ 35dB per slide | CI visual regression job |
| PDF/UA-1 compliance | 0 veraPDF violations | CI veraPDF gate |
| HTML WCAG AA | 0 axe-core critical violations | CI axe-core gate |
| Code quality: .unwrap() in prod | 0 occurrences | CI clippy gate |
| Supply chain: deps pinned | 100% of production deps | CI grep gate |

---

## Appendix A: Supplement Files

| File | Purpose | Primary Consumers |
|------|---------|------------------|
| `.factory/specs/prd-supplements/interface-definitions.md` | CLI interface, exit codes, JSON schema, config schema, flag interactions | implementer, test-writer |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Complete E-xxx-NNN error catalog | implementer, test-writer |
| `.factory/specs/prd-supplements/test-vectors.md` | Canonical test vector tables | test-writer, holdout-evaluator |
| `.factory/specs/prd-supplements/nfr-catalog.md` | NFR-NNN with numerical targets and validation | architect, performance-engineer |

---

## Appendix B: Spike Resolution Summary

| Spike | Resolution | Impact on BCs |
|-------|-----------|--------------|
| S2 — PDF Backend | pdf-writer + krilla + custom SlideTagEngine. NO Chrome headless. | BC-4.03.001, BC-4.03.002, BC-4.03.005 |
| S5 — Brand Synthesis Layout | 11 standard + 20 custom = 31 layouts per template | BC-2.01.002, BC-2.01.005, BC-4.01.005 |
| S6 — Multi-Renderer Parity | SSIM ≥ 0.99 AND PSNR ≥ 35dB CI gate (raised from 0.97) | BC-4.01.002 |
| S14 — Mermaid Engine | mermaid-rs-renderer (pure Rust, no Node.js). usvg normalization. | BC-1.12.001, BC-1.12.003, D-006 |
