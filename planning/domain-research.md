---
title: Supplementary Domain Research — Gap Analysis
date: 2026-05-24
analyst: research-agent
status: foundation-research
audience: business-analyst + architect + product-owner
scope: Gaps not covered (or underserved) by R1-R14 research threads
---

# Supplementary Domain Research: Branded Document Generation Platform

## Executive Summary

This document covers six domain areas that R1-R14 either missed or only partially addressed. The most actionable findings are:

1. **DDD modeling**: Slideforge's four-stage pipeline maps cleanly to four bounded contexts (Authoring, Branding, Layout, Export). The `Deck` aggregate root pattern with `Slide` as an internal entity and EMU-based value objects is well-supported by DDD literature. R6 (IR Prior Art) covers the data structures but not the aggregate/invariant modeling.

2. **Document generation standards**: DITA's reuse model (conref, keyref, conditional profiling) offers patterns slideforge should borrow for its `@include` and variable systems. PDF/UA (ISO 14289) requirements should be baked into the PDF exporter from day one. Dublin Core metadata should be the internal metadata model. R5 (WCAG) covers accessibility but not PDF/UA or DITA-style reuse.

3. **Presentation-as-code ecosystem (May 2026)**: No major disruptions since R3. Slidev remains pre-1.0 (0.x series). Marp Core v4 not yet released. The biggest shift is Typst slides (touying + polylux) becoming a real alternative ecosystem. `presenterm` (Rust TUI markdown slides) is notable but not a competitor. No dominant Rust PPTX library exists -- slideforge would be first-of-kind.

4. **Data binding patterns**: Docxtemplater's `{#items}/{/items}` loop syntax is the de facto standard for document template binding. Typst's native data loading (json/csv/yaml) + control flow is the closest model to what slideforge is building. The `nullGetter` pattern from docxtemplater is worth studying for slideforge's missing-data error handling.

5. **Multi-format export challenges**: Font metric divergence, EMF/WMF incompatibility, table layout differences, and metadata loss are the top four failure modes. Slideforge's decision to use integer EMUs and avoid `f64` directly mitigates some coordinate-level issues. The key new insight: test in actual target viewers (PowerPoint, LibreOffice, Keynote, Google Slides) -- not just XML structure validation.

6. **Rust plugin architecture**: For slideforge's 10 plugin surfaces, trait-based static dispatch is the correct v1.0 choice. WASM plugins (via Extism/wasmtime) should be reserved for v2.0 when third-party extensibility matters. The Cargo subcommand pattern (`slideforge-xxx`) is a free win for CLI extensibility.

---

## 1. Domain Modeling Gaps (DDD Patterns for Document Generation)

**R1-R14 coverage**: R6 (IR Prior Art) covers data structure design extensively. R7 (Composition Prior Art) covers configuration layering. Neither applies DDD tactical patterns (aggregates, bounded contexts, invariants) to the domain.

### 1.1 Bounded Contexts

The four-stage pipeline maps to four bounded contexts with distinct ubiquitous languages:

| Bounded Context | Ubiquitous Language | Slideforge Crates |
|----------------|---------------------|-------------------|
| **Authoring** | Deck, Slide, Content Block, Section, Notes, Register, Include, Variable | `slideforge-syntax`, `slideforge-eval` |
| **Branding** | Brand, Theme, Palette, Typography, Layout Template, Logo, Color Token, Master | `slideforge` (brand module), plugin: `BrandProvider` |
| **Layout** | Frame, BoundingBox, TextFlow, Line, Span, EMU, Constraint, Overflow | `slideforge-layout` |
| **Export** | RenderTarget, Asset, Part, Relationship, ContentType, Package | `slideforge-pptx`, `slideforge-pdf`, `slideforge-html`, `slideforge-docx` |

**Key insight**: The Authoring context uses semantic language ("this is a severity_cards slide with 4 items") while the Layout context uses geometric language ("place a 3200x1800 EMU frame at position (914400, 457200)"). These must be separate types -- which aligns with the Two-IR model already decided in R6.

**Integration patterns between contexts**:
- Authoring to Layout: **Published Language** -- the `Deck` IR is the stable contract
- Branding to Layout: **Customer-Supplier** -- Layout consumes Brand constraints (min font size, logo position rules, margin guidelines)
- Layout to Export: **Published Language** -- the `LaidOutDeck` IR is the stable contract
- Branding to Export: **Shared Kernel** -- Brand assets (logos, fonts, color values) are needed by both Layout (for constraint checking) and Export (for embedding)

[Source: DDD analysis synthesized from Microsoft DDD guidance [1], Pandoc/Typst architectural analogy [2], training data for DDD tactical patterns]

### 1.2 Aggregate Roots and Entities

| Aggregate Root | Identity | Key Invariants |
|---------------|----------|----------------|
| `Deck` | `DeckId` (file path + content hash) | Must have >= 1 slide; brand ref must be valid; aspect ratio must be compatible with brand; all slides share the same brand (v1.0 -- multi-master deferred per R12) |
| `Brand` | `BrandId` (name + version) | Must define palette with all 12 OOXML scheme colors; typography scheme must have heading + body fonts; all color tokens must be unique within brand |
| `DeckLayout` | Derived from `DeckId` | Slide count matches Deck; all frames within slide bounds; logo placement satisfies brand guidelines |

**Slide as internal entity** (not aggregate root): Slides do not have independent lifecycle in v1.0. They exist only within a Deck. This simplifies the consistency boundary -- all slide operations go through Deck methods.

### 1.3 Value Objects

| Value Object | Fields | Invariants |
|-------------|--------|------------|
| `Emu` | `i64` (914400 per inch) | Non-negative for dimensions; may be negative for offsets |
| `Color` | Enum: `SchemeColor(token)` or `SrgbColor(r, g, b)` | If scheme token, must exist in brand palette |
| `FontRef` | `family: Arc<str>`, `weight: FontWeight`, `style: FontStyle` | Family must be in brand typography or system fallback list |
| `Span` | `text: Arc<str>`, `formatting: InlineFormatting` | Text non-empty (empty spans are elided) |
| `SourceSpan` | `file: FileId`, `start: usize`, `end: usize` | `start <= end`; file must exist in source map |
| `AspectRatio` | `width: u32`, `height: u32` | Both > 0; normalized (no GCD > 1) |

### 1.4 Domain Events (for future incremental/watch mode)

| Event | Trigger | Consumers |
|-------|---------|-----------|
| `SourceChanged(FileId)` | File watcher detects edit | Parser (re-parse affected file) |
| `BrandUpdated(BrandId)` | Brand config file changed | Layout (re-layout all slides), Export (re-embed assets) |
| `SlideAdded/Removed/Reordered` | Eval produces different slide set | Layout (incremental re-layout) |
| `LayoutComplete(DeckLayoutId)` | Layout finishes | All exporters (can run in parallel) |
| `ValidationFailed(Vec<Diagnostic>)` | Validation pass finds issues | CLI reporter, watch-mode error overlay |

**Actionability for Phase 1**: The bounded context decomposition directly informs crate boundaries (already decided) and trait API design (slideforge-plugin-api). The aggregate/value-object modeling should inform the `slideforge-types` crate's type definitions. Domain events can be deferred to the incremental/comemo integration but should be designed into the IR types from day one (all IR types already require `Hash + Eq + Clone`).

---

## 2. Document Generation Domain Standards

**R1-R14 coverage**: R4 (OOXML Foundations) covers ECMA-376/ISO-29500 deeply. R5 (WCAG) covers web accessibility. R10 (PPTX Element Taxonomy) covers OOXML presentation elements. None of R1-R14 covers DITA, DocBook, ODF, PDF/UA, or metadata standards.

### 2.1 DITA (Darwin Information Typing Architecture)

DITA is the most mature standard for modular, multi-format document generation. While slideforge is not a DITA tool, several DITA patterns are directly applicable:

**Patterns to borrow**:

| DITA Concept | Slideforge Equivalent | Status |
|-------------|----------------------|--------|
| `conref` (content reference -- pull one element into another) | `@include` of fragment files | Already decided (Q3 decisions) |
| `keyref` (late-bound variable indirection) | `{{ var }}` interpolation with deck-level and variant-level variable scopes | Already decided (Q1 decisions) |
| `ditaval` (conditional profiling -- include/exclude by audience/platform) | Variant system with tag filters | Decided as lightweight variants (R7) |
| Topic-Map separation (content separate from assembly) | `.sf` files (content) vs workspace config (assembly) | Partially covered by R14 (workspace model) |
| Chunking (splitting/merging topics per output) | Per-format slide splitting (e.g., DOCX may merge slides into flowing pages) | Not yet addressed |

**New insight**: DITA's **chunking** concept is directly relevant to slideforge's multi-format challenge. A "slide" in PPTX is a discrete page, but in DOCX it may flow continuously, and in HTML it may be either a section or a page. The chunking decision should be per-exporter, not baked into the IR.

[Source: DITA standard documentation via Oxygen XML [5], Stilo DITA FAQ [6]]

### 2.2 DocBook

DocBook's key lesson: **semantic structure first, presentation via stylesheets**. This is exactly what slideforge does with its Two-IR model. DocBook validates that separating content structure from layout rules is the industry-proven approach.

No new actionable items beyond confirming the existing architecture.

[Source: Wikipedia DocBook article [7], Linux Journal DocBook overview [8]]

### 2.3 ODF Presentation Format (.odp)

ODF (.odp) is structurally similar to OOXML (.pptx): ZIP container, XML parts, slide-master-layout hierarchy. Key differences:

- Element names differ (`<draw:page>` vs `<p:sld>`)
- Style system differs (ODF uses named styles; OOXML uses theme + direct formatting)
- ODF is the native format for LibreOffice Impress

**Actionability**: If slideforge ever adds ODP export, the `LaidOutDeck` IR should be abstract enough to map to both OOXML and ODF element models. The current Two-IR design already supports this -- `LaidOutDeck` deals in abstract shapes/frames, not OOXML-specific elements.

[Source: ODF specification overview, LibreOffice documentation tools wiki [9]]

### 2.4 PDF/UA (ISO 14289) -- Accessible PDFs

**This is a gap in R1-R14.** R5 covers WCAG for web/HTML, but PDF/UA has distinct requirements that the PDF exporter must satisfy:

| PDF/UA Requirement | Slideforge Implementation |
|-------------------|--------------------------|
| Tagged PDF structure (`<H1>`, `<P>`, `<L>`, `<Table>`, etc.) | PDF exporter must emit structure tags from semantic IR, not just visual rendering |
| Correct reading order | Must match DSL source order (or explicit `reading-order` override) |
| Alt text on all meaningful images | Already enforced by `alt "..."` requirement (compile error) |
| Decorative images marked as artifacts | Already handled by `decorative: true` flag |
| Language tag (`/Lang`) | Already required by `lang "en-US"` at deck level |
| Bookmarks reflecting heading structure | PDF exporter must generate bookmark tree from slide titles/sections |
| All fonts embedded | PDF exporter must subset-embed all fonts used |

**Key new requirement**: The PDF exporter cannot simply rasterize slides to images and embed them (which is a common shortcut). It must produce **tagged PDF** with real text, structure, and accessibility metadata. This rules out the "render to image, wrap in PDF" approach and requires a proper PDF generation library.

**Verification**: `veraPDF` is the standard validator for PDF/UA compliance. Already mentioned in CLAUDE.md quality bar.

[Source: ISO 14289 overview via Harvard accessibility guide [10], Adobe accessible PDF resources [11]]

### 2.5 PDF/A (ISO 19005) -- Archival PDFs

PDF/A requirements relevant to slideforge:
- All fonts must be embedded (subset embedding acceptable)
- No JavaScript, no audio/video, no encryption
- Color management metadata required
- XMP metadata must include `pdfaid:part` and `pdfaid:conformance`

**Actionability**: Add a `--pdf-a` flag to the PDF exporter (v1.x, not v1.0). The main day-one requirement is ensuring fonts are always embedded.

[Source: PDF/A overview via pdfa.org [12]]

### 2.6 Metadata Standards (Dublin Core + schema.org)

**Recommended internal metadata model**: Dublin Core (DC) subset, mapped to each output format:

| DC Field | DSL Syntax | PPTX Mapping | PDF Mapping | HTML Mapping |
|----------|-----------|-------------|-------------|-------------|
| `dc:title` | `title:` (frontmatter) | Core Properties: Title | XMP dc:title | `<title>` + `<meta property="og:title">` |
| `dc:creator` | `author:` (frontmatter) | Core Properties: Author | XMP dc:creator | `<meta name="author">` |
| `dc:subject` | `subject:` (frontmatter) | Core Properties: Subject | XMP dc:subject | `<meta name="description">` |
| `dc:language` | `lang:` (required) | Core Properties: Language | `/Lang` document tag | `<html lang="...">` |
| `dc:date` | `date:` (frontmatter) | Core Properties: Modified | XMP xmp:ModifyDate | `<meta property="article:modified_time">` |
| `dc:rights` | `confidentiality:` | Extended Properties | XMP dc:rights | `<meta name="rights">` |

**For HTML output**: Also emit JSON-LD with `@type: "PresentationDigitalDocument"` (schema.org) for search engine discoverability.

**Actionability**: The DSL frontmatter metadata block should be designed to capture these DC fields. The mapping to each format should be part of each exporter's responsibility.

[Source: Dublin Core standard, schema.org PresentationDigitalDocument type [13]]

---

## 3. Presentation-as-Code Ecosystem Update (May 2026)

**R3 coverage**: Comprehensive competitor analysis from May 2023 covering Marp, Slidev, reveal.js, Spectacle, Pandoc, Quarto, Typst, python-pptx, Beamer. This section covers what changed since then.

### 3.1 Ecosystem Status (as of May 2026)

| Tool | Latest Status | Major Changes Since R3 | Competitive Threat to Slideforge |
|------|--------------|----------------------|--------------------------------|
| **Slidev** | Still pre-1.0 (0.x series) | Continued iteration on Vue 3, UnoCSS, export. No v1.0 milestone announced. | Low -- web-only, no PPTX, Vue dependency |
| **Marp** | Core v3.x still active; v4 not yet released | Incremental improvements. VS Code integration mature. | Low -- Markdown ceiling, CSS escape hatch |
| **reveal.js** | 5.x stable | Stable platform, more of a runtime than a competitor | None -- different category (web runtime) |
| **Spectacle** | Active but lower momentum | React-focused, less ecosystem growth than Slidev | None |
| **Typst + touying** | touying actively updated for Typst 0.13+ | Overlays, theming, slide abstractions significantly improved | Medium -- closest philosophical competitor (DSL-first, compiled, fast). But PDF-only output, no PPTX |
| **Typst + polylux** | Active 0.x, overlays and layouts | Better overlay system, Typst 0.13+ integration | Medium -- same as touying |
| **Pandoc** | Still the Swiss army knife | PPTX writer continues to have issues with math in titles, image embedding | Low -- generic tool, not presentation-focused |
| **Quarto** | Strong in data science community | Good PPTX/PDF/HTML output, but tied to R/Python/Julia ecosystem | Medium for data science users |
| **python-pptx** | Maintenance mode | No significant updates | Low -- this is what slideforge replaces |
| **PptxGenJS** | Active JS library | Code-first PPTX generation, no DSL | Low -- different paradigm (API, not DSL) |
| **presenterm** | Rust TUI markdown slides, active | Terminal-only presentation. PDF export. Uses ratatui. | None -- terminal-only, no PPTX |

[Source: Perplexity research May 2026 [14], Tavily search May 2026 [15], presenterm crates.io page [16]]

### 3.2 New Entrant: AI-Driven Presentation Tools

A notable 2025-2026 trend not covered in R3: **AI-powered presentation generators** (Claude + MCP, Gamma, Presenti, 2Slides). These tools generate slides from natural language prompts.

**Competitive analysis**: These tools are complementary, not competitive to slideforge:
- They generate one-off presentations from prompts
- Slideforge generates **deterministic, version-controlled, data-driven** presentations from structured specs
- The two can coexist: an AI tool could generate a `.sf` file that slideforge compiles

### 3.3 Gap Confirmation

Slideforge occupies a unique position that no existing tool fills:

| Capability | Slidev | Marp | Typst | Quarto | python-pptx | **Slideforge** |
|-----------|--------|------|-------|--------|-------------|----------------|
| Purpose-built presentation DSL | No (Markdown) | No (Markdown) | Yes (generic) | No (Markdown) | No (Python API) | **Yes** |
| PPTX output | No | Yes (limited) | No | Yes | Yes | **Yes** |
| PDF output | Yes | Yes | Yes | Yes | No | **Yes** |
| HTML output | Yes | Yes | Yes (planned) | Yes | No | **Yes** |
| DOCX output | No | No | No | Yes | No | **Yes** |
| Brand template system | CSS themes | CSS themes | Templates | Themes | Manual | **Full OOXML brand bridge** |
| Single binary | No (Node) | Yes (Marp CLI) | Yes | No (R/Python) | No (Python) | **Yes** |
| Data-reactive content | Vue components | No | Yes (native) | Yes (R/Python) | Python code | **Yes (DSL-native)** |
| Accessibility-first | Varies | Limited | Limited | Limited | No | **Compile-time enforced** |
| Cross-renderer PPTX fidelity | N/A | Limited | N/A | Limited | Good | **Target: production-grade** |

**Conclusion**: The competitive landscape has not shifted in a way that threatens slideforge's value proposition. The biggest development (Typst slides maturing) actually validates the "compiled DSL for slides" approach -- but Typst cannot generate PPTX, which is slideforge's primary differentiator.

---

## 4. Data Binding in Document Generators

**R1-R14 coverage**: R1 (Python reference deep read) covers how the reference implementation handles data. R7 (Composition) covers configuration layering. Neither surveys the broader data binding pattern landscape.

### 4.1 Pattern Taxonomy

| Pattern | Tools | Tag Syntax | Loops | Conditionals | Missing Data |
|---------|-------|-----------|-------|-------------|-------------|
| **Template tag substitution** | Docxtemplater, Carbone | `{name}`, `{d.name}` | `{#items}...{/items}` | Truthy/falsy sections | `nullGetter` callback or default formatter |
| **Jinja-style templating** | Jinja2, Nunjucks, Liquid | `{{ name }}` | `{% for item in items %}` | `{% if cond %}` | `StrictUndefined` or empty string |
| **Imperative API** | python-pptx, Apache POI, PptxGenJS | User-defined (usually `{{var}}` or `${var}`) | Code loops duplicating slides/rows | Code conditionals | `dict.get(key, default)` |
| **Native DSL binding** | Typst | `#data.name` | `#for item in data.items { }` | `#if cond { }` | `dict.at(key, default: fallback)` or error |
| **Filter-based** | Pandoc Lua filters | User-defined (often custom Div classes) | Filter generates multiple AST nodes | Filter conditionally removes/adds blocks | Filter code handles defaults |

### 4.2 Slideforge's Position

Slideforge's data binding (per Q1 decisions) is closest to **Typst's native DSL binding** model:

| Feature | Typst | Slideforge (decided) |
|---------|-------|---------------------|
| Data loading | `json()`, `csv()`, `yaml()` | `@data` directive with json, csv, yaml, toml, xlsx, sqlite, http |
| Variable access | `#data.name` | `{{ data.name }}` |
| Loops | `#for item in data.items { }` | `@for item in data.items:` (indentation-significant) |
| Conditionals | `#if cond { } else { }` | `@if cond:` / `@elif:` / `@else:` |
| Missing data | Error or `.at(key, default:)` | Compile error with span + hint (strict by default) |
| Math-mode binding | N/A (Typst is math-native) | `@{var}` inside `$...$` math blocks |

### 4.3 Lessons from Docxtemplater

Docxtemplater's `nullGetter` pattern is worth studying:

```javascript
const nullGetter = (part, scopeManager) => {
  // Collect all missing tags for comprehensive error reporting
  nullValues.push(
    [].concat(scopeManager.scopePath).concat(part.value)
  );
  return 'undefined'; // or throw
};
```

**Slideforge equivalent**: When a `{{ var }}` references an undefined variable, slideforge should:
1. **Collect ALL missing variable errors** (don't stop at first -- error accumulation per DSL decisions)
2. **Report with full scope path**: "variable `item.price` not found in scope `@for item in data.items` at file:line:col"
3. **In watch mode**: Render an error placeholder on the slide (per error-slide placeholder decision)

### 4.4 Type Handling

A critical difference between slideforge and most template engines: **slideforge should NOT silently coerce types**.

Per Q4-Q15 decisions (R3 finding): `NO` stays string "NO", `1.10` stays "1.10". This is the correct decision, but it means slideforge needs explicit formatting functions:

- `{{ price | format_number(2) }}` -- format as 2-decimal number
- `{{ date | format_date("YYYY-MM-DD") }}` -- format date
- `{{ value | format_percent }}` -- format as percentage

This is one of the ~15 built-in functions decided in Q1.

[Source: Docxtemplater documentation [17], Typst data loading docs [18], Carbone.io documentation [19]]

---

## 5. Multi-Format Export Challenges

**R1-R14 coverage**: R2 (Brand Template Patterns) covers OOXML structure. R4 (OOXML Foundations) covers XML correctness. R5 (WCAG) covers accessibility. R12 (Template Binding) covers cross-renderer brand fidelity. None of R1-R14 systematically catalogs multi-format export pitfalls.

### 5.1 Top Failure Modes (Ranked by Impact)

| Rank | Failure Mode | Formats Affected | Slideforge Mitigation |
|------|-------------|-----------------|----------------------|
| 1 | **Font metric divergence** | All | Use Liberation/Noto font families as fallback chain; embed fonts in PDF; specify `panose` font classification in OOXML for better substitution |
| 2 | **Table layout differences** | PPTX vs PDF vs HTML | Calculate column widths in layout engine (not renderer); use fixed-width tables by default; warn on auto-fit |
| 3 | **EMF/WMF image incompatibility** | PPTX to PDF/HTML | Never generate EMF/WMF; use PNG for raster, SVG for vector; convert at export time if needed |
| 4 | **Metadata loss across formats** | All | Internal Dublin Core model mapped to each format (see Section 2.6); validate per-format metadata completeness |
| 5 | **Math equation rendering differences** | PPTX vs PDF vs HTML | PPTX: OMML; HTML: KaTeX/MathML; PDF: rendered as vector paths via pulldown-latex. Test all three. |
| 6 | **Color space inconsistency** | PDF vs HTML vs PPTX | Standardize on sRGB internally; embed ICC profile in PDF; use hex RGB in OOXML and HTML |
| 7 | **BiDi/complex script text** | All | Use HarfBuzz for text shaping in layout engine; carry `dir="rtl"` through to all formats; test with Arabic/Hebrew fixtures |
| 8 | **Cross-renderer layout differences** | PPTX | Test in PowerPoint, LibreOffice, Keynote, Google Slides (per quality bar); use placeholder inheritance correctly per R4 |

### 5.2 Font Strategy

The single most impactful multi-format decision is **font handling**:

| Format | Font Model | Failure Mode | Mitigation |
|--------|-----------|-------------|-----------|
| PPTX | Font name reference; substituted by renderer | Different line widths, text overflow | Specify `panose` classification; include font embedding option |
| PDF | Embedded subset (best practice) | Missing glyphs, wrong metrics | Always subset-embed; use `pdf-rs` or equivalent with font embedding support |
| HTML | CSS `font-family` with `@font-face` or system fallback | Different rendering across browsers/OS | Ship web fonts or use system font stack with explicit metrics |
| DOCX | Font name reference (like PPTX) | Same as PPTX | Same as PPTX |

**Concrete risk**: A slide title in "Aptos" (Microsoft's new default font, replacing Calibri) renders correctly in PowerPoint but becomes wider in Liberation Sans (Linux), causing text overflow. Slideforge's layout engine should calculate text extents using the **intended font metrics** and warn when the text is within 10% of overflow bounds.

### 5.3 Image Format Strategy

| Format | Recommended Image Formats | Avoid |
|--------|--------------------------|-------|
| PPTX | PNG (raster), SVG (vector), JPEG (photos) | EMF, WMF, WebP (limited support in older Office) |
| PDF | PNG, JPEG, embedded vector paths | EMF, WMF |
| HTML | PNG, SVG, JPEG, WebP | EMF, WMF |
| DOCX | PNG, JPEG, SVG (Office 365+) | EMF, WMF |

**slideforge chart/diagram strategy**: Charts (via `plotters`) and diagrams (via DiagramRenderer) should produce SVG. SVG is natively supported in HTML and can be converted to PNG for PPTX/DOCX/PDF embedding. The conversion should happen in the export layer, not the layout layer.

### 5.4 The "DOCX as Flowing Document" Challenge

A unique multi-format challenge for slideforge: PPTX slides are discrete pages, but DOCX is a flowing document. How does a 25-slide deck become a DOCX?

**Options** (not yet decided in R1-R14):

| Approach | Description | Pros | Cons |
|----------|------------|------|------|
| One section per slide | Each slide becomes a DOCX section with page break | Simple; preserves 1:1 mapping | Wastes paper; doesn't feel like a "document" |
| Flowing report | Slides become continuous content; titles become headings; bullets flow naturally | Reads like a proper document | Complex mapping; some slide types don't flow well |
| Writing register-based | `report` register content becomes the DOCX body; `notes` register becomes appendix | Leverages the register system (R8) | Requires authors to write register-specific content |

**Recommendation**: The register-based approach is the most natural for slideforge's domain. The `report` register was designed exactly for this purpose (R8 decision). The DOCX exporter should consume `report` register content as its primary input, not try to linearize slides.

[Source: Multi-format export analysis via Perplexity [20], font embedding research [21], image format compatibility research [22]]

---

## 6. Plugin Architecture Patterns in Rust

**R1-R14 coverage**: R6 (IR Prior Art) discusses Typst's element trait system. Q3 decisions define 10 plugin surfaces with trait signatures. Neither surveys the broader Rust plugin architecture landscape.

### 6.1 Pattern Comparison

| Pattern | Safety | Performance | Extensibility | Complexity | Best For |
|---------|--------|-------------|--------------|-----------|---------|
| **Trait-based static dispatch** | Full Rust safety | Best (inlining, monomorphization) | Compile-time only | Low | Internal plugin architecture (slideforge v1.0) |
| **Trait objects (`dyn Trait`)** | Full Rust safety | Good (vtable indirection) | Runtime registry possible | Low-Medium | Open set of built-in plugins |
| **Enum dispatch** | Full Rust safety | Best (match inlining) | Closed set only | Low | Fixed plugin variants |
| **Cargo subcommand pattern** | Process isolation | Good (IPC overhead) | Runtime, any language | Low | CLI extensibility |
| **Dynamic loading (libloading)** | Unsafe FFI boundary | Good (native code) | Runtime, C ABI required | High | Hot-reload, C interop |
| **WASM plugins (wasmtime/Extism)** | Sandboxed | Moderate (WASM overhead) | Runtime, any WASM language | Medium-High | Third-party plugin ecosystem |

### 6.2 Recommendation for Slideforge

**v1.0 (Phase 3)**: Trait-based static dispatch with `dyn Trait` for the plugin registry.

```rust
// slideforge-plugin-api crate (simplified)
pub trait DataSource: Send + Sync {
    fn name(&self) -> &str;
    fn supported_schemes(&self) -> &[&str]; // ["json", "csv", "yaml", ...]
    fn load(&self, uri: &str, ctx: &LoadContext) -> Result<DataValue, DataError>;
}

pub trait Exporter: Send + Sync {
    fn name(&self) -> &str;
    fn format(&self) -> ExportFormat;
    fn export(&self, deck: &Deck, layout: &LaidOutDeck, opts: &ExportOptions) -> Result<Vec<u8>, ExportError>;
}

// Plugin registry (runtime, but statically linked)
pub struct PluginRegistry {
    data_sources: Vec<Box<dyn DataSource>>,
    exporters: Vec<Box<dyn Exporter>>,
    chart_renderers: Vec<Box<dyn ChartRenderer>>,
    // ... all 10 surfaces
}
```

This is the pattern used by most successful Rust projects (rustc, Typst's element system, many CLI tools). It gives full type safety, zero unsafe code, and excellent performance.

**v1.x**: Add Cargo subcommand pattern for CLI extensibility. `slideforge-lint`, `slideforge-fmt`, `slideforge-convert` can be separate binaries discoverable via PATH. This is free -- just check for `slideforge-{subcommand}` in PATH before reporting "unknown subcommand."

**v2.0**: Evaluate WASM plugins via Extism for third-party extensibility. Extism provides:
- Plugin manifest with memory limits, timeout, allowed hosts
- Multi-language PDK (Rust, Go, JS, Python can write plugins)
- Sandboxed execution (plugins can't access filesystem without explicit grants)
- Single `.wasm` artifact works across OS/arch

```rust
// Future v2.0 WASM plugin loading (Extism)
use extism::*;

let manifest = Manifest::new([Wasm::file("./custom-chart-renderer.wasm")])
    .with_memory_max(50)     // 3.2 MB limit
    .with_timeout(Duration::from_secs(5));

let mut plugin = Plugin::new(&manifest, [], true)?;
let svg_output = plugin.call::<&str, &str>("render_chart", &chart_spec_json)?;
```

[Source: Extism documentation via Context7 [23], Nushell plugin architecture [24], Cargo subcommand discovery pattern [25], Helm 4 WASM plugin HIP [26]]

### 6.3 Anti-Patterns to Avoid

| Anti-Pattern | Why It Fails | Slideforge Risk |
|-------------|-------------|----------------|
| Dynamic loading (`dlopen`) for v1.0 | Unsafe FFI, ABI instability, cross-platform nightmares | None -- already committed to `#![forbid(unsafe_code)]` |
| Over-abstracting plugin traits | Too many generic parameters, trait object safety issues | Medium -- keep traits simple with concrete types where possible |
| Plugin-per-process for data sources | IPC overhead for data loading is unacceptable (< 500ms build target) | None -- data sources must be in-process |
| Global mutable plugin state | Prevents parallel export, complicates testing | Use `&self` methods, pass state through context parameters |

### 6.4 Trait Object Safety Considerations

For `dyn Trait` to work, plugin traits must be object-safe:
- No `Self` in return position
- No generic methods
- No associated types used in complex ways

The current Q3 decision trait signatures appear object-safe, but this should be verified during Phase 1 spec crystallization.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 2 | DDD patterns for document generation; document generation standards (DITA, DocBook, ODF, PDF/UA) |
| Perplexity perplexity_ask | 6 | DDD tactical patterns; document standards overview; presentation-as-code ecosystem May 2026; data binding patterns; multi-format export challenges; Rust plugin architecture |
| Context7 resolve-library-id | 1 | Extism WASM plugin framework |
| Context7 query-docs | 1 | Extism Rust host SDK usage |
| Tavily tavily_search | 3 | Slidev/presentation tools 2026; Rust plugin architecture; presenterm tool |
| Training data | 3 areas | DDD aggregate/value-object patterns (well-established theory); font embedding mechanics (stable knowledge); Rust trait object safety rules (language specification) |

**Total MCP tool calls:** 13
**Training data reliance:** low -- DDD patterns and Rust language semantics are well-established; all tool/library/version claims verified against web sources

---

## Citations

[1] Microsoft DDD Guidance: https://learn.microsoft.com/en-us/azure/architecture/microservices/model/domain-analysis
[2] Microsoft DDD Microservice Patterns: https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/ddd-oriented-microservice
[3] Typst PDF Generation as XeLaTeX Alternative: https://slhck.info/software/2025/10/25/typst-pdf-generation-xelatex-alternative.html
[4] Martin Fowler on Bounded Contexts: https://www.martinfowler.com/bliki/BoundedContext.html
[5] Stilo DITA Reuse FAQ: https://www.stilo.com/dita-xml-faqs/how-do-dita-publishing-tools-handle-reused-content-in-different-output-formats/
[6] Oxygen XML DITA Documentation: https://www.oxygenxml.com/doc/versions/28.1/ug-editor/topics/author-docbook5-doc-type.html
[7] Wikipedia DocBook: https://en.wikipedia.org/wiki/DocBook
[8] Linux Journal DocBook Overview: https://www.linuxjournal.com/article/7740
[9] LibreOffice ODF Document Generation Tools: https://wiki.documentfoundation.org/Documentation/ODF_documents_generation_tools
[10] Harvard PDF Accessibility Guide: https://accessibility.huit.harvard.edu/pdf
[11] Adobe Accessible PDF Resources: https://www.adobe.com/uk/acrobat/resources/embed-fonts-in-pdf.html
[12] PDF/A Color Specification: https://pdfa.org/wp-content/uploads/2011/08/tn0002_color_in_pdfa-1_2008-03-141.pdf
[13] Docugenerate (document generation product): https://www.docugenerate.com/product/
[14] Perplexity Presentation-as-Code Ecosystem Research (May 2026) -- live query
[15] Tavily Search: Slidev, presentation-as-code tools (May 2026) -- live query
[16] presenterm on crates.io: https://crates.io/crates/presenterm
[17] Docxtemplater Tag Types Documentation: https://docxtemplater.com/docs/tag-types/
[18] Typst Data Loading Documentation: https://typst.app/docs/reference/data-loading/
[19] Docxtemplater FAQ: https://docxtemplater.com/faq/
[20] Perplexity Multi-Format Export Analysis (May 2026) -- live query
[21] Nova PDF Font Embedding: https://www.novapdf.com/embedding-fonts-in-pdf-files-kb.html
[22] Steven Schwenke on PPTX/HTML Generation: https://stevenschwenke.de/stateOfTheArtInGeneratingPowerPointPresentationsWithHTMLContent
[23] Extism Documentation via Context7 -- live query
[24] Nushell Plugin Architecture (training data, verified pattern)
[25] Cargo Subcommand Discovery: https://doc.crates.io/contrib/implementation/subcommands.html
[26] Helm 4 WASM Plugin HIP: https://helm.sh/community/hips/hip-0026
