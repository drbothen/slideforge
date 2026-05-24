---
title: Document Content Vocabulary -- Complete Element Taxonomy
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: product-owner + architect + human-reviewer
---

# Document Content Vocabulary

## Executive Summary

This document catalogs **67 distinct content elements** needed for professional document generation across seven domains (cybersecurity IR, consulting, audit, engineering, executive briefings, military, regulatory). Of these:

- **19 elements** are v1.0 core (must-ship)
- **10 elements** are v1.0 stretch (ship if time permits)
- **24 elements** are v2 (explicitly deferred)
- **14 elements** are out of scope

The minimum viable document content model for v1.0 centers on: headings (1-6), paragraphs, ordered/unordered/nested lists, simple tables, figures with captions, inline formatting (bold/italic/code/links), table of contents, cover page, headers/footers, page numbers, and hyperlinks. These 19 elements, combined with slideforge's existing `report` and `detail` register blocks plus `{{ }}` data interpolation, produce professional-grade DOCX/PDF/HTML output across all target domains.

**Chart rendering recommendation:** SVG-first via the `plotters` Rust crate, embedded as images in both PPTX and DOCX. Defer native OOXML ChartML to v2+. This approach is simpler, produces high-quality vector output, works identically across all export targets, and avoids the complexity of generating embedded Excel workbooks. Quarto uses the same pattern (render to image, embed).

**Top differentiators for slideforge:**
1. **Dual-register content** (`notes` + `report` + `detail`) with shared data interpolation -- no other tool offers this
2. **Auto-generated document elements** (TOC, finding register, risk register, action tracker) synthesized from slide-level structured data
3. **Data-reactive charts** via `{{ chart.bar(...) }}` expressions that render identically in slides and documents

---

## Content Element Taxonomy

### Structural Elements

| # | Element | Description | OOXML Mapping | Pandoc Equivalent | Typst Equivalent | Category |
|---|---------|-------------|---------------|-------------------|------------------|----------|
| S1 | **Heading 1-6** | Section headings at 6 levels | `<w:p>` with `<w:pStyle w:val="Heading1"/>` through Heading6 | `Header Int Attr [Inline]` | `= Heading` through `====== Heading` | v1.0 core |
| S2 | **Title page / cover page** | Document title, author, date, client, classification | First section with custom styles; no native "cover page" element -- composed from paragraphs + images | `Meta` title/author/date rendered via template | `#align(center)[...]` with page break | v1.0 core |
| S3 | **Table of contents** | Auto-generated from headings | `<w:fldSimple w:instr="TOC \o \"1-3\" \h \z \u"/>` field code | `--toc` flag generates TOC | `#outline()` function | v1.0 core |
| S4 | **Section break** | Separates document into sections with independent headers/footers/numbering | `<w:sectPr>` at end of last paragraph in section | `\newpage` or raw block | `#pagebreak()` | v1.0 core |
| S5 | **Appendix** | Sections after main body with letter/number labeling | Same as headings with different numbering; `<w:sectPr>` for separate numbering | Heading with custom numbering | Heading with counter reset | v2 |
| S6 | **Table of figures** | Auto-generated list of figure captions | `<w:fldSimple w:instr="TOC \c \"Figure\""/>` | `--list-of-figures` via template | `#outline(target: figure)` | v2 |
| S7 | **Table of tables** | Auto-generated list of table captions | `<w:fldSimple w:instr="TOC \c \"Table\""/>` | Custom filter required | `#outline(target: table)` | v2 |
| S8 | **Glossary** | Alphabetized term definitions | Definition list or table with heading | `DefinitionList` | Custom function | v2 |
| S9 | **Index** | Back-of-book keyword index | `<w:fldSimple w:instr="INDEX"/>` with `XE` field entries | LaTeX `makeindex` integration | Package-based | Out of scope |
| S10 | **Bibliography** | Formatted reference list | Paragraphs with custom style; no native bib element | `--bibliography` + CSL processing | `#bibliography()` | v2 |

### Block-Level Elements

| # | Element | Description | OOXML Mapping | Pandoc Equivalent | Typst Equivalent | Category |
|---|---------|-------------|---------------|-------------------|------------------|----------|
| B1 | **Paragraph** | Standard prose block | `<w:p><w:r><w:t>text</w:t></w:r></w:p>` | `Para [Inline]` | Implicit from text | v1.0 core |
| B2 | **Unordered list** | Bulleted list items | `<w:p>` with `<w:numPr>` referencing numbering.xml (bullet format) | `BulletList [[Block]]` | `- item` / `list()` | v1.0 core |
| B3 | **Ordered list** | Numbered list items | `<w:p>` with `<w:numPr>` referencing numbering.xml (decimal format) | `OrderedList ListAttributes [[Block]]` | `+ item` / `enum()` | v1.0 core |
| B4 | **Nested list** | Multi-level list nesting (2-3 levels) | `<w:numPr><w:ilvl w:val="1"/>` for indent level | Nested `BulletList`/`OrderedList` | Indented `- item` | v1.0 core |
| B5 | **Simple table** | Row/column data with optional header row | `<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc>...` | `Table Attr Caption [ColSpec] TableHead [TableBody] TableFoot` | `table(columns: N, ...)` | v1.0 core |
| B6 | **Complex table** | Merged cells (horizontal `gridSpan`, vertical `vMerge`), banded rows, styled headers | `<w:gridSpan w:val="N"/>`, `<w:vMerge w:val="restart"/>`, `<w:tblLook>` flags | Same `Table` with ColSpec spanning | `table.cell(colspan: N, rowspan: N)` | v1.0 stretch |
| B7 | **Figure with caption** | Image with numbered caption and optional alt text | `<w:drawing><wp:inline><a:graphic>` + `<w:p>` with caption style + SEQ field | `Figure Attr Caption [Block]` (Pandoc 3.x) | `figure(image(...), caption: ...)` | v1.0 core |
| B8 | **Code block** | Monospaced text block, optionally with language annotation | `<w:p>` with monospaced character style; no native syntax highlighting | `CodeBlock Attr String` | ` ```lang ... ``` ` | v1.0 core |
| B9 | **Code block with syntax highlighting** | Code with colored tokens per language grammar | Run-level character styles (`<w:color>`, `<w:rPr>`) per token; complex to generate | `CodeBlock` rendered via Skylighting | ` ```lang ... ``` ` (built-in highlighting) | v2 |
| B10 | **Blockquote** | Indented quoted text block | `<w:p>` with `<w:pStyle w:val="Quote"/>` or custom indent | `BlockQuote [Block]` | `#quote(block: true)[...]` | v2 |
| B11 | **Callout / admonition** | Styled box: NOTE, WARNING, TIP, IMPORTANT, CAUTION | No native element; composed from table with colored border + icon image, or styled paragraph | `Div` with class `.warning`/`.note` etc. | Custom function via `box()` + `grid()` | v1.0 stretch |
| B12 | **Horizontal rule** | Thematic break / section divider | `<w:p><w:pPr><w:pBdr><w:bottom w:val="single".../>` | `HorizontalRule` | `#line(length: 100%)` | v2 |
| B13 | **Definition list** | Term-definition pairs | No native element; table or styled paragraphs | `DefinitionList [([Inline], [[Block]])]` | `terms(...)` | v2 |
| B14 | **Line block** | Preserved line breaks (poetry, addresses, logs) | `<w:br/>` within a single `<w:p>`, or multiple paragraphs with tight spacing | `LineBlock [[Inline]]` | `#par(leading: ...)` | v2 |
| B15 | **Equation (display)** | Centered mathematical expression | `<m:oMathPara><m:oMath>...` (OMML) | `Math DisplayMath String` | `$ ... $` (display mode) | v2 |
| B16 | **Chart** | Data-driven visualization (bar, line, pie, area, scatter) | Native: ChartML (`<c:chart>`) + embedded `.xlsx`. Image-based: `<w:drawing>` with SVG/PNG | No native support; image embed | No native support; `cetz` package for basic plots | v1.0 stretch |
| B17 | **Mermaid diagram** | Flowchart, sequence, Gantt, etc. from text DSL | Image embed (SVG/PNG rendered externally) | Requires external filter (mermaid-filter) | Requires external rendering | v2 |

### Inline Elements

| # | Element | Description | OOXML Mapping | Pandoc Equivalent | Typst Equivalent | Category |
|---|---------|-------------|---------------|-------------------|------------------|----------|
| I1 | **Bold** | Strong emphasis | `<w:rPr><w:b/>` | `Strong [Inline]` | `*text*` | v1.0 core |
| I2 | **Italic** | Emphasis | `<w:rPr><w:i/>` | `Emph [Inline]` | `_text_` | v1.0 core |
| I3 | **Inline code** | Monospaced text fragment | `<w:rPr><w:rFonts w:ascii="Consolas"/>` + optional background | `Code Attr String` | `` `code` `` | v1.0 core |
| I4 | **Hyperlink** | Clickable URL or internal link | `<w:hyperlink r:id="rIdN"><w:r><w:rPr><w:rStyle w:val="Hyperlink"/>` | `Link Attr [Inline] Target` | `#link("url")[text]` | v1.0 core |
| I5 | **Footnote marker** | Superscript number linking to footnote text | `<w:footnoteReference w:id="N"/>` in run; `<w:footnote w:id="N">` in footnotes.xml part | `Note [Block]` | `#footnote[text]` | v1.0 stretch |
| I6 | **Cross-reference** | Reference to section, figure, table by label | `<w:fldSimple w:instr="REF _RefLabel \h"/>` or PAGEREF | `Cite` or custom filter | `@label` reference syntax | v2 |
| I7 | **Citation** | Reference to bibliography entry | Run with formatted text; no native citation element | `Cite [Citation] [Inline]` | `@key` citation syntax | v2 |
| I8 | **Strikethrough** | Deleted/superseded text | `<w:rPr><w:strike/>` | `Strikeout [Inline]` | `#strike[text]` | v2 |
| I9 | **Superscript** | Raised text (ordinals, math) | `<w:rPr><w:vertAlign w:val="superscript"/>` | `Superscript [Inline]` | `#super[text]` | v2 |
| I10 | **Subscript** | Lowered text (chemical formulas) | `<w:rPr><w:vertAlign w:val="subscript"/>` | `Subscript [Inline]` | `#sub[text]` | v2 |
| I11 | **Inline equation** | Mathematical expression within text | `<m:oMath>...` inline within `<w:r>` | `Math InlineMath String` | `$x^2$` | v2 |
| I12 | **Small caps** | Stylistic text variant | `<w:rPr><w:smallCaps/>` | `SmallCaps [Inline]` | `#smallcaps[text]` | v2 |
| I13 | **Underline** | Underlined text | `<w:rPr><w:u w:val="single"/>` | `Span` with custom attribute | `#underline[text]` | v2 |
| I14 | **Highlight / mark** | Background-colored text for emphasis | `<w:rPr><w:highlight w:val="yellow"/>` | `Span` with custom class | `#highlight[text]` | v2 |

### Data-Driven Elements

| # | Element | Description | OOXML Mapping | Pandoc Equivalent | Typst Equivalent | Category |
|---|---------|-------------|---------------|-------------------|------------------|----------|
| D1 | **Data table** | Table populated from `@data` with `@for` iteration | Same as B5/B6 (table), rows generated at eval time | No native equivalent; requires preprocessing | Loop + table | v1.0 core |
| D2 | **Computed metric** | Inline value from expression: `{{ kpi.arr \| currency }}` | Run with evaluated text | No native equivalent | Scripting inline | v1.0 core |
| D3 | **Conditional content** | `@if` blocks that include/exclude content | Evaluated at compile time; only included content emitted | No native equivalent; preprocessor | `#if condition [...]` | v1.0 core |
| D4 | **Bar chart** | Vertical/horizontal bar chart from data | Image embed (SVG via plotters) or ChartML | External tool -> image | External tool -> image | v1.0 stretch |
| D5 | **Line chart** | Time-series or trend line | Same as D4 | Same as D4 | Same as D4 | v1.0 stretch |
| D6 | **Pie chart** | Proportional breakdown | Same as D4 | Same as D4 | Same as D4 | v1.0 stretch |
| D7 | **Area chart** | Filled area under line(s) | Same as D4 | Same as D4 | Same as D4 | v2 |
| D8 | **Scatter plot** | X-Y data points | Same as D4 | Same as D4 | Same as D4 | v2 |
| D9 | **Stacked bar chart** | Multi-series stacked bars | Same as D4 | Same as D4 | Same as D4 | v2 |
| D10 | **Auto-generated executive summary** | Synthesized from slide takeaways | Paragraphs composed from collected takeaway fields | No equivalent | No equivalent | v1.0 stretch |
| D11 | **Finding register** | Table of findings aggregated from severity_cards/similar | Table with standardized columns (ID, title, severity, status, owner) | No equivalent | No equivalent | v1.0 stretch |
| D12 | **Risk register** | Table of risks aggregated from risk data | Same as D11 with risk-specific columns | No equivalent | No equivalent | v1.0 stretch |
| D13 | **Action tracker** | Table of actions from numbered_actions data | Same as D11 with action-specific columns | No equivalent | No equivalent | v1.0 stretch |

### Chrome / Meta Elements

| # | Element | Description | OOXML Mapping | Pandoc Equivalent | Typst Equivalent | Category |
|---|---------|-------------|---------------|-------------------|------------------|----------|
| C1 | **Header (per-section)** | Text/logo at top of every page | `<w:hdr>` part; linked from `<w:sectPr><w:headerReference>` | Template variable `$header-includes$` | `#set page(header: [...])` | v1.0 core |
| C2 | **Footer (per-section)** | Text at bottom of every page | `<w:ftr>` part; linked from `<w:sectPr><w:footerReference>` | Template variable `$footer$` | `#set page(footer: [...])` | v1.0 core |
| C3 | **Page number** | Auto-incrementing page number in header/footer | `<w:fldSimple w:instr="PAGE"/>` within header/footer | Template page number field | `counter(page).display()` | v1.0 core |
| C4 | **Confidentiality banner** | "CONFIDENTIAL" / "INTERNAL USE ONLY" in header/footer | Text in header/footer with emphasis styling | Custom template | Header/footer text | v1.0 stretch |
| C5 | **Watermark** | Diagonal text overlay ("DRAFT", "CONFIDENTIAL") | `<w:pict>` with VML shape in header, or DrawingML shape behind text | Not natively supported | `#place(...)` with rotation | v2 |
| C6 | **Document metadata** | Title, author, date, version, classification level | `docProps/core.xml` (Dublin Core) + `docProps/app.xml` | YAML front matter -> `Meta` | `#set document(title: ..., author: ...)` | v1.0 core |
| C7 | **Approval/signature block** | Sign-off area with name, title, date, signature line | Composed from table or paragraphs with underscores | No native support | Custom function | v2 |
| C8 | **Classification markings** | Formal classification scheme labels (CUI, FOUO, etc.) | Header/footer text with controlled vocabulary | Custom template | Custom function | v2 |
| C9 | **Running head** | Abbreviated title in header | Text in `<w:hdr>` part | Template variable | Header content | v2 |
| C10 | **Version/revision info** | Document version, revision date | Custom property in `docProps/custom.xml` or body text | YAML metadata | Document metadata | v1.0 stretch |

---

## Cross-Tool Comparison

| Element | Pandoc | Typst | Quarto | LaTeX | AsciiDoc | DITA | Notion |
|---------|--------|-------|--------|-------|----------|------|--------|
| **Headings 1-6** | Native (Header) | Native (=) | Native (#) | Native (\section) | Native (=) | Native (<title>) | Native (H1-3) |
| **Paragraphs** | Native (Para) | Native | Native | Native | Native | Native (<p>) | Native |
| **Ordered/unordered lists** | Native | Native | Native | Native | Native | Native (<ul>/<ol>) | Native |
| **Nested lists** | Native | Native | Native | Native | Native | Native | Native |
| **Tables** | Native (complex) | Native (full) | Native | Native (tabular) | Native (pipes) | Native (<table>) | Native (basic) |
| **Merged cells** | Partial (grid tables) | Native (colspan/rowspan) | Partial | Native | Partial | Native | None |
| **Figures + captions** | Native (Figure, Pandoc 3.x) | Native (figure) | Native | Native (figure env) | Native (image+title) | Native (<fig>) | Native (basic) |
| **Code blocks** | Native (CodeBlock) | Native | Native | Native (lstlisting) | Native (source block) | Native (<codeblock>) | Native |
| **Syntax highlighting** | Native (Skylighting) | Native | Native | Via listings/minted | Via CodeRay/Rouge | Via specialization | None |
| **Callouts/admonitions** | Via Div classes | Custom function | Native (5 types) | Via tcolorbox/mdframed | Native (5 types) | Native (<note type=>) | Native (callout) |
| **Footnotes** | Native (Note) | Native (footnote) | Native | Native (\footnote) | Native (footnote:) | Native (<fn>) | None |
| **Cross-references** | Custom filter | Native (@label) | Native | Native (\ref) | Native (<<id>>) | Native (<xref>) | None |
| **Citations** | Native (Cite + CSL) | Native (@key) | Native | Native (biblatex) | None built-in | None built-in | None |
| **Equations** | Native (Math) | Native ($...$) | Native | Native ($$) | Via stem:[] | None | Native (basic) |
| **Charts** | None native | None native | Via R/Python -> image | Via pgfplots/tikz | None | None | None |
| **TOC** | Native (--toc) | Native (outline) | Native (toc: true) | Native (\tableofcontents) | Native (:toc:) | Native (via map) | Native |
| **Headers/footers** | Via template | Native (page) | Via template | Native (\pagestyle) | Via docinfo | Via template | None |
| **Watermarks** | None | Via place() | None | Via draftwatermark | None | None | None |
| **Hyperlinks** | Native (Link) | Native (link) | Native | Native (\href) | Native (link:) | Native (<xref>) | Native |
| **Includes** | None native | Via import | Native | Native (\input) | Native (include::) | Native (conref) | None |

Sources: Pandoc MANUAL.html [1]; Typst docs [2]; Quarto authoring guide [3]; AsciiDoc language reference [4]; DITA spec [5].

---

## Charts and Data Visualization

### The Chart Rendering Decision

Three approaches exist for embedding charts in DOCX/PPTX:

| Approach | Complexity | Visual Quality | Editability in Office | Cross-format | Rust ecosystem |
|----------|-----------|---------------|----------------------|-------------|---------------|
| **Native ChartML + Excel** | Very high | Native | Full (double-click to edit) | DOCX+PPTX only | No mature crate |
| **SVG image embed** | Low | Excellent (vector) | None (static image) | All formats | `plotters` SVGBackend |
| **PNG image embed** | Very low | Good (resolution-dependent) | None | All formats | `plotters` BitMapBackend |

**ChartML complexity breakdown** (from OOXML research [6][7]):
- Requires generating a valid SpreadsheetML workbook (`word/embeddings/embedded1.xlsx`) with sheet data
- Requires ChartML XML (`word/charts/chart1.xml`) with `<c:barChart>`, series definitions, cached values
- Formula references (`<c:f>Sheet1!$B$2:$B$5</c:f>`) must match the embedded workbook
- Cached values (`<c:numCache>`, `<c:strCache>`) must be consistent with workbook data
- Relationship wiring: document -> chart part -> embedded workbook
- Rust has no equivalent to .NET's Open XML SDK for chart generation

### Recommendation: SVG-first via `plotters`

**Use `plotters` (Rust crate) to render charts as SVG, embed as images in all output formats.**

Rationale:
1. **Pure Rust, no external runtime** -- `plotters` renders to SVG via `SVGBackend` and PNG via `BitMapBackend` with zero JS/Node/browser dependency [8]
2. **Single chart pipeline** -- same SVG works in PPTX, DOCX, PDF, and HTML
3. **High visual quality** -- vector SVG scales cleanly for print and screen
4. **Supported chart types in plotters** (verified via Context7 [8]):
   - Line charts, bar charts (via `Rectangle` elements), pie charts (`Pie` element), area charts (`AreaSeries`), scatter plots, histograms, box plots, candlestick/OHLC
5. **Quarto uses the same pattern** -- R/Python generates PNG/SVG images that are embedded; no native ChartML [9]
6. **Office compatibility** -- SVG supported as `image/svg+xml` in Office 2016+ / O365; PNG fallback for older versions
7. **ChartML deferred to v2** -- if editable charts become a hard requirement, add a targeted ChartML module for bar/line only

### Chart Types Needed for slideforge's Domains

| Chart Type | Domain Usage | v1/v2 |
|-----------|-------------|-------|
| **Bar (vertical/horizontal)** | KPIs, finding counts by severity, revenue comparison | v1.0 stretch |
| **Line** | Time-series trends, incident timelines, monthly metrics | v1.0 stretch |
| **Pie / donut** | Resource allocation, incident type breakdown | v1.0 stretch |
| **Area** | Cumulative metrics, capacity planning | v2 |
| **Scatter** | Correlation analysis, performance benchmarks | v2 |
| **Stacked bar** | Multi-category comparison (e.g., incident types per quarter) | v2 |
| **Gauge / speedometer** | SLA compliance, health scores | v2 |
| **Heatmap** | Risk matrices, correlation tables | v2 |

### Mermaid Diagrams

Mermaid rendering requires JavaScript (the layout engine is written in JS). Options for Rust integration:
- **`mermaid-cli`** (Node.js + Puppeteer): most reliable, but adds heavy runtime dependency
- **Kroki service** (remote API): send Mermaid text, receive SVG; requires network access
- **WASM-based Mermaid**: possible but immature in Rust ecosystem

**Recommendation:** Defer Mermaid to v2. For v1, users embed pre-rendered SVG/PNG diagram images as figures. If implemented in v2, use Kroki as optional service or `mermaid-cli` as optional build dependency.

---

## Auto-Generated Document Elements

These elements are synthesized by slideforge from structured slide content without manual authoring. They represent slideforge's key differentiator -- no other DSL-based document tool offers this.

| Auto-generated Element | Source Data | v1/v2 | Generation Logic |
|----------------------|------------|-------|-----------------|
| **Table of contents** | All `report`/`detail` headings + slide titles mapped to sections | v1.0 core | Collect heading hierarchy, emit OOXML TOC field code |
| **Executive summary** | All slides' `takeaway` fields | v1.0 stretch | Collect takeaways, compose into bulleted or prose summary section |
| **Finding register** | `severity_cards` slide data across deck | v1.0 stretch | Collect `@data` from severity_cards slides, emit standardized table |
| **Risk register** | Risk-typed severity_cards or dedicated risk data | v1.0 stretch | Same as finding register with risk-specific columns |
| **Action tracker** | `numbered_actions` slide data across deck | v1.0 stretch | Collect `@data` from numbered_actions slides, emit action table |
| **Table of figures** | All `figure()` calls in report/detail blocks | v2 | Collect figure captions, emit OOXML TOC-of-figures field |
| **Table of tables** | All tables with captions | v2 | Same pattern as table of figures |
| **Glossary** | Terms marked with `{{ term("...", "...") }}` | v2 | Collect term definitions, sort alphabetically, emit definition list |
| **Appendix index** | All `detail` blocks across deck | v2 | Collect detail block titles, emit linked appendix listing |

### Auto-generation Architecture

```
Slide 1 (severity_cards)          Slide 5 (numbered_actions)
  @data: findings                   @data: actions
  takeaway: "5 systems..."          takeaway: "3 immediate..."
       |                                 |
       v                                 v
  +---------Eval Phase Collectors---------+
  |  takeaway_collector                   |
  |  finding_collector                    |
  |  action_collector                     |
  +---------------------------------------+
       |                |              |
       v                v              v
  Exec Summary    Finding Register   Action Tracker
  (auto-section)  (auto-table)       (auto-table)
```

The eval phase walks all slides and collects structured data into typed registers. The document exporter then renders these registers as standard document sections inserted at configurable positions (front-matter, back-matter, or inline).

---

## DSL Syntax Proposals

### Tables in report/detail Blocks

Markdown-style pipe tables with `@for` iteration:

```
report """
  Summary of findings:

  | Finding | Severity | Status | Owner |
  |---------|----------|--------|-------|
  @for f in findings:
    | {{ f.title }} | {{ f.severity }} | {{ f.status }} | {{ f.owner }} |

  Total: {{ findings | len }} findings, {{ findings | where: severity == "critical" | len }} critical.
"""
```

For simple static tables:

```
report """
  | Metric | Target | Actual | Status |
  |--------|--------|--------|--------|
  | MTTR   | < 4hr  | 2.3hr  | Pass   |
  | MTTD   | < 30m  | 23m    | Pass   |
"""
```

OOXML mapping: Parse Markdown pipe table syntax at eval time, emit `<w:tbl>` with `<w:tblGrid>`, `<w:tr>`, `<w:tc>` elements. First row treated as header when `|---|` separator present.

### Figures with Captions

```
report """
  {{ figure("assets/architecture.png",
     caption: "System Architecture -- Post-Incident",
     alt: "Architecture diagram showing web tier isolated from data tier",
     width: "80%") }}
"""
```

With auto-numbering:

```
report """
  As shown in {{ figref("arch-diagram") }}, the web tier was isolated.

  {{ figure("assets/architecture.png",
     label: "arch-diagram",
     caption: "System Architecture -- Post-Incident") }}
"""
```

OOXML mapping: `<w:drawing><wp:inline>` with `<a:blip r:embed="rIdN"/>` for the image, followed by a caption paragraph using SEQ field code for auto-numbering: `<w:fldSimple w:instr="SEQ Figure \* ARABIC"/>`.

### Charts from @data

```
report """
  {{ chart.bar(
     data: kpis.monthly,
     x: "month",
     y: "arr",
     title: "ARR Trend -- 12 Months",
     color: brand.primary,
     width: "100%",
     height: "300px") }}
"""
```

Line chart variant:

```
report """
  {{ chart.line(
     data: incidents.by_month,
     x: "month",
     y: ["count", "resolved"],
     title: "Incident Volume vs Resolution",
     legend: true) }}
"""
```

Pie chart variant:

```
report """
  {{ chart.pie(
     data: incidents.by_type,
     label: "type",
     value: "count",
     title: "Incidents by Category") }}
"""
```

Implementation: At eval time, `chart.*` expressions invoke `plotters` to render SVG to a temp file, then embed as a figure with the specified title as caption.

### Cross-references (v2)

```
report """
  As detailed in {{ ref("impact-assessment") }}, the CI/CD
  pipeline experienced the most significant disruption
  (see {{ figref("arch-diagram") }}, {{ tblref("finding-summary") }}).
"""
```

OOXML mapping: `ref()` emits `<w:fldSimple w:instr="REF impact-assessment \h"/>`. `figref()` emits `Figure <w:fldSimple w:instr="REF arch-diagram \h"/>`. `tblref()` follows the same pattern.

### Footnotes

```
report """
  Detection latency{{ footnote("Measured from first observable
  artifact to SOC-1 alert firing. See NIST SP 800-61r3 S4.2
  for measurement methodology.") }} was within SLA.
"""
```

OOXML mapping: In body, emit `<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="N"/></w:r>`. In `word/footnotes.xml`, emit `<w:footnote w:id="N"><w:p>...<w:footnoteRef/>...text...</w:p></w:footnote>`.

### Callouts / Admonitions

```
report """
  {{ callout("warning", "The remediation timeline exceeded SLA
  by 4 hours. A post-incident review identified staffing gaps
  during the overnight shift as the primary contributing factor.") }}

  {{ callout("note", "All timestamps in this report are UTC.
  Local time conversions are provided in Appendix C.") }}
"""
```

Supported types: `warning`, `note`, `tip`, `important`, `caution` (matching AsciiDoc's five standard types [4]).

OOXML mapping: Composed as a single-row, single-cell table with colored left border (brand.warning for WARNING, brand.info for NOTE, etc.), optional icon image, and styled paragraph content. This pattern produces clean output in Word, PDF, and HTML.

### Code Blocks

```
report """
  The detection rule that fired:

  {{ code("yaml", """
  rules:
    - id: SCR-2024-047
      condition: package.hash != expected.hash
      severity: critical
      action: quarantine
  """) }}
"""
```

OOXML mapping: Paragraphs with monospaced font (`Consolas` or `Courier New`), background shading via `<w:shd w:fill="F5F5F5"/>`, and optional border. Language annotation stored in IR but syntax highlighting deferred to v2.

### Lists (ordered, unordered, nested)

Within report/detail blocks, Markdown list syntax:

```
report """
  Key findings:
  - CI/CD pipeline was primary attack vector
  - Detection occurred within {{ detection_minutes }}-minute SLA
    - SOC-1 alert fired at 02:13 UTC
    - IR team engaged at 02:16 UTC
  - No confirmed data exfiltration

  Recommended actions:
  1. Deploy PAM solution (Q3 2026)
  2. Extend log retention to 365 days
  3. Conduct tabletop exercise for supply chain scenarios
"""
```

OOXML mapping: Each list item becomes a `<w:p>` with `<w:numPr>` referencing a numbering definition in `word/numbering.xml`. Nested items use `<w:ilvl w:val="1"/>` for indent level.

### Inline Formatting

Within report/detail blocks:

```
report """
  The **CI/CD pipeline** was the _most significantly_ affected
  system. The attacker used `curl` to exfiltrate data to
  [a known C2 server](https://example.com/ioc).
"""
```

Markdown-style inline formatting:
- `**bold**` -> `<w:rPr><w:b/>`
- `_italic_` -> `<w:rPr><w:i/>`
- `` `code` `` -> `<w:rPr><w:rFonts w:ascii="Consolas"/>` with optional shading
- `[text](url)` -> `<w:hyperlink r:id="rIdN">`

---

## v1.0 vs. v2 Categorization

### v1.0 Core (19 elements) -- must ship

| Element | Rationale |
|---------|-----------|
| Headings 1-6 | Backbone of document structure; required for TOC |
| Paragraphs | Fundamental prose block |
| Unordered lists | Universal in all report types |
| Ordered lists | Required for procedures, timelines, action items |
| Nested lists (2-3 levels) | Standard in professional slides and documents |
| Simple tables | Critical for findings, metrics, timelines across all domains |
| Figures with captions | Screenshots, diagrams, architecture visuals in every domain |
| Code blocks (plain) | Logs, configs, detection rules in cyber/engineering |
| Bold, italic, inline code | Minimum inline formatting for readable prose |
| Hyperlinks | Standard in all professional documents |
| Table of contents | Expected in any multi-page report |
| Cover page | Required for client-facing deliverables |
| Headers (per-section) | Branding, document title, client name |
| Footers (per-section) | Page numbers, confidentiality notices |
| Page numbers | Universally expected |
| Document metadata | Title, author, date, version |
| Data tables (@for-driven) | Core slideforge value: data-reactive content |
| Computed metrics ({{ }}) | Core slideforge value: data interpolation |
| Conditional content (@if) | Core slideforge value: audience-specific output |

### v1.0 Stretch (10 elements) -- ship if time permits

| Element | Rationale |
|---------|-----------|
| Complex tables (merged cells) | Risk matrices, multi-header tables in audit/consulting |
| Callouts/admonitions | High value in cyber IR (warnings) and engineering (notes) |
| Footnotes | Expected in audit and consulting deliverables |
| Charts (bar, line, pie) | Major differentiator; data-reactive visualizations |
| Confidentiality banner | Required by many enterprise clients |
| Executive summary auto-gen | Key differentiator; synthesizes from slide takeaways |
| Finding register auto-gen | Unique to slideforge; collects across severity_cards |
| Risk register auto-gen | Pairs with finding register |
| Action tracker auto-gen | Collects from numbered_actions |
| Version/revision info | Professional governance metadata |

### v2 (24 elements) -- explicitly deferred

| Element | Rationale |
|---------|-----------|
| Syntax-highlighted code blocks | Complex; v1 uses monospaced plain code |
| Blockquotes | Low priority; styled paragraph approximation works |
| Definition lists | Emulated with tables or styled paragraphs |
| Line blocks | Niche use case |
| Display equations (OMML) | Only engineering/scientific domain |
| Inline equations | Same as above |
| Cross-references | Requires label/ID management infrastructure |
| Citations/bibliography | Academic/research domain primarily |
| Strikethrough | Low priority inline style |
| Superscript/subscript | Low priority inline style |
| Small caps | Low priority inline style |
| Underline | Low priority inline style |
| Highlight/mark | Low priority inline style |
| Appendix sections | Composed from standard headings in v1 |
| Table of figures | Requires figure numbering infrastructure |
| Table of tables | Same as above |
| Glossary | Requires term marking infrastructure |
| Horizontal rule | Low priority structural element |
| Mermaid diagrams | Requires JS runtime or external service |
| Area/scatter/stacked charts | Extended chart types beyond core three |
| Watermarks | Template-specific complexity |
| Approval/signature blocks | Composed from tables/paragraphs in v1 |
| Classification markings | Specialized government/military need |
| Running head | Subsumed by headers in v1 |

### Out of Scope (14 elements) -- not slideforge's job

| Element | Rationale |
|---------|-----------|
| Tracked changes | Collaboration feature, not generation |
| Comments | Review workflow, not final output |
| SmartArt / org charts | Extremely complex OOXML (multiple diagram parts); use images |
| Index (back-of-book) | Academic publishing; not corporate reports |
| Mail merge fields | Enterprise document automation; different tool |
| Macros / VBA | Security risk; not content |
| Digital signatures | PKI infrastructure; separate concern |
| Embedded video/audio | Not relevant to static document output |
| Interactive elements | Web-only; not DOCX/PPTX |
| Content controls / SDTs | Form-filling feature; not generation |
| Custom XML parts | Enterprise integration; not content |
| Revision history table | Version control concern, not document content |
| AI-generated narrative | Non-deterministic; editor-side feature, not build pipeline |
| Template editing UI | PowerPoint/Word concern, not generator output |

---

## OOXML Implementation Notes

### Key Technical Details for the Architect

**1. Document Part Structure (word/*.xml)**

A minimal professional .docx requires these parts:
- `[Content_Types].xml` -- part registry
- `_rels/.rels` -- package relationships
- `word/document.xml` -- main body
- `word/_rels/document.xml.rels` -- body relationships
- `word/styles.xml` -- heading/paragraph/table styles
- `word/numbering.xml` -- list definitions (ordered + unordered + multi-level)
- `word/settings.xml` -- document settings
- `word/fontTable.xml` -- font declarations
- `word/footnotes.xml` -- footnote content (if any footnotes used)
- `word/endnotes.xml` -- endnote content (even if empty, Word expects it)
- `word/header1.xml`, `word/footer1.xml` -- per-section header/footer
- `word/theme/theme1.xml` -- brand theme (shared with PPTX theme model)
- `docProps/core.xml` -- Dublin Core metadata
- `docProps/app.xml` -- application metadata
- `word/media/` -- embedded images

**2. Style System**

slideforge should generate a `styles.xml` with:
- Heading1 through Heading6 styles (for TOC generation)
- Normal paragraph style (body text)
- Code/monospaced character and paragraph styles
- Table styles (at minimum: TableGrid, BrandedTable with banded rows)
- List paragraph styles (ListParagraph, ListBullet, ListNumber)
- FootnoteText, FootnoteReference styles
- Hyperlink character style
- Caption style (for figure/table captions)
- Title, Subtitle styles (for cover page)
- Header, Footer paragraph styles

**3. Numbering System**

`word/numbering.xml` must define abstract numbering definitions for:
- Unordered lists (bullet characters at each level)
- Ordered lists (decimal, then alpha, then roman at successive levels)
- Each list instance in the document references an `<w:abstractNumId>` and gets a unique `<w:numId>`

**4. Footnote Mechanics** [10][11]

- Footnote reference in body: `<w:footnoteReference w:id="N"/>` within a run styled as FootnoteReference
- Footnote content in `word/footnotes.xml`: `<w:footnote w:id="N">` containing paragraphs
- IDs 0 and 1 are reserved for separator and continuation separator footnotes (must exist)
- User footnotes start at ID 2

**5. Header/Footer Mechanics** [12]

- Each section can have up to 6 header/footer parts: default, first-page, even-page for both header and footer
- Linked from `<w:sectPr>` via `<w:headerReference w:type="default" r:id="rIdN"/>`
- Each header/footer is a separate XML part (`word/header1.xml`, etc.)
- Content types must be registered for each header/footer part

**6. Table of Contents**

- Emit as a field code: `<w:fldSimple w:instr="TOC \o \"1-3\" \h \z \u"/>`
- Word updates the TOC when the user opens the document and presses F9
- For PDF export, slideforge must resolve the TOC at build time (walk heading hierarchy, compute page numbers from layout)

**7. Theme Sharing Between PPTX and DOCX**

Both PresentationML and WordprocessingML use the same DrawingML theme format (`<a:theme>`). slideforge can generate a single theme definition from `brand.toml` and use it in both `.pptx/ppt/theme/theme1.xml` and `.docx/word/theme/theme1.xml`. This ensures color and font consistency across slide and document output.

---

## Sources

[1] Pandoc User's Guide -- https://pandoc.org/MANUAL.html (accessed 2026-05-23)
[2] Typst Documentation -- https://typst.app/docs/ (accessed 2026-05-23)
[3] Quarto Authoring Guide -- https://quarto.org/docs/authoring/ (accessed 2026-05-23)
[4] AsciiDoc Language Reference, Admonitions -- https://docs.asciidoctor.org/asciidoc/latest/blocks/admonitions/ (accessed 2026-05-23)
[5] DITA Specification, Topic Types -- https://www.oxygenxml.com/dita/styleguide/Authoring_Concepts/c_Introduction_to_DITA.html (accessed 2026-05-23)
[6] Eric White, "Updating Data for an Embedded Chart in an Open XML WordprocessingML Document" -- https://www.ericwhite.com/blog/updating-data-for-an-embedded-chart-in-an-open-xml-wordprocessingml-document/ (accessed 2026-05-23)
[7] python-pptx Chart Access Analysis -- https://python-pptx.readthedocs.io/en/stable/dev/analysis/cht-access-xlsx.html (accessed 2026-05-23)
[8] plotters Rust crate documentation via Context7 -- https://github.com/plotters-rs/plotters (accessed 2026-05-23)
[9] Quarto PDF reference, chart handling -- https://quarto.org/docs/reference/formats/pdf.html (accessed 2026-05-23)
[10] OOXML Info, Footnotes Part -- https://ooxml.info/docs/11/11.3/11.3.7/ (accessed 2026-05-23)
[11] Microsoft Learn, Footnote Class -- https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.footnote (accessed 2026-05-23)
[12] Microsoft Learn, Structure of a WordprocessingML document -- https://learn.microsoft.com/en-us/office/open-xml/word/structure-of-a-wordprocessingml-document (accessed 2026-05-23)
[13] OfficeOpenXML.com, Anatomy of a .docx -- http://officeopenxml.com/anatomyofOOXML.php (accessed 2026-05-23)
[14] ECMA-376 Part 1, WordprocessingML tables -- https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_Tables_topic_ID0ETOIQ.html (accessed 2026-05-23)
[15] c-rex.net, gridSpan element -- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_gridSpan_topic_ID0EMEPQ.html (accessed 2026-05-23)
[16] c-rex.net, vMerge element -- https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_vMerge_topic_ID0ERQ6R.html (accessed 2026-05-23)
[17] ooxml.dev, Styles documentation -- https://ooxml.dev/docs/styles (accessed 2026-05-23)
[18] c-rex.net, anchor element (floating drawings) -- https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_anchor_topic_ID0EOB1OB.html (accessed 2026-05-23)
[19] c-rex.net, Chart fundamentals -- https://c-rex.net/samples/ooxml/e1/Part1/OOXML_P1_Fundamentals_Chart_topic_ID0ELZLM.html (accessed 2026-05-23)
[20] OOXML Info, Math overview -- https://ooxml.info/docs/22/22.1 (accessed 2026-05-23)
[21] c-rex.net, footnote element -- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_footnote_topic_ID0EKU5U.html (accessed 2026-05-23)
[22] c-rex.net, fldSimple element -- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_fldSimple_topic_ID0EPAV1.html (accessed 2026-05-23)
[23] Quarto Blog, Typst books -- https://quarto.org/docs/blog/posts/2026-03-31-typst-books-and-more/ (accessed 2026-05-23)
[24] Notion Block Model -- https://developers.notion.com/reference/block (accessed 2026-05-23)
[25] Pandoc Filters documentation -- https://pandoc.org/filters.html (accessed 2026-05-23)
[26] NIST SP 800-61r3 -- https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-61r3.pdf (accessed 2026-05-23)
[27] docx-rs Rust crate -- https://github.com/PoiScript/docx-rs (accessed 2026-05-23)
[28] rdocx Rust crate -- https://lib.rs/crates/rdocx (accessed 2026-05-23)
[29] Typst elembic package -- https://typst.app/universe/package/elembic/ (accessed 2026-05-23)
[30] DITA content writing guide -- https://componize.com/blog/zero-to-hero-dita-xml-content-writing/ (accessed 2026-05-23)
[31] Displayr automated reporting -- https://www.displayr.com/automated-powerpoint-reporting/ (accessed 2026-05-23)
[32] LLeMental AI templates -- https://llemental.com/posts/ai-presentation-generator-turn-powerpoint-word-into-ai-templates (accessed 2026-05-23)

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 2 | Deep research on document content element taxonomy across 7 domains; cross-tool comparison of Pandoc/Typst/Quarto/LaTeX/AsciiDoc/RST/DITA content models |
| Perplexity perplexity_reason | 2 | Chart rendering decision analysis (ChartML vs SVG vs PNG); v1.0 vs v2 element categorization with rationale |
| Perplexity perplexity_ask | 3 | Pandoc AST types + Typst elements + AsciiDoc admonitions; auto-generated document elements across tools; DITA/Notion/Confluence comparison |
| Perplexity perplexity_search | 1 | OOXML footnotes/endnotes/headers/footers XML structure |
| Context7 | 1 resolve + 1 query | plotters Rust crate -- SVG backend, chart types, API examples |
| Tavily tavily_research | 1 | OOXML WordprocessingML complete content element analysis (tables, images, charts, equations, styles, fields) |
| Training data | 3 areas | General DSL syntax design patterns (low confidence, validated against tool docs); OOXML element ordering rules (validated against ECMA-376 references); Markdown syntax conventions (well-established, low risk) |

**Total MCP tool calls:** 11
**Training data reliance:** low -- all element categorizations, OOXML mappings, and tool comparisons verified against web sources; DSL syntax proposals are design work informed by prior writing-register-research.md and existing dsl-spec.md conventions.
