---
document_type: domain-spec-section
level: L2
section: "capabilities"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/specs/product-brief.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q2-decision-final.md
  - .factory/planning/q3-decision-final.md
  - .factory/planning/q4-q15-decisions.md
  - .factory/planning/q16-q25-decisions.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Domain Capabilities

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.

## CAP-001: DSL Source Parsing

Parse indentation-significant .sf source files into a typed AST with full error recovery.
Accumulate all parse errors; never fail on first. Every error carries file:line:col span and
correction hint. Support multi-file projects via `@include` and `@import` directives.

**Priority:** P0 | **Grounding:** product-brief.md §5, q1-decision-final.md §1,
q16-q25-decisions.md Q23 — DSL parsing is the entry point for all system value.

---

## CAP-002: Variable Interpolation and Expression Evaluation

Evaluate `{{ expr }}` interpolations in string fields using variables declared in `vars:` blocks
and data loaded via `@data`. Arithmetic, comparison, logical operators, field access, pipe
filters (~15 built-ins), and method calls on collections.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rungs 2, 7).

---

## CAP-003: Data Binding from External Sources

Load structured data at compile time from JSON files, CSV files, YAML files, TOML files,
HTTP/HTTPS URLs, Excel spreadsheets, and SQLite databases via `@data name from "source"`
directive. All data resolved before evaluation begins.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 6), q3-decision-final.md §3
(DataSource plugin surface).

---

## CAP-004: Iteration over Data Collections

Generate slides, content blocks, or document sections dynamically via `@for item in collection:`
over finite data collections. Iteration is provably terminating (no unbounded loops, no
user-defined functions in v1.0).

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 8). The live data-reactive
product vision in q1 §0 requires iteration for nightly metrics briefs and client-specific
batch rendering.

---

## CAP-005: Conditional Rendering

Include or exclude slides, elements, fields, and document sections via `@if / @elif / @else:`
blocks at all four scopes (slide-level, element-level, field-level, section-level).

**Priority:** P0 | **Grounding:** q4-q15-decisions.md Q14.

---

## CAP-006: Multi-File Composition via Includes

Compose decks from multiple .sf files using `@include "path.sf"` for local files
(complete slides, set rules, vars, aliases, or fragments inside slide fields) and
`@import "package/item"` for installed package content. Support cycle detection.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 4), q4-q15-decisions.md Q12,
q16-q25-decisions.md Q19.

---

## CAP-007: Variant-Based Deck Segmentation

Build audience-targeted deck variants from a single .sf source using `variants:` blocks with
`include_tags`, `exclude_tags`, and `vars:` overrides. Support multiple inheritance via
`inherits:` list with documented merge semantics (last-wins scalars, replace lists, deep-merge
maps).

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 5), q4-q15-decisions.md Q11,
q16-q25-decisions.md Q25.

---

## CAP-008: Set Rules for Slide-Type Defaults

Declare deck-scoped or workspace-scoped default field values for any slide type or section type
via `set <type>: <field> <value>`. Supports `{{ }}` interpolation and `brand.*` references.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 3), q4-q15-decisions.md Q10.

---

## CAP-009: Parametric Slide-Type Aliases

Define named presets on existing slide types via `alias <name> = <type>:` with optional set
defaults. Aliases resolve at parse time; they cannot add new fields, only preset existing ones.

**Priority:** P1 | **Grounding:** q2-decision-final.md Layer B.

---

## CAP-010: 31 Built-in Slide Types

Render any of 31 opinionated slide types (23 from the seed reference + 8 new in Q2:
chart, toc, agenda, quote, grid, bio, diagram, team). Each type enforces its own
required fields and layout rules via the `SlideType` plugin trait.

**Priority:** P0 | **Grounding:** q1-decision-final.md §4, q2-decision-final.md Layer A.
The shared vocabulary of slide types is slideforge's primary differentiator per R3 research.

---

## CAP-011: Document Section Generation

Produce ~15 document section types in DOCX/PDF/HTML output, both auto-generated from slide
data (executive_summary from `takeaway` fields, risk_register from severity_cards slides, etc.)
and manually authored (`section methodology:`, `section scope:`, `section approval:`).

**Priority:** P0 | **Grounding:** q1-decision-final.md §5.

---

## CAP-012: Math and LaTeX Rendering

Render inline (`$...$`) and display (`$$...$$`) math via mode-based parser switching.
Support `@{var}` interpolation inside math blocks. Produce OMML for PPTX/DOCX,
MathML/KaTeX for HTML, vector paths for PDF.

**Priority:** P1 | **Grounding:** q1-decision-final.md §9.

---

## CAP-013: Chart Rendering from Data

Render full-slide data charts (`slide chart:`) and inline chart figures
(`{{ chart.bar(...) }}`) as SVG via the plotters-backed ChartRenderer plugin.
Support bar, line, pie, scatter, area, histogram, and stacked bar in v1.0.

**Priority:** P1 | **Grounding:** q1-decision-final.md §8, q2-decision-final.md (chart type).

---

## CAP-014: Diagram Rendering (Mermaid)

Render full-slide Mermaid diagrams (`slide diagram:`) as SVG via the DiagramRenderer
plugin. Ships v1.0; implementation approach resolved by Spike S14 (single-binary constraint).

**Priority:** P1 | **Grounding:** q2-decision-final.md (diagram type), q3-decision-final.md
§2 surface #4.

---

## CAP-015: PPTX Export

Serialize a laid-out deck to a valid .pptx file using ooxmlsdk. Load a brand template
or synthesize one from brand.toml. Embed speaker notes, master/layout/theme system,
placeholder inheritance, slide sections, and accessibility metadata.

**Priority:** P0 | **Grounding:** product-brief.md §2, q1-decision-final.md §2 (PPTX row).
PPTX export is the founding purpose; the seed reference implementation proves the domain.

---

## CAP-016: DOCX Export

Serialize a deck to a .docx report document, consuming the `report` and `detail` writing
registers as narrative body content, auto-generating document sections from slide data.

**Priority:** P0 | **Grounding:** q1-decision-final.md §2 (DOCX row), §3 (writing registers),
§5 (document sections).

---

## CAP-017: PDF, HTML, and Web Preview Export

Produce PDF (tagged, PDF/UA-1 compliant via pdf-writer + krilla + SlideTagEngine), static HTML
(WCAG AA via axe-core), and a live web preview (axum + websocket + SVG canvas) from
the same laid-out deck IR.

**Priority:** P0 | **Grounding:** q1-decision-final.md §2 (PDF, HTML, web preview rows).

---

## CAP-018: Brand Template Loading and Synthesis

Load brand configuration from an existing .pptx or .docx template file OR synthesize
a complete brand template from a brand.toml file (colors, fonts, logo, footer).
Support bidirectional extraction: `slideforge extract-brand deck.pptx → brand.toml`.

**Priority:** P0 | **Grounding:** q1-decision-final.md §10 (brand bridge), product-brief.md §3.

---

## CAP-019: Per-Slide Brand Overlay

Override logo, footer, and confidentiality banner on individual slides or sections
via `brand_overlay:` blocks, without switching masters (single-master architecture in v1.0).

**Priority:** P1 | **Grounding:** q4-q15-decisions.md Q4.

---

## CAP-020: Accessibility Validation

Enforce at compile time: `alt "..."` on all visual elements (images, charts, diagrams),
`label "..."` on all color-coded elements (severity_cards, status, progress_bar),
`lang "en-US"` at deck level. Validators emit structured diagnostics with source spans.

**Priority:** P0 | **Grounding:** q4-q15-decisions.md Q6. The compile-time accessibility
contract is a stated product differentiator (product-brief.md §2).

---

## CAP-021: Plugin Architecture with 10 Extensibility Surfaces

Expose 10 `dyn Trait` plugin surfaces (DataSource, Exporter, ChartRenderer,
DiagramRenderer, Validator, MathRenderer, BrandProvider, SlideType, SectionType,
InlineFormat). All bundled plugins dog-food the same traits; no bypass paths allowed.

**Priority:** P0 | **Grounding:** q3-decision-final.md, product-brief.md §3.

---

## CAP-022: Compile-Time Content Validation

Validate canvas overflow, WCAG contrast ratios, color name existence, alt text presence,
bullet length, and weight normalization. Strict mode (default build): validation errors
produce no output. Warn-only (watch mode): output with error-slide placeholders.

**Priority:** P0 | **Grounding:** q3-decision-final.md §2 (Validator surface), q16-q25-decisions.md Q17.

---

## CAP-023: Structured Shape DSL

Declare custom visual shapes via `shape:` blocks with type, position, size, fill, text, and
required alt text. No raw XML exposed to users; shape DSL is accessible and multi-format
renderable. `raw` keyword is reserved and rejects at parse time.

**Priority:** P1 | **Grounding:** q4-q15-decisions.md Q7.

---

## CAP-024: Rich Inline Formatting

Format text with bold, italic, code, hyperlinks, math, footnotes, cross-references,
superscript, subscript, strikethrough, and highlight. All 11 inline types ship v1.0.

**Priority:** P1 | **Grounding:** q4-q15-decisions.md Q8.

---

## CAP-025: Package Management

Install versioned content packages from git repositories via `slideforge package install`.
Manage dependencies in `slideforge.toml`, lock in `sf.lock`. Import package content via
`@import "package/item"` (distinct from local `@include`).

**Priority:** P1 | **Grounding:** q16-q25-decisions.md Q19.

---

## CAP-026: Cargo-Style Workspace Configuration

Declare multi-deck workspaces in `slideforge.toml`, apply family-specific overrides via
`.sfconfig` cascade (max 3 levels), build all members with `slideforge build --workspace`.
Inspect configuration provenance with `slideforge config explain`. Scaffold new projects
and workspaces via `slideforge init`.

**Priority:** P1 | **Grounding:** q16-q25-decisions.md Q20, Q21.

---

## CAP-027: Watch Mode with Live Data Refresh

Run `slideforge watch` to poll data sources and .sf files for changes, re-evaluate on
change, and push incremental updates to the web preview via websocket. Render error-slide
placeholders in warn-only mode during watch.

**Priority:** P1 | **Grounding:** q1-decision-final.md §0 and §1 (data binding, watch),
q16-q25-decisions.md Q17.

---

## CAP-028: DSL Versioning and Forward Compatibility

Require `slideforge_version "1"` in deck metadata. Gate grammar on major version.
Reject forward-incompatible files with a clear error. Enable future `slideforge migrate`
tooling for version upgrades.

**Priority:** P1 | **Grounding:** q16-q25-decisions.md Q16.

---

## CAP-029: Writing Register Support

Distinguish three writing registers: `notes` (presenter-facing), `report` (reader-facing
formal prose), `detail` (document-only extended analysis). Each field routes to the correct
output format; no register content bleeds across formats.

**Priority:** P0 | **Grounding:** q1-decision-final.md §3. Writing registers are
foundational to the dual PPTX+DOCX output value proposition.

---

## CAP-030: Diagnostic Reporting with Source Spans

Emit all errors and warnings with file:line:col spans and correction hints ("did you
mean...?") rendered via miette/ariadne. Accumulate all diagnostics in one pass — never
fail on first error. Three-tier severity: parse errors (always fatal), validation errors
(fatal in strict, warnings in warn-only), lint (always warnings).

**Priority:** P0 | **Grounding:** q16-q25-decisions.md Q17, Q23.
