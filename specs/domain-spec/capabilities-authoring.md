---
document_type: domain-spec-section
level: L2
section: "capabilities-authoring"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/specs/product-brief.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q2-decision-final.md
  - .factory/planning/q4-q15-decisions.md
  - .factory/planning/q16-q25-decisions.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Domain Capabilities — Authoring Subsystem (CAP-001 to CAP-015)

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Companion: capabilities-output.md (CAP-016 to CAP-030).

---

## CAP-001: DSL Source Parsing

Parse indentation-significant .sf source files into a typed AST with full error recovery.
Accumulate all parse errors; never fail on first. Every error carries file:line:col span
and correction hint. Support multi-file projects via `@include` and `@import`.

**Priority:** P0 | **Grounding:** product-brief.md §5, q1-decision-final.md §1, q16-q25-decisions.md Q23.

---

## CAP-002: Variable Interpolation and Expression Evaluation

Evaluate `{{ expr }}` interpolations in string fields using variables from `vars:` blocks
and data loaded via `@data`. Support arithmetic, comparison, logical operators, field
access, pipe filters (~15 built-ins), and method calls on collections.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rungs 2, 7).

---

## CAP-003: Data Binding from External Sources

Load structured data at compile time from JSON, CSV, YAML, TOML, HTTP/HTTPS, Excel,
and SQLite via `@data name from "source"`. All data resolved before evaluation begins.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 6), q3-decision-final.md §3.

---

## CAP-004: Iteration over Data Collections

Generate slides, content blocks, or document sections dynamically via `@for item in collection:`
over finite data collections. Provably terminating — no unbounded loops.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 8).

---

## CAP-005: Conditional Rendering

Include or exclude slides, elements, fields, and document sections via `@if / @elif / @else:`
at all four scopes: slide-level, element-level, field-level, section-level.

**Priority:** P0 | **Grounding:** q4-q15-decisions.md Q14.

---

## CAP-006: Multi-File Composition via Includes

Compose decks from multiple .sf files using `@include "path.sf"` for local files and
`@import "package/item"` for installed package content. Cycle detection required.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 4), q4-q15-decisions.md Q12,
q16-q25-decisions.md Q19.

---

## CAP-007: Variant-Based Deck Segmentation

Build audience-targeted deck variants from a single .sf source using `variants:` blocks
with `include_tags`, `exclude_tags`, and `vars:` overrides. Support multiple inheritance
via `inherits:` list.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 5), q4-q15-decisions.md Q11,
q16-q25-decisions.md Q25.

---

## CAP-008: Set Rules for Slide-Type Defaults

Declare deck-scoped or workspace-scoped default field values for any slide type via
`set <type>: <field> <value>`. Supports `{{ }}` interpolation and `brand.*` references.

**Priority:** P0 | **Grounding:** q1-decision-final.md §1 (rung 3), q4-q15-decisions.md Q10.

---

## CAP-009: Parametric Slide-Type Aliases

Define named presets on existing slide types via `alias <name> = <type>:`. Aliases resolve
at parse time; they cannot add new fields, only preset existing ones.

**Priority:** P1 | **Grounding:** q2-decision-final.md Layer B.

---

## CAP-010: 31 Built-in Slide Types

Render any of 31 opinionated slide types (23 from the seed reference + 8 new: chart, toc,
agenda, quote, grid, bio, diagram, team). Each type enforces required fields and layout rules
via the `SlideType` plugin trait.

**Priority:** P0 | **Grounding:** q1-decision-final.md §4, q2-decision-final.md Layer A.

---

## CAP-011: Document Section Generation

Produce ~15 document section types in DOCX/PDF/HTML output: auto-generated from slide data
(executive_summary, risk_register, action_tracker, etc.) and manually authored (methodology,
scope, approval).

**Priority:** P0 | **Grounding:** q1-decision-final.md §5.

---

## CAP-012: Math and LaTeX Rendering

Render inline (`$...$`) and display (`$$...$$`) math via mode-based parser switching.
Support `@{var}` inside math blocks. Produce OMML for PPTX/DOCX, MathML/KaTeX for HTML.

**Priority:** P1 | **Grounding:** q1-decision-final.md §9.

---

## CAP-013: Chart Rendering from Data

Render full-slide data charts (`slide chart:`) and inline chart figures as SVG via the
plotters-backed ChartRenderer plugin. Support bar, line, pie, scatter, area, histogram,
stacked bar in v1.0.

**Priority:** P1 | **Grounding:** q1-decision-final.md §8, q2-decision-final.md (chart type).

---

## CAP-014: Diagram Rendering (Mermaid)

Render full-slide Mermaid diagrams (`slide diagram:`) as SVG via the DiagramRenderer plugin.
Ships v1.0; implementation approach resolved by Spike S14.

**Priority:** P1 | **Grounding:** q2-decision-final.md (diagram type), q3-decision-final.md §2.

---

## CAP-015: Writing Register Support

Distinguish three writing registers: `notes` (presenter-facing), `report` (reader-facing
formal prose), `detail` (document-only extended analysis). Each register routes to the
correct output format exclusively.

**Priority:** P0 | **Grounding:** q1-decision-final.md §3.
