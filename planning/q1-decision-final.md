---
title: "Q1 Decision — Computation, Content Model, and Product Scope"
date: 2026-05-24
status: LOCKED
decided_by: human
research_inputs:
  - planning/python-reference-deep-read.md (R1)
  - planning/brand-template-patterns.md (R2)
  - planning/dsl-competitor-analysis.md (R3)
  - planning/ooxml-foundations.md (R4)
  - planning/wcag-for-slides.md (R5)
  - planning/ir-prior-art.md (R6)
  - planning/composition-prior-art.md (R7)
  - planning/writing-register-research.md (R8)
  - planning/document-content-vocabulary.md (R9)
  - planning/pptx-element-taxonomy.md (R10)
  - planning/math-latex-research.md (R11)
last_updated: 2026-05-24
update_notes: "Reconciled with Q4-Q25 decisions. Fixed 8 contradictions (Q8/Q12/Q14/Q19/Q20/Q21 scope, variants inheritance, @include interpolation). Added Section 0 (product vision), progressive examples, dual-output example, brand.toml example, Q19-Q21 v1.0 features."
---

# Q1 Decision: Computation, Content Model, and Product Scope

## Original Question
"Does the DSL have computation?"

## How Q1 Expanded
During discussion, Q1 grew from a simple computation question into the foundational product scope decision. The human explored: live data-reactive slides, Word document generation, writing registers, document section types, brand template bridge for documents, chart rendering, PowerPoint element coverage, math/LaTeX syntax, and the full version roadmap. All of these are captured here as binding decisions.

## Decision Summary (one paragraph)
slideforge v1.0 ships as a DATA-REACTIVE BRANDED DOCUMENT PLATFORM. The DSL supports computation rungs 1-9 (variables, expressions, @data binding, @for iteration, @if conditionals, ~15 built-in functions, pipe filters) but NOT user-defined functions (v2). It produces 5 output formats from a single .sf source: PPTX, DOCX, PDF, HTML, and a live web preview. Content uses three writing registers: `notes` (presenter-facing), `report` (reader-facing formal), and `detail` (document-only extended analysis). Math uses standard LaTeX delimiters ($...$, $$...$$) with mode-based parsing and @{var} interpolation inside math. Charts render as SVG via the plotters crate. The brand template bridge works bidirectionally for both .pptx and .docx formats.

---

## Section 0: Product Vision

### The Live Data-Reactive Deck
slideforge's transformative capability is LIVE DATA REACTIVITY. A deck connected to data sources auto-refreshes — in watch mode, in CI/CD pipelines, and in the web preview.

Use cases this unlocks (that NO competitor offers):
1. **Nightly metrics briefs on autopilot.** A CI job runs `slideforge build metrics.sf` at 6am. It fetches KPIs from an API, renders 25 branded slides + a formal report, posts the .pptx and .docx to Slack.
2. **Live incident dashboards as slides.** `slideforge watch` on a wall monitor shows current incident status, auto-refreshing every 5 minutes from a JSON API.
3. **Client-specific report generation.** One .sf template + `clients.json` → batch-render 50 branded client reports, each with their data, their logo, their talk tracks.
4. **Self-documenting data pipelines.** The .sf file IS the source of truth for both the presentation and the data it displays. No "which Python script generated this?" problem.

### Updated Tagline
> *"Branded documents from structured data — one source, every format."*

### Competitive Positioning: Beyond Typst
| Capability | Typst | slideforge |
|-----------|-------|-----------|
| Office-native output (.pptx/.docx with editable text, master layouts, speaker notes) | No — PDF only | Yes |
| Live data reactivity (polling APIs, auto-refresh, websocket preview) | No — Static files | Yes |
| Brand template ingestion (load any corporate .pptx/.docx template) | No — Typst-internal templates | Yes |
| Domain-specific slide intelligence (23+ opinionated types with semantic validation) | No — General-purpose | Yes |
| Multi-format from single source (PPTX + DOCX + PDF + HTML + preview) | No — PDF only | Yes |
| Bidirectional brand bridge (.pptx ↔ .toml ↔ synthesis) | No | Yes |

---

## Section 1: Computation Model

### Level: Data-Reactive Declarative (Rungs 1-9)

The DSL supports progressive complexity. Each rung adds ONE concept:

| Rung | Feature | Concept added |
|------|---------|---------------|
| 1 | Static slides | Slide types + string fields |
| 2 | Variables | `vars:` block + `{{ var }}` interpolation |
| 3 | Set rules | `set <type>: <defaults>` at deck scope |
| 4 | Includes | `@include "path.sf"` — complete slides AND fragments (partial blocks inside slide fields) |
| 5 | Variants | `tags:` + `variants:` block with include/exclude filters |
| 6 | Data binding | `@data name from "file.json"` / `"file.csv"` / `"https://..."` |
| 7 | Expressions | `{{ expr | filter }}` with math/formatting pipes |
| 8 | Iteration | `@for item in collection:` — dynamic slides/content |
| 9 | Conditionals | `@if condition:` — include/exclude based on data |
| 10 (v2) | Functions | `@fn name(params):` — user-defined reusable patterns |

### Progressive Complexity Examples

**Rung 1 — A PM who's never seen code:**
```
slide title:
  color blue
  title "Quarterly Business Review"
  subtitle "Q2 2026"

slide content:
  title "Revenue Summary"
  bullets:
    - "ARR grew 12% YoY"
    - "Net retention at 115%"
  takeaway "Strong quarter."

slide end:
  color blue
```

**Rung 2-3 — Variables + set rules:**
```
vars:
  client "Acme Corp"
  quarter "Q2 2026"

set severity_cards:
  color_high red
  color_medium orange

slide title:
  color blue
  title "{{ client }} — {{ quarter }}"
```

**Rung 6-9 — Data-reactive (the live-refresh power):**
```
@data kpis from "https://api.acme.com/v1/kpis"
@data incidents from "incidents.json"

@if incidents.sev1_count > 0:
  slide severity_cards:
    title "Active SEV-1s ({{ incidents.sev1_count }})"
    @for inc in incidents.where(severity: "SEV-1"):
      card:
        severity "critical"
        title {{ inc.title }}
        owner {{ inc.commander }}
        label "CRITICAL: {{ inc.title }}"
        alt "Critical incident: {{ inc.title }}"

slide chart:
  title "ARR Trend"
  type bar
  data {{ kpis.monthly }}
  x "month"
  y "arr"
  color brand.primary
  alt "Bar chart showing 12-month ARR trend"
```

### Variable Interpolation
- Syntax: `{{ var_name }}` in string fields (text mode)
- Scope: deck-level `vars:` block + per-variant `vars:` overrides
- Names: `[a-z][a-z0-9_]*` (snake_case only)
- Undefined variables: hard compile error with line+column diagnostic
- Escape: `\{{` for literal double-brace in text

### Data Binding
- Directive: `@data <name> from "<source>"`
- Sources: local JSON files, local CSV files, HTTP/HTTPS URLs
- Format auto-detection from file extension or Content-Type header
- Build-time fetch: all data resolved at compile time (no runtime network I/O in exported output)
- `slideforge watch` mode: polls data sources on configured interval, re-evaluates on change
- `@include` path interpolation: `@include` paths support `{{ }}` interpolation. Vars are resolved BEFORE includes are processed (evaluation order dependency per Q9).

### Expressions
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `>`, `<`, `>=`, `<=`
- Logical: `and`, `or`, `not`
- Field access: `data.field`, `data.nested.field`
- Method calls: `collection.len()`, `collection.first()`, `collection.where(key: value)`
- Pipe filters: `| number`, `| number(decimals)`, `| percent`, `| pct`, `| signed_pct`, `| currency`, `| date(format)`, `| datetime(format)`, `| time`, `| duration`, `| upper`, `| lower`, `| join(sep)`, `| len`

### Built-in Functions (~15)
`upper`, `lower`, `len`, `sum`, `avg`, `min`, `max`, `first`, `last`, `sort`, `filter`, `take`, `format_number`, `format_date`, `format_percent`, `join`, `duration`, `abs`, `round`, `ceil`, `floor`

### Iteration
- Syntax: `@for item in collection:` (indented block)
- Renders slides, slide elements, or document content per item
- Iterates over finite data only (provably terminating — no unbounded loops)

### Conditionals
- Syntax: `@if condition:` / `@elif condition:` / `@else:` (indented blocks) — all three supported in v1.0 at all scopes (slide, element, field, section) per Q14
- Controls slide inclusion, content block inclusion, field values

### Set Rules
- Syntax: `set <slide_type>: <field> <value>` in deck block
- Precedence: slide-level field > set default > brand-template default
- Lexically scoped to deck block (no cross-file set in v1.0)

### Variants
- Syntax: `variants:` block with named variants
- Per-variant: `include_tags`, `exclude_tags`, `vars:` overrides, `inherits: [parent1, parent2, ...]`
- Multiple inheritance via `inherits:` list. Merge order: deck vars → parents left-to-right (later wins) → variant's own vars (per Q11)
- Build: `slideforge build deck.sf --variant exec-external`
- No variant flag = all slides included
- Precedence: slide vars > variant vars > deck vars
- List replacement (not append)

### Provable Termination
The computation model guarantees termination: @for iterates over finite collections, @if is a branch (not a loop), no recursion, no user-defined functions. This makes formal verification via Kani tractable.

---

## Section 2: Output Formats (5)

| Format | Engine | Brand template? | Math? | Accessibility |
|--------|--------|-----------------|-------|--------------|
| .pptx | ooxmlsdk + custom OOXML generation | Yes — load .pptx or synthesize from .toml | Yes — OMML (native, editable) + SVG fallback | PPTX accessibility checker |
| .docx | ooxmlsdk (Level 1 in v1.0) | Yes — load .docx or synthesize from .toml | Yes — OMML + SVG fallback | Custom OOXML linter + manual Office check |
| .pdf | HTML → Chrome --print-to-pdf | Yes — via HTML styling | Yes — KaTeX → MathML | PDF/UA-1 via veraPDF |
| .html | Direct HTML exporter | Yes — CSS theming from brand.toml | Yes — KaTeX | WCAG AA via @axe-core/playwright |
| Web preview | axum + websocket + SVG canvas | Yes — live brand preview | Yes — KaTeX | WCAG AA via axe-core |

---

## Section 3: Writing Registers

| Field | Purpose | .pptx | .docx | .pdf/.html |
|-------|---------|-------|-------|------------|
| `notes` | Presenter-facing (conversational, first-person, delivery guidance) | Speaker notes pane | Omitted (or optional appendix) | Hidden notes panel (toggle) |
| `report` | Reader-facing (formal, third-person, standalone, citations) | Omitted | Body narrative paragraphs | Same as docx |
| `detail` | Document-only extended analysis (tables, deep dives, per-item breakdowns) | Omitted | Extended sub-sections | Same as docx |
| `takeaway` | Cross-format callout (same text everywhere) | Bottom callout bar | Pull-quote / highlight box | Callout |

### Why separate registers
R8 research confirmed: talk tracks and reports are "linguistically irreconcilable registers" (McKinsey Pyramid Principle, NIST SP 800-61, US Military FM 6-0 SITREP vs AAR). No tool successfully auto-converts between them. The DSL models them as separate fields sharing a common data foundation.

### Progressive depth
- No notes/report/detail → skeletal document (headings + figures)
- + notes → slides get speaker notes; document unchanged
- + report → document gets narrative prose
- + detail → document gets extended analysis sections

### Dual-Output Example: One .sf → PPTX + DOCX

The same .sf file produces both a slide deck AND a formal report:

```
@data incidents from "incidents.json"

slide severity_cards:
  title "Impact Assessment"
  @for system in incidents.affected_systems:
    card:
      severity {{ system.impact_level }}
      title {{ system.name }}
      label "{{ system.impact_level | upper }}: {{ system.name }}"
      alt "{{ system.name }}: {{ system.impact_level }} severity"

  takeaway "{{ incidents.affected_systems | len }} systems impacted. All restored."

  notes """
    Walk through each card. Emphasize the 23-min detection time.
    Expect questions on CI/CD outage duration.
  """

  report """
    The impact assessment identified {{ incidents.affected_systems | len }}
    systems with confirmed exposure. Detection latency of
    {{ detection_minutes }} minutes was within the {{ sla_minutes }}-minute
    SLA threshold (IRP-2025, §4.2).
  """

  detail """
    ## System-by-System Impact Analysis

    @for system in incidents.affected_systems:
      ### {{ system.name }} ({{ system.impact_level | upper }})

      {{ system.detailed_narrative }}

      | Metric | Before | During | After |
      |--------|--------|--------|-------|
      | Availability | {{ system.avail_before | pct }} | {{ system.avail_during | pct }} | {{ system.avail_after | pct }} |

      **Root cause:** {{ system.root_cause }}
  """
```

**In .pptx:** severity cards on screen, notes in speaker notes panel, report/detail ignored.
**In .docx:** slide title → heading, cards → risk table, takeaway → callout box, report → body prose, detail → extended sub-sections with per-system tables.

---

## Section 4: Slide Types (31)

31 built-in types in v1.0 (23 original seed + 8 new added in Q2). See q2-decision-final.md for full details.

### Original 23 (from seed)
title, content, two_column, content_stat, stat_callout, stats_summary, highlight, highlight_boxes, split_contrast, card_rows, severity_cards, numbered_actions, vertical_timeline, horizontal_timeline, enhanced_table, table, status, progress_bar, metric_tree, formula, weighted_composite, end, key_metrics (alias for stat_callout)

### 8 New Types (added in Q2)
| Type | Description |
|------|-------------|
| chart | Full-slide data chart from @data. SVG via plotters. |
| toc | Auto-generated table of contents. |
| agenda | Meeting agenda with items, times, owners. |
| quote | Full-slide testimonial/pull-quote. |
| grid | Multi-column/multi-row flexible layout. |
| bio | Speaker/team member profile. |
| diagram | Full-slide Mermaid diagram rendering (SVG). Ships v1.0. |
| team | Team roster grid. |

---

## Section 5: Document Section Types (~15)

### Auto-generated from slide data (zero manual authoring)
| Section type | Generated from | Content |
|-------------|----------------|---------|
| cover_page | metadata + brand | Title page with logo, date, author |
| executive_summary | All slides' `takeaway` fields | Composed summary of key findings |
| table_of_contents | Slide titles as section headings | Auto-generated TOC |
| risk_register | severity_cards slides | Consolidated risk register table |
| action_tracker | numbered_actions slides | Action item table with status/owner/due |
| kpi_scorecard | stat_callout / key_metrics slides | Formatted KPI table |
| timeline_narrative | timeline slides | Chronological event list/table |
| comparison_matrix | split_contrast / two_column slides | Side-by-side comparison table |
| appendix | Slides tagged `appendix` | Appendix sections |
| glossary | Auto-collected from defined terms | Term → definition list |
| references | Auto-collected from footnotes/citations | Reference list |

### Document-only sections (manual authoring)
| Section type | Purpose | DSL syntax |
|-------------|---------|------------|
| methodology | How analysis was conducted | `section methodology:` block with `report` field |
| scope | What's in/out of scope | `section scope:` block with `report` field |
| approval | Signature block | `section approval:` block with `signatories:` list |

### Document structure control
```
document:
  structure:
    - cover_page
    - executive_summary
    - table_of_contents
    - section: methodology
    - section: scope
    - slides: [1-5]
    - risk_register
    - action_tracker
    - kpi_scorecard
    - section: approval
    - appendix
    - glossary
    - references
```
Default (if no `document:` block): cover → exec summary → TOC → slides in order → appendices.

---

## Section 6: Document Content Elements (19 v1.0-core)

| Category | Elements |
|----------|----------|
| Structural | Headings (H1-H4), Table of Contents, Title/cover page, Page breaks |
| Block-level | Paragraphs, Tables (data-driven via @for), Ordered/unordered lists, Callout/admonition boxes (info/warning/danger/tip/note), Code blocks (syntax highlighted), Block quotes, Figures with captions |
| Inline | Bold, italic, code, links, Footnotes (`{{ footnote("...") }}`), Cross-references (`{{ ref("id") }}`, `{{ figref(n) }}`) |
| Data-driven | Data tables via @for, Charts (SVG via plotters) |
| Chrome | Headers/footers (from brand), Watermarks |

---

## Section 7: PowerPoint Elements (35 v1.0-core)

Shapes (auto-shapes, text boxes, lines, connectors), rich text (runs, paragraphs, formatting), styled tables (banded rows, header rows, merged cells), embedded images (PNG/JPEG/SVG), URL hyperlinks, charts-as-SVG-image, full master/layout/theme system (11 standard + ~20 custom layouts per R2), solid + gradient backgrounds, slide sections, slide hide, core metadata (title/author/subject/keywords), speaker notes, clrMapOvr for dark-themed layouts, placeholder inheritance (type-matching at layout level, idx-matching at slide level per R4).

---

## Section 8: Charts & Data Visualization

| Decision | Answer |
|----------|--------|
| Rendering engine | SVG via `plotters` crate (pure Rust, single pipeline for all formats) |
| v1.0 chart types | Bar/column, line, area, pie/donut, scatter, histogram, stacked bar |
| Embedding | SVG image in PPTX/DOCX; inline SVG in HTML/PDF/preview |
| DSL syntax | `{{ chart.bar(data: kpis, x: "month", y: "value", title: "...", color: brand.primary) }}` |
| Native OOXML ChartML | Deferred to v2 (editable charts in Office) |

---

## Section 9: Math / LaTeX Support

| Decision | Answer |
|----------|--------|
| Math delimiters | `$...$` (inline), `$$...$$` (display) — standard LaTeX |
| Parser mode | Mode-based: text mode vs math mode. `$` toggles mode. |
| Text interpolation | `{{ var }}` — active in text mode, DISABLED in math mode |
| Math interpolation | `@{var}` — active in math mode, ERROR in text mode |
| Math-interp scope | `@{var}` emits value as LaTeX literal (number or \text{string}) |
| LaTeX → MathML | `pulldown-latex` v0.7.1 (pure Rust) |
| MathML → OMML | Microsoft `mml2omml.xsl` via libxslt FFI (for PPTX/DOCX) |
| LaTeX → HTML | KaTeX (Rust bindings or pre-rendered) |
| Cross-renderer | OMML + SVG fallback image in PPTX/DOCX (OMML degrades to image in non-Office apps) |
| Accessibility | KaTeX MathML for HTML; SVG alt with LaTeX source for Office |
| Equation numbering | Deferred to v2 (`{{ eqref(n) }}`) |

### Syntax conflict resolution (14 conflicts analyzed)
| Conflict | Resolution |
|----------|-----------|
| `{{ }}` vs `{}` in math | `{{ }}` disabled in math mode; `{}` is always LaTeX grouping |
| `_` italic vs subscript | `_` is italic in text mode, subscript in math mode |
| `^` superscript | Reserved in text mode, superscript in math mode |
| `#` comment vs macro param | `#` is comment at start-of-line in text mode only; no conflict in math |

---

## Section 10: Brand Template Bridge

| Direction | PPTX | DOCX |
|-----------|------|------|
| Load existing template | `--template brand.pptx` | `--template report.docx` |
| Synthesize from .toml | Full PPTX synthesis (theme, master, 31 layouts, notes/handout masters) | Full DOCX synthesis (heading/paragraph/table styles, header/footer, cover page) |
| Extract brand | `slideforge extract-brand deck.pptx → brand.toml` | `slideforge extract-brand report.docx → brand.toml` |

### Unified brand.toml
Single file drives branding for ALL output formats:
- `[colors]` → PPTX theme colors (dk1/lt1/dk2/lt2/accent1-6) + DOCX heading/emphasis colors + HTML/PDF CSS
- `[fonts]` → PPTX theme fonts (major/minor) + DOCX style fonts + HTML/PDF font-family
- `[logo]` → PPTX master + DOCX header
- `[footer]` → PPTX footer placeholder + DOCX footer
- `[templates]` → paths to existing .pptx and .docx templates

### Example brand.toml
```toml
[brand]
name = "1898 & Co."
tagline = "Industrial Cybersecurity"

[colors]
primary = "#003766"       # → pptx: dk1/accent1 | docx: Heading 1 | html: --brand-primary
accent1 = "#FF6F00"       # → pptx: accent2 | docx: Emphasis | html: --brand-accent
accent2 = "#6B2D8B"       # → pptx: accent3 | docx: Heading 2
text = "#333333"          # → pptx: tx1 | docx: Body text
background = "#FFFFFF"    # → pptx: bg1 | docx: Page background
danger = "#CC0000"        # → severity_cards color_high
warning = "#FF8C00"       # → severity_cards color_medium
success = "#228B22"       # → severity_cards color_low

[fonts]
heading = "Aptos Display" # → pptx: major font | docx: Heading styles
body = "Aptos"            # → pptx: minor font | docx: Normal style
mono = "Cascadia Code"    # → code blocks in all formats
cjk = "Yu Gothic"         # → CJK fallback in theme font scheme

[logo]
path = "assets/1898-logo.png"
width = "1.5in"
position = "header-right" # → pptx: master slide | docx: header

[footer]
text = "CONFIDENTIAL — {{ brand.name }}"
show_page_numbers = true
show_date = true

[templates]
pptx = "templates/1898-Presentation-V3.0.pptx"
docx = "templates/1898-Report-V2.0.docx"
```

---

## Section 11: Inline Formatting

| Syntax | Effect | In text mode | In math mode |
|--------|--------|--------------|--------------|
| `**bold**` | Bold text | Yes | N/A (use \textbf{}) |
| `_italic_` | Italic text | Yes | N/A (default italic in math) |
| `` `code` `` | Code span | Yes | N/A |
| `[text](url)` | Hyperlink | Yes | N/A |
| `$...$` | Inline math | Enters math mode | N/A |
| `$$...$$` | Display math | Enters math mode | N/A |
| `{{ var }}` | Text interpolation | Yes | DISABLED |
| `@{var}` | Math interpolation | ERROR | Yes |
| `{{ footnote("...") }}` | Footnote marker | Yes | N/A |
| `{{ ref("id") }}` | Cross-reference | Yes | N/A |
| `{{ figref(n) }}` | Figure reference | Yes | N/A |

Ships in v1.0 (per Q8): `^sup^` (superscript), `~sub~` (subscript), `~~del~~` (strikethrough), `==highlight==` (highlighted text).
Reserved for v2: emoji shortcodes (`:rocket:`), `{{ color() }}` inline color spans.

---

## Section 12: Crate Layout (19 crates)

Plugin-first architecture (Q3 decision). All functionality runs through plugin traits in `slideforge-plugin-api`. See q3-decision-final.md for full details.

```
crates/
├── slideforge-plugin-api/ # THE foundation — all 10 trait definitions
├── slideforge/            # Main library — assembles PluginRegistry
├── slideforge-cli/        # CLI binary
├── slideforge-syntax/     # Parser (chumsky) — DSL → AST (core, NOT a plugin)
├── slideforge-eval/       # AST → semantic IR (Deck) (core, NOT a plugin)
├── slideforge-layout/     # Semantic IR → LaidOutDeck (core, NOT a plugin)
├── slideforge-pptx/       # Bundled Exporter plugin (ooxmlsdk)
├── slideforge-docx/       # Bundled Exporter plugin (ooxmlsdk)
├── slideforge-pdf/        # Bundled Exporter plugin (HTML → Chrome --print-to-pdf)
├── slideforge-html/       # Bundled Exporter plugin
├── slideforge-preview/    # Bundled Exporter/preview plugin (axum + websocket + SVG)
├── slideforge-data/       # Bundled DataSource plugins (json/csv/yaml/toml/http/xlsx/sqlite)
├── slideforge-charts/     # Bundled ChartRenderer plugin (plotters)
├── slideforge-diagrams/   # Bundled DiagramRenderer plugin (mermaid, ships v1.0)
├── slideforge-math/       # Bundled MathRenderer plugin (pulldown-latex + KaTeX)
├── slideforge-validate/   # Bundled Validator plugins (WCAG + brand contrast)
├── slideforge-brand/      # Bundled BrandProvider plugin (.pptx/.docx ↔ .toml ↔ synthesis)
├── slideforge-types/      # Bundled SlideType + SectionType plugins (31 types)
└── slideforge-package/    # Package manager (install, @import resolution, sf.lock, git-based) — Q19
```

---

## Section 13: Version Roadmap

### v1.0 — "The Foundation"
*A data-reactive branded document platform. One source, every format.*

Everything in Sections 1-12 above, plus:
- 31 opinionated slide types (23 seed + 8 new: chart/toc/agenda/quote/grid/bio/diagram/team)
- ~15 document section types (auto-generated + manual)
- Plugin-first architecture: 10 extensibility surfaces, all bundled plugins dog-food the same traits as future external plugins
- @include fragments: partial blocks inside slide fields, shared definitions, shared set rules (per Q12)
- @if / @elif / @else: full conditional chain at all scopes — slide, element, field, section (per Q14)
- Inline formatting: `^sup^`, `~sub~`, `~~del~~`, `==highlight==` all ship v1.0 (per Q8)
- Full package model: `slideforge package install`, `@import`, `sf.lock`, git-based packages (per Q19)
- Cargo-style workspace: `[workspace]` in slideforge.toml, `.sfconfig` family cascade, `slideforge config explain`, `slideforge build --workspace` (per Q20)
- Auto-apply defaults/ directory (Hugo-style convention, per Q21)
- Structured shape DSL (`shape:` block with type/position/fill/text/alt) for custom visuals without raw XML escape hatch
- Mermaid diagram rendering ships in v1.0 via the DiagramRenderer plugin (Spike S14 determines implementation approach)
- Production-grade quality bar: Kani proofs, cargo-fuzz, cargo-mutants, WCAG AA, multi-renderer visual parity, signed releases, SBOM
- CI/CD matrix: clippy + fmt + test + bench + audit + deny + mutants + fuzz smoke + cross-platform + LibreOffice render + accessibility
- CLI: `slideforge build`, `slideforge watch`, `slideforge extract-brand`, `slideforge package install`, `slideforge config explain`

### v1.x — "Polish & Expansion"
- More chart types (box-whisker, waterfall, funnel, treemap, sunburst)
- Figure auto-numbering + table of figures
- Nested lists, definition lists
- DOCX Level 2 (proper flow layout, formatted detail sub-sections, heading hierarchy)
- comemo incremental compilation (< 50ms incremental rebuilds)
- slideforge fmt (canonical formatter, deterministic round-trip)
- slideforge lint (standalone linter)
- Performance optimizations

### v2 — "The Platform"
- User-defined functions: @fn name(params):
- SmartArt-as-grouped-shapes (org charts, process flows, Venn diagrams)
- Progressive bullet reveal animation (entrance: one-by-one on click)
- Zoom sections (non-linear navigation — PowerPoint desktop only)
- Native OOXML charts (ChartML — editable in Office)
- Dynamic plugin loading (WASI modules / shared libraries via `slideforge plugin install`)
- Plugin data connectors (Grafana, Datadog, PagerDuty, Jira, Salesforce) — builds on the DataSource trait shipping in v1.0
- Mixin syntax: @mixin / @include with parameters
- Component/parameterized slide types (reserved syntax ships in v1.0 parser; implementation in v2)
- Equation auto-numbering + {{ eqref(n) }}
- Bibliography/citations (BibTeX integration)
- Full RTL/BiDi support
- Document variants (separate document structures per variant)
- @if expressions in fields (inline conditionals)
- AI-assisted content (slideforge suggest)
- Presentation mode (slideforge present — dual-monitor presenter view)
- LaTeX/Beamer output format (slideforge-latex exporter crate)
- Custom Rust OMML converter (drop libxslt XSLT dependency)
- Math interpolation without @{} (if a cleaner syntax emerges)
- Emoji shortcodes (`:rocket:`) and `{{ color() }}` inline color spans

### v3 — "The Vision"
- Morph transitions (shape-level morphing between slides)
- Video export (MP4 with narration + slide timings + transitions)
- Multi-document pipeline (one @data source → deck + report + email + dashboard)
- Template marketplace (community-published brand templates + slide kits)
- Plugin ecosystem (third-party exporters: Google Slides, Keynote native)
- Real-time collaboration (multi-author with slide-level merge)
- Embedded analytics layer (slideforge as the reporting tier)
- Custom animation sequences (beyond progressive reveal)
- Spreadsheet export (.xlsx for data-heavy decks)

### Out of Scope (never)
VBA/macros, ActiveX controls, OLE embedding, form fields, digital signatures (cryptographic), DRM/IRM, 3D models (.glb/.gltf), ink/pen annotations, tracked changes, mail merge, print-specific layout (widows/orphans — let Word handle), co-authoring real-time protocol, pivot tables, embedded spreadsheet editing.

---

## Section 14: Research Inputs

| ID | Title | Key finding | File |
|----|-------|-------------|------|
| R1 | Python reference deep-read | 23 types catalogued; silent color fallbacks; string-prefix bold = anti-pattern | planning/python-reference-deep-read.md |
| R2 | Brand template patterns | 31 layouts needed; clrMap inheritance; element ordering strict | planning/brand-template-patterns.md |
| R3 | DSL competitor user-pain | 70+ citations; no `---` separator; no YAML coercion; `{{ }}` interpolation traps | planning/dsl-competitor-analysis.md |
| R4 | OOXML foundations | ooxmlsdk = serializer only; element ordering enforced; placeholder matching keys differ by level | planning/ooxml-foundations.md |
| R5 | WCAG AA for slides | 23 criteria mapped; SVG > canvas for a11y; axe-core for HTML, veraPDF for PDF | planning/wcag-for-slides.md |
| R6 | IR prior art | Two-IR model (Deck + LaidOutDeck); Pandoc ADT + Typst Frame hybrid; Hash-stable from day 1 | planning/ir-prior-art.md |
| R7 | Composition/mixins | Partial adoption; set rules + variants; no user-defined components v1.0 | planning/composition-prior-art.md |
| R8 | Writing registers | Talk track not report prose (McKinsey/NIST/military confirm); dual-track with shared data | planning/writing-register-research.md |
| R9 | Document content vocabulary | 67 elements; 19 v1.0-core; charts via SVG/plotters; auto-generated sections | planning/document-content-vocabulary.md |
| R10 | PowerPoint element taxonomy | 160+ elements; 35 v1.0-core; charts = biggest gap; morph/zoom = worst cross-renderer | planning/pptx-element-taxonomy.md |
| R11 | Math/LaTeX syntax | $...$ with mode-based parsing; @{var} in math; pulldown-latex → OMML; 14 conflicts resolved | planning/math-latex-research.md |
