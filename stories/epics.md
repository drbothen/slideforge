---
document_type: epic-decomposition
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-25T00:00:00
phase: 2
traces_to:
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/domain-spec/L2-INDEX.md
---

# Epic Decomposition — slideforge v1.0

> Epics are organized along crate and subsystem boundaries, following the
> 20-crate Cargo workspace structure. Each epic maps to one primary subsystem
> (SS-ID from ARCH-INDEX.md). Cross-cutting epics (CI/CD, formal verification,
> documentation) span multiple subsystems.
>
> Phase ordering: Epics within the same wave run in parallel. Epics in later
> waves depend on earlier epics completing (see dependency-graph.md).

---

## Epic Registry

| Epic ID | Title | SS-ID | Primary Crate | BC Range | Wave | Story Count | Priority |
|---------|-------|-------|--------------|----------|------|-------------|---------|
| EPIC-01 | Foundation Types and Plugin API | SS-14, SS-15 | slideforge-types, slideforge-plugin-api | BC-3.01, BC-5.02 | 1 | 4 | P0 |
| EPIC-02 | DSL Parser | SS-01 | slideforge-syntax | BC-1.01, BC-1.06, BC-1.07, BC-1.08, BC-1.09, BC-1.13, BC-1.15 | 1 | 6 | P0 |
| EPIC-03 | Expression Evaluator | SS-02 | slideforge-eval | BC-1.02, BC-1.04, BC-1.05 | 2 | 4 | P0 |
| EPIC-04 | Compile-Time Validation | SS-03 | slideforge-validate | BC-3.03, BC-5.01 | 2 | 3 | P0 |
| EPIC-05 | Data Sources | SS-10 | slideforge-data | BC-1.03 | 3 | 4 | P0 |
| EPIC-06 | Brand System | SS-04 | slideforge-brand | BC-2.01, BC-2.02 | 3 | 4 | P0 |
| EPIC-07 | Layout Engine | SS-05 | slideforge-layout | BC-3.02, BC-3.04, BC-3.05, BC-3.06 | 3 | 3 | P0 |
| EPIC-08 | PPTX Export | SS-06 | slideforge-pptx | BC-4.01 | 4 | 4 | P0 |
| EPIC-09 | DOCX Export | SS-08 | slideforge-docx | BC-4.02 | 4 | 2 | P0 |
| EPIC-10 | Math Rendering | SS-13 | slideforge-math | BC-1.10 | 3 | 2 | P1 |
| EPIC-11 | Chart Rendering | SS-12 | slideforge-charts | BC-1.11 | 3 | 2 | P1 |
| EPIC-12 | Diagram Rendering | SS-11 | slideforge-diagrams | BC-1.12 | 3 | 2 | P1 |
| EPIC-13 | PDF Export | SS-07 | slideforge-pdf | BC-4.03 (PDF) | 4 | 3 | P0 |
| EPIC-14 | HTML Export and Web Preview | SS-09 | slideforge-html, slideforge-preview | BC-4.03 (HTML/preview) | 5 | 3 | P1 |
| EPIC-15 | CLI Orchestrator | SS-18 | slideforge-cli | BC-1.15, BC-5.05 | 5 | 5 | P0 |
| EPIC-16 | Package Management | SS-16 | slideforge-package | BC-5.03, BC-1.06 (import) | 5 | 4 | P1 |
| EPIC-17 | Workspace Configuration | SS-17 | slideforge-config | BC-5.04 | 5 | 2 | P1 |
| EPIC-18 | Writing Registers | SS-02, SS-06, SS-08 | slideforge-eval + exporters | BC-1.14 | 4 | 2 | P0 |
| EPIC-19 | CI/CD Infrastructure | — | cross-cutting | NFR-016 through NFR-031 | 1 | 4 | P0 |
| EPIC-20 | Formal Verification (Phase 6) | SS-01,02,03,05,06,07 | multiple | VP-001 through VP-015 | 6 | 6 | P0 |
| EPIC-21 | Plugin Registry Assembly | SS-14, SS-18 | slideforge (root) | BC-5.02 | 4 | 2 | P0 |
| **Total** | | | | **112 BCs** | | **71** | |

---

## Epic Details

### EPIC-01: Foundation Types and Plugin API

**SS-ID:** SS-14 (Plugin API), SS-15 (IR Types)
**Primary Crates:** `slideforge-types`, `slideforge-plugin-api`
**Wave:** 1 (must exist before any other crate compiles)
**Priority:** P0

**Description:** Define the core IR types (`Deck`, `LaidOutDeck`, `Slide`, `Value`,
`ContentBlock`, `Brand`) and all 10 plugin trait surfaces. These are pure-type crates
with zero production dependencies on other workspace crates. They are the foundation
all other crates build on. Includes all 31 `SlideType` trait implementations as logical
submodules within `slideforge-types`.

**BC Scope:**
- BC-3.01.001 through BC-3.01.003 — SlideType trait enforcement
- BC-5.02.001, BC-5.02.002 — Plugin trait surfaces and dog-fooding

**NFR Scope:**
- NFR-021 through NFR-025 (code quality enforcement on all crates starts here)
- NFR-024 `#![forbid(unsafe_code)]` applied from day 1

**Story Count:** 4 stories (types core, plugin traits, slide type implementations, value system)

---

### EPIC-02: DSL Parser

**SS-ID:** SS-01
**Primary Crate:** `slideforge-syntax`
**Wave:** 1 (pure core — no workspace deps; can build in parallel with EPIC-01)
**Priority:** P0

**Description:** Implement the chumsky 0.10 indentation-significant lexer and parser.
Produces a Typed AST from `.sf` source with full error accumulation (no fail-on-first).
Handles all DSL surface syntax: deck metadata, slide types, `@for`, `@if/@elif/@else`,
`{{ expr }}` interpolation, `@include`, `variants:`, `set` rules, `alias` definitions,
`$...$` math delimiters, and `shape:` blocks. DSL versioning check included.

**BC Scope:**
- BC-1.01.001 through BC-1.01.006 — core parsing
- BC-1.06.001, BC-1.06.003 — @include (cycle detection in eval/compilation stage)
- BC-1.07.001 through BC-1.07.005 — variants
- BC-1.08.001 through BC-1.08.003 — set rules
- BC-1.09.001, BC-1.09.002 — aliases
- BC-1.13.001 — DSL versioning
- BC-1.15.001, BC-1.15.002, BC-1.15.003 — diagnostics (parse layer)
- BC-3.04.002 — reject `raw` keyword

**VP Scope:** VP-001 (Kani — tab detection), VP-003 (Kani — @for termination), VP-009 (proptest), VP-014 (fuzz)

**Story Count:** 6 stories

---

### EPIC-03: Expression Evaluator

**SS-ID:** SS-02
**Primary Crate:** `slideforge-eval`
**Wave:** 2 (depends on EPIC-01 types, EPIC-02 AST)
**Priority:** P0

**Description:** Implement the expression evaluator: `{{ expr }}` interpolation with
arithmetic, pipe filters, and ~15 built-in functions. Variable scoping (lexical, nested
`@for`). No implicit type coercion. Conditional evaluation (`@if/@elif/@else`). `@for`
collection iteration with termination guarantee. `@data` binding (connects to
slideforge-data). `@include` cycle detection happens here after AST merge.

**BC Scope:**
- BC-1.02.001 through BC-1.02.005 — variable interpolation and expression evaluation
- BC-1.04.001 through BC-1.04.003 — @for iteration
- BC-1.05.001, BC-1.05.002 — conditional rendering
- BC-1.06.002 — circular @include detection
- BC-1.06.004 — @import (connects to slideforge-package)

**VP Scope:** VP-004 (Kani — no coercion), VP-005 (Kani — no overflow), VP-010 (proptest — scoping), VP-015 (fuzz)

**Story Count:** 4 stories

---

### EPIC-04: Compile-Time Validation

**SS-ID:** SS-03
**Primary Crate:** `slideforge-validate`
**Wave:** 2 (depends on EPIC-01 types, runs on completed Deck IR)
**Priority:** P0

**Description:** All compile-time content validation. Alt text enforcement (compile
error when missing), color-coded label enforcement, canvas overflow detection, strict
mode / warn-only mode gate, zero-slide deck error, WCAG contrast check. The validation
stage runs after evaluation and before layout — the only position with full semantic
information.

**BC Scope:**
- BC-3.03.001 through BC-3.03.004 — content validation
- BC-5.01.001 through BC-5.01.005 — accessibility validation

**VP Scope:** VP-002 (Kani — alt-missing before layout), VP-007 (Kani — WCAG contrast formula), VP-008 (Kani — alt diagnostic invariant)

**Story Count:** 3 stories

---

### EPIC-05: Data Sources

**SS-ID:** SS-10
**Primary Crate:** `slideforge-data`
**Wave:** 3 (depends on EPIC-01 types; provides DataSource plugin impl)
**Priority:** P0

**Description:** `DataSource` plugin implementations for all supported sources:
JSON/CSV/YAML/TOML files, HTTP/HTTPS URLs (with allowlist enforcement for SSRF
prevention), Excel (.xlsx) via calamine, SQLite via rusqlite. Structured error on
missing field access. `--offline` flag to skip HTTP sources.

**BC Scope:**
- BC-1.03.001 through BC-1.03.007 — all @data loading sources

**NFR Scope:**
- NFR-018 (SHA-256 integrity, via slideforge-package)
- NFR-019 (HTTP allowlist)

**Story Count:** 4 stories (file sources, HTTP source + SSRF, Excel + SQLite, offline mode)

---

### EPIC-06: Brand System

**SS-ID:** SS-04
**Primary Crate:** `slideforge-brand`
**Wave:** 3 (depends on EPIC-01 types)
**Priority:** P0

**Description:** `BrandProvider` plugin implementation. Load brand from existing
`.pptx`/`.docx` template (extract via quick-xml). Synthesize complete brand from
`brand.toml` (all 12 OOXML slots, 31 layouts: 11 standard + 20 custom). Brand
extraction (`slideforge extract-brand`). Missing color slot synthesis with defaults.
Font fallback. Per-slide `brand_overlay:` (logo/footer/confidentiality overrides,
no master switching).

**BC Scope:**
- BC-2.01.001 through BC-2.01.006 — brand loading and synthesis
- BC-2.02.001, BC-2.02.002 — per-slide brand overlay

**VP Scope:** VP-012 (proptest — 12-slot palette round-trip)

**Story Count:** 4 stories (brand loading, brand synthesis, brand extraction CLI, brand overlay)

---

### EPIC-07: Layout Engine

**SS-ID:** SS-05
**Primary Crate:** `slideforge-layout`
**Wave:** 3 (depends on EPIC-01 types; produces LaidOutDeck consumed by all exporters)
**Priority:** P0

**Description:** `Deck → LaidOutDeck` transformation. EMU coordinate system
(914400 EMU per inch, integer arithmetic). Text flow computation. Document section
generation for DOCX (auto-generated from slide data + manually authored section
blocks). Shape DSL layout for `shape:` blocks. All 11 inline format types layout.

**BC Scope:**
- BC-3.02.001, BC-3.02.002 — document section generation
- BC-3.04.001 — shape: block layout
- BC-3.05.001 — rich inline formatting layout
- BC-3.06.001, BC-3.06.002, BC-3.06.003 — layout overflow detection and bounds enforcement

**VP Scope:** VP-011 (proptest — slide count invariant)

**Story Count:** 3 stories (core layout + EMU, section generation, shape + inline formats)

---

### EPIC-08: PPTX Export

**SS-ID:** SS-06
**Primary Crate:** `slideforge-pptx`
**Wave:** 4 (depends on EPIC-01, EPIC-06 brand, EPIC-07 LaidOutDeck)
**Priority:** P0

**Description:** `Exporter` plugin implementation for PPTX via ooxmlsdk 0.6.1.
Full OOXML compliance: placeholder inheritance (layout→master by `type`, slide→layout
by `idx`), slide IDs starting at 256, master IDs at 2^31, all 31 layouts present,
`notesMaster1.xml` + `handoutMaster1.xml` always present, element ordering
schema-significant, `[Content_Types].xml` comprehensive registration, `clrMapOvr`
for dark layouts. Accessibility metadata (alt text, lang, WCAG contrast). Speaker
notes. Slide sections.

**BC Scope:**
- BC-4.01.001 through BC-4.01.006 — PPTX export

**VP Scope:** VP-013 (proptest — valid ZIP with Content_Types.xml)

**NFR Scope:** NFR-007, NFR-008 (visual parity SSIM/PSNR), NFR-009, NFR-010, NFR-011, NFR-014, NFR-015

**Story Count:** 4 stories (core serialization, layout compliance, a11y metadata, notes/sections)
> Note: Visual parity CI belongs to EPIC-19 (STORY-052), not EPIC-08.

---

### EPIC-09: DOCX Export

**SS-ID:** SS-08
**Primary Crate:** `slideforge-docx`
**Wave:** 4 (depends on EPIC-01, EPIC-06, EPIC-07, EPIC-18)
**Priority:** P0

**Description:** `Exporter` plugin implementation for DOCX via ooxmlsdk 0.6.1.
`report` register content as body paragraphs. Auto-generated document sections
from slide data. Writing register routing (report/detail into DOCX body; notes
into DOCX presenter notes area).

**BC Scope:**
- BC-4.02.001, BC-4.02.002 — DOCX export

**Story Count:** 2 stories (core DOCX, section generation)

---

### EPIC-10: Math Rendering

**SS-ID:** SS-13
**Primary Crate:** `slideforge-math`
**Wave:** 3 (pure core; depends on EPIC-01 types)
**Priority:** P1

**Description:** LaTeX transformation pipeline. Mode-based parsing: `$...$` and
`$$...$$` toggle math mode; `@{var}` interpolation inside math mode (not `{{ }}`).
Output: OMML for PPTX/DOCX, MathML for HTML, paths for PDF. Error on unsupported
LaTeX command with source span and hint.

**BC Scope:**
- BC-1.10.001, BC-1.10.002, BC-1.10.003 — math rendering

**Story Count:** 2 stories (math parser + OMML output, HTML/PDF math output)

---

### EPIC-11: Chart Rendering

**SS-ID:** SS-12
**Primary Crate:** `slideforge-charts`
**Wave:** 3 (pure core; depends on EPIC-01 types)
**Priority:** P1

**Description:** `ChartRenderer` plugin implementation via plotters 0.3.7.
Produces SVG from chart spec. Supported types: bar, line, pie, scatter, area,
histogram, stacked-bar. Empty data produces error-slide placeholder, not crash.
Brand-aware color application.

**BC Scope:**
- BC-1.11.001, BC-1.11.002 — chart rendering

**Story Count:** 2 stories (core chart types, empty data error handling)

---

### EPIC-12: Diagram Rendering

**SS-ID:** SS-11
**Primary Crate:** `slideforge-diagrams`
**Wave:** 3 (effectful due to font DB; depends on EPIC-01 types)
**Priority:** P1

**Description:** `DiagramRenderer` plugin implementation via mermaid-rs-renderer 0.2.2.
Renders Mermaid source to PPTX-safe SVG. SVG normalized via usvg 0.47.0 before PPTX
embedding (no `foreignObject`, absolute dims). Error on invalid Mermaid syntax with
line number. Cold font DB scan < 200ms (NFR-003). Warm render < 10ms (NFR-004).

**BC Scope:**
- BC-1.12.001, BC-1.12.002, BC-1.12.003 — diagram rendering

**NFR Scope:** NFR-003, NFR-004

**Story Count:** 2 stories (mermaid render pipeline, SVG normalization + performance)

---

### EPIC-13: PDF Export

**SS-ID:** SS-07
**Primary Crate:** `slideforge-pdf`
**Wave:** 4 (depends on EPIC-07 LaidOutDeck; Phase 4 crate added to workspace then)
**Priority:** P0

**Description:** `Exporter` plugin for PDF via pdf-writer 0.14.0 + krilla 0.6.0.
PDF/UA-1 compliant tagged PDF (veraPDF). EMU-to-PDF user unit coordinate mapping
with correct Y-axis flip. `SlideTagEngine` for accessibility tagging. No Chrome/
headless dependency. PDF math via paths.

**BC Scope:**
- BC-4.03.001, BC-4.03.002, BC-4.03.005 — PDF export

**VP Scope:** VP-006 (Kani — EMU-to-PDF coordinate mapping)

**NFR Scope:** NFR-012 (veraPDF PDF/UA-1)

**Story Count:** 3 stories (core PDF, coordinate mapping + tagging, PDF/UA-1 verification CI)

---

### EPIC-14: HTML Export and Web Preview

**SS-ID:** SS-09
**Primary Crates:** `slideforge-html`, `slideforge-preview`
**Wave:** 5 (depends on EPIC-07 LaidOutDeck; Phase 4 crate)
**Priority:** P1

**Description:** Static HTML `Exporter` (WCAG AA via axe-core). Web preview server
via axum 0.8.1 + WebSocket + SVG canvas (not `<canvas>` element per S3 spike).
Live reload on save. WebSocket reconnect on drop. `prefers-reduced-motion` support.
Accessibility: ARIA roles, keyboard navigation, `aria-live` connection status.

**BC Scope:**
- BC-4.03.003 — static HTML WCAG AA
- BC-4.03.004 — web preview axum + WebSocket
- BC-5.05.005 — WebSocket reconnect

**NFR Scope:** NFR-013 (axe-core WCAG AA)

**Story Count:** 3 stories (static HTML export, web preview server, WebSocket live reload + a11y)

---

### EPIC-15: CLI Orchestrator

**SS-ID:** SS-18
**Primary Crate:** `slideforge-cli`
**Wave:** 5 (depends on all core crates and all exporters)
**Priority:** P0

**Description:** CLI binary via clap 4.5. Pipeline orchestration (`build`, `watch`,
`serve`, `extract-brand`, `init`, `config`, `package` subcommands). Watch mode via
notify 6.1 (file watching + incremental rebuild). `--offline`, `--warn-only`,
`--json`, `--quiet`, `--verbose` flags. miette 7.2 error rendering. NO_COLOR +
non-TTY compliance. Structured tracing via tracing-subscriber. All 6 pipeline stage
spans. Criterion build benchmarks.

**BC Scope:**
- BC-1.15.001, BC-1.15.002, BC-1.15.003 — diagnostic rendering at CLI layer
- BC-5.05.001 through BC-5.05.004 — watch mode
- BC-5.06.001, BC-5.06.002 — init/scaffolding

**NFR Scope:** NFR-001, NFR-002 (performance benchmarks), NFR-005, NFR-032, NFR-033

**Story Count:** 5 stories (build command, watch mode, init + extract-brand, config explain, error display)

---

### EPIC-16: Package Management

**SS-ID:** SS-16
**Primary Crate:** `slideforge-package`
**Wave:** 5 (depends on EPIC-03 eval for @import resolution)
**Priority:** P1

**Description:** Git-based package install with `sf.lock` and SHA-256 integrity.
`@import` resolution at compile time (connects from slideforge-eval). `slideforge
package install/remove/list/verify`. `sf.lock` missing warning.

**BC Scope:**
- BC-5.03.001 through BC-5.03.006 — package management
- BC-1.06.004 — @import resolution

**NFR Scope:** NFR-018 (SHA-256 integrity)

**Story Count:** 4 stories (install + lock, import resolution, remove + list, verify + integrity)

---

### EPIC-17: Workspace Configuration

**SS-ID:** SS-17
**Primary Crate:** `slideforge-config`
**Wave:** 5 (depends on EPIC-15 CLI)
**Priority:** P1

**Description:** `slideforge.toml [workspace]` multi-deck member declaration.
`.sfconfig` cascade (max 3 levels, family-specific overrides). `slideforge config
explain` configuration provenance. `build --workspace`.

**BC Scope:**
- BC-5.04.001, BC-5.04.002, BC-5.04.003 — workspace configuration

**Story Count:** 2 stories (workspace build, .sfconfig cascade + config explain)

---

### EPIC-18: Writing Registers

**SS-ID:** SS-02 (eval routing) + SS-06, SS-08 (export rendering)
**Primary Crates:** `slideforge-eval`, `slideforge-pptx`, `slideforge-docx`
**Wave:** 4 (register routing is an eval concern; rendering is export concern)
**Priority:** P0

**Description:** Three writing registers: `notes` (presenter notes in PPTX/DOCX/HTML),
`report` (DOCX body paragraphs; excluded from PPTX slide content), `detail` (DOCX/PDF
only; excluded from PPTX and web preview). No content bleeds to wrong format. Register
routing in eval stage; register-aware rendering in each exporter.

**BC Scope:**
- BC-1.14.001 through BC-1.14.004 — writing registers

**Story Count:** 2 stories (register routing in eval, register rendering in exporters)

---

### EPIC-19: CI/CD Infrastructure

**SS-ID:** — (cross-cutting)
**Primary Crates:** devops (all crates affected)
**Wave:** 1 (CI must exist before any story merges)
**Priority:** P0

**Description:** GitHub Actions workflows for the full quality bar. 5-platform matrix
(macOS arm64/x86_64, Linux x86_64/arm64, Windows x86_64). Format + clippy + nextest.
Visual regression job (LibreOffice headless + SSIM/PSNR). Supply-chain audit (cargo
audit + cargo deny + SBOM). Release pipeline with signed artifacts. Performance
benchmark regression gate. Reproducible builds job.

**NFR Scope:** NFR-016 through NFR-031

**Story Count:** 4 stories (CI matrix + linting, visual regression CI, supply-chain CI, release pipeline)

---

### EPIC-20: Formal Verification (Phase 6)

**SS-ID:** SS-01, SS-02, SS-03, SS-05, SS-06, SS-07 (multiple)
**Wave:** 6 (post-implementation; requires complete implementations)
**Priority:** P0

**Description:** Kani proofs for pure-core functions (8 VPs). Proptest suites for
property-based testing (5 VPs). Fuzz targets via cargo-fuzz (2 VPs). cargo-mutants
mutation testing with documented kill-rate budget. All Phase 6 formal verification work.

**VP Scope:** VP-001 through VP-015

**Story Count:** 6 stories (Kani proofs syntax+eval, Kani proofs validate+pdf, proptest suites, fuzz harnesses, mutation testing, verification CI integration)

---

### EPIC-21: Plugin Registry Assembly

**SS-ID:** SS-14 (plugin-api), SS-18 (CLI via root crate)
**Primary Crate:** `slideforge` (root), `slideforge-plugin-registry` (if separate)
**Wave:** 4 (depends on all plugin implementations existing)
**Priority:** P0

**Description:** The root `slideforge` crate assembles the `PluginRegistry` from all
bundled plugin crates. Verifies dog-fooding guarantee: all bundled plugins registered
via the public trait API (BC-5.02.001, BC-5.02.002). Integration tests drive the
full pipeline from .sf source to output bytes.

**BC Scope:**
- BC-5.02.001, BC-5.02.002 — plugin architecture guarantee

**Story Count:** 2 stories (registry assembly, end-to-end integration test suite)

---

## BC Coverage Summary by Epic

| Epic | BCs Covered | BC IDs |
|------|-------------|--------|
| EPIC-01 | 5 | BC-3.01.001-003, BC-5.02.001-002 |
| EPIC-02 | 23 | BC-1.01.001-006, BC-1.06.001+003, BC-1.07.001-005, BC-1.08.001-003, BC-1.09.001-002, BC-1.13.001, BC-1.15.001-003, BC-3.04.002 |
| EPIC-03 | 12 | BC-1.02.001-005, BC-1.04.001-003, BC-1.05.001-002, BC-1.06.002+004 |
| EPIC-04 | 9 | BC-3.03.001-004, BC-5.01.001-005 |
| EPIC-05 | 7 | BC-1.03.001-007 |
| EPIC-06 | 8 | BC-2.01.001-006, BC-2.02.001-002 |
| EPIC-07 | 7 | BC-3.02.001-002, BC-3.04.001, BC-3.05.001, BC-3.06.001-003 |
| EPIC-08 | 6 | BC-4.01.001-006 |
| EPIC-09 | 2 | BC-4.02.001-002 |
| EPIC-10 | 3 | BC-1.10.001-003 |
| EPIC-11 | 2 | BC-1.11.001-002 |
| EPIC-12 | 3 | BC-1.12.001-003 |
| EPIC-13 | 3 | BC-4.03.001-002+005 |
| EPIC-14 | 3 | BC-4.03.003-004, BC-5.05.005 |
| EPIC-15 | 9 | BC-1.15.001-003, BC-5.05.001-004, BC-5.06.001-002 |
| EPIC-16 | 7 | BC-5.03.001-006, BC-1.06.004 |
| EPIC-17 | 3 | BC-5.04.001-003 |
| EPIC-18 | 4 | BC-1.14.001-004 |
| EPIC-19 | NFRs | NFR-016 through NFR-031 |
| EPIC-20 | VPs | VP-001 through VP-015 |
| EPIC-21 | 2 | BC-5.02.001-002 (integration verification) |
| **Total** | **112** | All BCs covered |

> Note: BC-5.02.001 and BC-5.02.002 appear in both EPIC-01 (trait definition)
> and EPIC-21 (integration verification). This is intentional — EPIC-01 defines
> the contracts, EPIC-21 verifies them end-to-end.

---

## Wave Summary

| Wave | Epics | Can Run in Parallel | Blocker |
|------|-------|-------------------|---------|
| Wave 1 | EPIC-01, EPIC-02, EPIC-19 | Yes | None |
| Wave 2 | EPIC-03, EPIC-04 | Yes | Wave 1 complete |
| Wave 3 | EPIC-05, EPIC-06, EPIC-07, EPIC-10, EPIC-11, EPIC-12 | Yes | Wave 2 complete |
| Wave 4 | EPIC-08, EPIC-09, EPIC-13, EPIC-18, EPIC-21 | Yes | Wave 3 complete |
| Wave 5 | EPIC-14, EPIC-15, EPIC-16, EPIC-17 | Yes | Wave 4 complete |
| Wave 6 | EPIC-20 | Yes (sub-stories parallel) | Wave 5 complete |
