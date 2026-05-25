---
document_type: domain-spec-section
level: L2
section: "entities"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/planning/domain-research.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q3-decision-final.md
  - .factory/planning/ir-prior-art.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Domain Entities

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.

## Bounded Context Overview

The four bounded contexts communicate through two stable IR contracts (domain-research.md §1.1):

- **Authoring → Layout**: `Deck` IR (semantic, pre-layout)
- **Layout → Export**: `LaidOutDeck` IR (geometric, post-layout)
- **Branding → Layout and Export**: `Brand` struct (shared kernel)

---

## Aggregate Roots

### Deck

The primary aggregate root. Identity: `DeckId` (file path + content hash).

| Attribute | Type | Invariant |
|-----------|------|-----------|
| `metadata` | DeckMetadata | Must declare `slideforge_version`; `lang` defaults to "en" |
| `brand` | BrandRef | Must resolve to a valid Brand |
| `slides` | `Vec<Slide>` | At least one slide required |
| `vars` | VariableMap | All variable names must be snake_case |
| `set_rules` | SetRuleMap | Keyed by slide type id |
| `variants` | VariantMap | All referenced parent variants must exist |
| `document` | Option<DocumentStructure> | If absent, default structure applies |
| `assets` | AssetRegistry | All referenced asset paths must exist at build time |

Slides do not have an independent lifecycle — they exist only within a Deck. All slide
operations go through Deck methods (domain-research.md §1.2).

### Brand

Identity: `BrandId` (name + version from brand.toml or extracted from template).

| Attribute | Type | Invariant |
|-----------|------|-----------|
| `palette` | ColorPalette | All 12 OOXML scheme color slots populated |
| `typography` | TypographyScheme | Must define heading + body fonts |
| `logo` | Option<AssetRef> | If present, path must be valid image |
| `footer` | Option<FooterSpec> | May reference `{{ brand.name }}` |
| `templates` | TemplateRefs | PPTX and DOCX template paths are optional |

---

## Core Entities (within Deck aggregate)

### Slide

An individual slide within a Deck. Not an aggregate root — no independent lifecycle.

| Attribute | Type | Notes |
|-----------|------|-------|
| `slide_type` | SlideTypeId | One of 31 built-in types or a defined alias |
| `fields` | FieldMap | Type-specific fields (title, bullets, etc.) |
| `tags` | Vec<Tag> | Used for variant include/exclude filtering |
| `brand_overlay` | Option<BrandOverlay> | Per-slide logo/footer/confidentiality override |
| `registers` | RegisterMap | `notes`, `report`, `detail` per slide |
| `shapes` | Vec<ShapeSpec> | Custom shapes via `shape:` blocks |

### ContentBlock

A structural content unit within a slide or document section.

Types: Paragraph, BulletList, OrderedList, Table, CodeBlock, Callout, BlockQuote, Figure,
DataTable (from @for), ChartFigure, DiagramFigure, MathDisplay.

### TextRun / Inline

An inline-formatted run of text within a paragraph or heading.

Types: Plain, Bold, Italic, Code, Link, Math (inline), Footnote, CrossRef, FigRef,
Superscript, Subscript, Strikethrough, Highlight, Variable (resolved at eval).

### DataSource

A configured data-loading plugin instance. Identifies source type (json/csv/yaml/toml/http/
xlsx/sqlite), URI, and watch interval for `slideforge watch` mode.

### Variable

A named value in scope at evaluation time. Scope chain: slide-level > variant-level > deck-level.
Names follow `[a-z][a-z0-9_]*` pattern. Undefined variable = compile error.

### Expression

A computed value from the `{{ expr }}` interpolation syntax. Produces a typed value (string,
number, boolean, list, map) via arithmetic, comparison, logical, field-access, and pipe-filter
operators.

---

## Value Objects

From domain-research.md §1.3:

| Value Object | Fields | Key Invariant |
|-------------|--------|---------------|
| `Emu` | `i64` (914400 per inch) | Non-negative for dimensions; may be negative for offsets |
| `Color` | `SchemeColor(token)` or `SrgbColor(r, g, b)` | Scheme token must exist in brand palette |
| `FontRef` | `family: Arc<str>`, `weight`, `style` | Family in brand typography or system fallback list |
| `Span` (text) | `text: Arc<str>`, `formatting` | Text non-empty (empty spans are elided) |
| `SourceSpan` | `file: FileId`, `start: usize`, `end: usize` | `start <= end`; file in source map |
| `AspectRatio` | `width: u32`, `height: u32` | Both > 0; normalized (no GCD > 1) |
| `SlideTypeId` | `Arc<str>` | Must match a registered SlideType plugin |
| `Tag` | `Arc<str>` | Non-empty; used for variant filtering |

---

## IR Types (Pipeline Contracts)

### Deck IR (Authoring → Layout contract)

```
Deck {
  metadata: DeckMetadata,
  brand: Brand,
  slides: Vec<Arc<Slide>>,
  // All vars resolved; all includes expanded; all @for iterations materialized
}
```

All types implement `Hash + Eq + Clone` from day one (comemo compatibility, domain-research.md §1.2).
All coordinates use `Emu` (integer EMU, not f64).

### LaidOutDeck IR (Layout → Export contract)

```
LaidOutDeck {
  slides: Vec<LaidOutSlide>,
  // Each slide: positioned frames (Point, FrameItem) pairs
}
```

Exporters consume both `LaidOutDeck` (geometry) and the source `Deck` (semantic features
like accessibility metadata, speaker notes, placeholder inheritance).

---

## Plugin System Entities

### PluginRegistry

Holds all registered plugin instances at startup. Assembled at binary startup from
19 bundled plugin crates. External (v2): WASM modules loaded at runtime.

### SlideType (plugin)

Defines layout rules for one slide type: `layout()`, `validate()`, `required_fields()`.
31 instances ship bundled.

### Exporter (plugin)

Consumes `(Deck, LaidOutDeck, Brand, ExportOptions) → Vec<u8>`. Five instances: pptx,
docx, pdf, html, preview.

---

## Entity Relationships Summary

```
Deck (aggregate root)
  |-- has many --> Slide
  |                  |-- has type --> SlideType (plugin)
  |                  |-- has many --> ContentBlock
  |                  |                  |-- has many --> TextRun
  |                  |-- has --> RegisterMap (notes/report/detail)
  |                  |-- has many --> ShapeSpec
  |-- references --> Brand (aggregate root)
  |-- has many --> DataSource
  |-- has many --> Variable
  |-- has many --> Variant
  |
  compiled to
  |
  v
Deck IR  -------> LaidOutDeck IR -------> Output bytes (per Exporter)
```
