---
document_type: domain-spec-section
level: L2
section: "ubiquitous-language"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/planning/domain-research.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q2-decision-final.md
  - .factory/planning/q3-decision-final.md
  - .factory/planning/q4-q15-decisions.md
  - .factory/planning/q16-q25-decisions.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Ubiquitous Language

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Precise definitions for all domain terms used across agents, code, and specs.

---

## Core Domain Terms

**Deck**
The top-level compiled document. One Deck corresponds to one .sf source file (after all
includes are expanded). A Deck has exactly one brand, one list of slides, and optional
variant definitions. The aggregate root of the Authoring bounded context.

**Slide**
A single page or frame within a Deck. Each slide has exactly one `SlideType` and a set
of type-specific fields. Slides are internal entities within the Deck aggregate — they
have no lifecycle independent of the Deck.

**SlideType**
One of 31 named visual patterns (e.g., `severity_cards`, `chart`, `content`). A SlideType
defines what fields are required, how the content is laid out, and how it renders across
formats. Implemented as a `SlideType` plugin trait instance.

**Alias**
A named preset on an existing SlideType. An alias cannot add new fields — only preset
existing field defaults. Resolved at parse time to the underlying type plus merged defaults.
Example: `alias incident_card = severity_cards:`.

**ContentBlock**
A structural unit of content within a slide or document section. Types include paragraphs,
bullet lists, tables, code blocks, callouts, figures, and data-driven blocks from `@for`.

**TextRun**
An inline-formatted run of text within a paragraph or heading. Carries formatting
attributes (bold, italic, link, math, etc.) but no block structure.

**WritingRegister**
One of three linguistically distinct content tracks within a slide:
- `notes`: presenter-facing, conversational, first-person delivery guidance
- `report`: reader-facing, formal, third-person, standalone-readable prose
- `detail`: document-only extended analysis, tables, deep dives
Each register routes to a different output format: notes → PPTX speaker notes pane;
report/detail → DOCX body; all formats ignore registers not applicable to them.

**Variable**
A named value declared in a `vars:` block or imported from a `@data` binding. Interpolated
into string fields via `{{ var }}` syntax. Must follow `[a-z][a-z0-9_]*` naming. Undefined
at compile time = compile error.

**Expression**
A computed value within `{{ ... }}` interpolation. Supports arithmetic, comparison,
logical operators, field access, method calls, and pipe filters.

**DataSource**
A named data binding loaded at compile time via `@data name from "source"`. Sources include
JSON files, CSV files, YAML files, TOML files, HTTP/HTTPS URLs, Excel spreadsheets, and
SQLite databases.

---

## DSL Structural Terms

**.sf file (source file)**
A slideforge source file. Indentation-significant (spaces only; tabs are rejected).
Extension `.sf`. May include other `.sf` files via `@include` or reference installed
packages via `@import`.

**Frontmatter / Metadata block**
The `metadata:` block at the top of a .sf file declaring `slideforge_version`, `title`,
`author`, `date`, `lang`, and `brand` path.

**vars: block**
A deck-level or variant-level block declaring named variables available for `{{ }}` interpolation.

**set rule**
A deck-scoped or workspace-scoped default field value for a SlideType or SectionType.
Declared via `set <type>: <field> <value>`. Lower precedence than slide-level field values.

**variant**
A named build configuration within a `variants:` block that filters slides by tags and
overrides variables. Built with `slideforge build deck.sf --variant <name>`.

**tag**
A string label on a slide used for variant include/exclude filtering. Declared in `tags:` list
on a slide.

**@include**
A directive that injects a local .sf file at the declaration point. Can inject complete slides,
set rules, vars, aliases, or fragments inside slide fields.

**@import**
A directive that injects content from an installed package. Distinct from `@include`.
Resolved against `sf.lock`.

**@data**
A directive that loads external data into a named binding. Evaluated at compile time.
Syntax: `@data <name> from "<source>"`.

**@for**
An iteration directive. Generates slides, content blocks, or document sections for each item
in a collection. Always iterates over finite data (provably terminating).

**@if / @elif / @else**
Conditional rendering directives. Available at slide-level, element-level, field-level,
and section-level scope.

---

## Architecture Terms

**IR (Intermediate Representation)**
Two distinct IR types exist in the pipeline:
- `Deck IR`: semantic, pre-layout. Produced by the evaluator. Input to the layout engine.
- `LaidOutDeck IR`: geometric, post-layout. Produced by the layout engine. Input to all exporters.

**EMU (English Metric Unit)**
The internal unit for all coordinates and dimensions. 914400 EMU = 1 inch. All layout
calculations use integer EMU — no floating point. Required for `Hash + Eq` compatibility.

**Brand**
The complete set of visual identity parameters for a Deck: color palette, typography,
logo, footer, and references to optional .pptx and .docx template files. The stable contract
of the Branding bounded context.

**brand.toml**
A TOML file declaring brand configuration with `[colors]`, `[fonts]`, `[logo]`, `[footer]`,
and `[templates]` sections. Drives all five output formats from a single source.

**Master (PPTX)**
The slide master in an OOXML presentation — defines the base visual template, theme, and
chrome (logo, footer placeholders) shared by all slides. slideforge v1.0 uses a single
master per deck (multi-master is v2).

**Plugin**
An implementation of one of the 10 `dyn Trait` extensibility surfaces. All bundled plugins
(pptx, docx, plotters, mermaid, etc.) use the same traits as future external plugins.

**PluginRegistry**
The runtime registry of all loaded plugin instances. Assembled at binary startup from
the 19 bundled plugin crates.

---

## Output Format Terms

**PPTX**
Office Open XML Presentation format (.pptx). The primary v1.0 output. Must render correctly
in PowerPoint, Keynote, Google Slides, and LibreOffice (multi-renderer fidelity invariant).

**DOCX**
Office Open XML Document format (.docx). Report-oriented output consuming the `report`
and `detail` writing registers.

**Tagged PDF**
A PDF with structure tags (H1, P, Table, etc.), reading order, and accessibility metadata
meeting PDF/UA-1 (ISO 14289) requirements. The slideforge PDF output must be tagged.

**Web Preview**
A live browser preview served by an axum WebSocket server. SVG-based canvas with ARIA.
Auto-refreshes in watch mode.

---

## Pipeline Stage Terms

**Parse**
The first pipeline stage. Input: .sf source bytes. Output: AST. Accumulates all parse
errors; never fails on first. Implemented in `slideforge-syntax`.

**Evaluate**
The second pipeline stage. Input: AST. Output: `Deck` IR. Resolves variables, expands
@for iterations, evaluates @if conditionals, loads data sources.

**Layout**
The third pipeline stage. Input: `Deck` IR + `Brand`. Output: `LaidOutDeck` IR.
Computes geometric positions, text flow, and overflow detection in EMU coordinates.

**Export**
The fourth pipeline stage. Input: `LaidOutDeck` IR + `Deck` IR + `Brand`. Output: format
bytes. One Exporter plugin per format, run in parallel.

---

## Package / Workspace Terms

**Workspace**
A multi-deck project declared in `slideforge.toml` with `[workspace]` section. Multiple
.sf files share a single brand, set of packages, and sf.lock.

**sf.lock**
The lockfile recording exact versions and checksums of all installed packages. Committed
to version control; ensures reproducible builds.

**Package**
A versioned, git-distributed collection of .sf content and/or plugin WASM modules
(v2). Declared in `slideforge.toml [dependencies]` and locked in `sf.lock`.

**.sfconfig**
A directory-level configuration file for overriding workspace defaults. Maximum cascade
depth: 3 levels. Small surface area (defaults, set rules, vars, variants only).
