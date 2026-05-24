---
title: "Q3 Decision — Data Binding & Plugin Architecture"
date: 2026-05-24
status: LOCKED
decided_by: human
---

# Q3 Decision: Data Binding & Plugin Architecture

## Decision Summary
slideforge uses a PLUGIN-FIRST architecture from day 1. The plugin system IS the architecture — there is no "built-in code" and "plugin code," there is just plugin code. Some plugins ship bundled with the binary (v1.0), others are installable later (v2+). ALL go through the same API. We dog-food every extensibility surface.

## The Plugin Principle
"If JSON data loading can't work through the DataSource trait, the trait is wrong. If PPTX export can't work through the Exporter trait, the trait is wrong. We fix the API, not bypass it."

## 10 Extensibility Surfaces

Every extensibility surface gets a trait in `slideforge-plugin-api`:

| # | Surface | Trait | What it does | v1.0 bundled plugins | v2+ installable |
|---|---------|-------|-------------|---------------------|-----------------|
| 1 | Data sources | `DataSource` | Fetches data → returns Value | json, csv, yaml, toml, http/https, xlsx, sqlite | postgres, mysql, parquet, graphql, grafana, datadog, jira, s3 |
| 2 | Exporters | `Exporter` | Consumes Deck IR → produces output | pptx, docx, pdf, html, preview | latex/beamer, google-slides, keynote, xlsx, epub |
| 3 | Chart renderers | `ChartRenderer` | Consumes ChartSpec → produces SVG | plotters (bar, line, pie, scatter, area, histogram, stacked) | vega/vega-lite, native OOXML ChartML |
| 4 | Diagram renderers | `DiagramRenderer` | Consumes diagram source → produces SVG | mermaid (ships v1.0) | graphviz, d2, excalidraw |
| 5 | Validators | `Validator` | Consumes Deck → produces Diagnostics | canvas-overflow, wcag-contrast, color-name, alt-text, bullet-length, weight-normalization | custom org rules, policy engine |
| 6 | Math renderers | `MathRenderer` | Consumes LaTeX → produces MathML/OMML/HTML | pulldown-latex + KaTeX + mml2omml | mathjax, typst-math |
| 7 | Brand providers | `BrandProvider` | Loads/synthesizes/extracts brand config | file-based (.pptx, .docx, .toml) | api-based (corporate brand registry), git-based |
| 8 | Slide types | `SlideType` | Defines visual slide pattern + layout rules | 31 built-in types | v2: user-defined component types |
| 9 | Section types | `SectionType` | Defines document section pattern | auto-generated + manual sections | v2: user-defined section types |
| 10 | Inline formatters | `InlineFormat` | Defines inline formatting rule | bold, italic, code, link, math, footnote, cross-ref | v2: custom inline extensions |

## What Is NOT a Plugin (Core Engine)

These are the RAILS that plugins run on — fixed architecture, not extensible:
- Parser (chumsky) — the DSL grammar is fixed
- Evaluator — expression evaluation rules are fixed
- IR types (Deck, Slide, Block, Inline, Value) — the data model is fixed
- Layout engine — the positioning/sizing algorithm is core
- Plugin registry — the mechanism itself is core
- The parse → eval → layout → export pipeline

## v1.0 Data Source Plugins (Tier 1 + Tier 2)

| Plugin | Crate dependency | Pure Rust? | Format |
|--------|-----------------|------------|--------|
| json | serde_json | yes | JSON files |
| csv | csv (BurntSushi) | yes | CSV files |
| yaml | serde_yaml | yes | YAML files |
| toml | toml | yes | TOML files |
| http | reqwest | yes | HTTP/HTTPS JSON APIs |
| xlsx | calamine | yes | Excel spreadsheets |
| sqlite | rusqlite | C dep (bundled) | SQLite databases |

## v1.0 Plugin Loading: Static (Compiled In)

All plugins are statically compiled into the single binary. The plugin system is an internal architecture decision — invisible to the v1.0 user. They get a single binary with everything.

## v2 Plugin Loading: Dynamic

External plugins loaded via WASI modules or shared libraries:
```bash
slideforge plugin install slideforge-grafana
slideforge plugin install slideforge-postgres
```
Uses the EXACT same traits as bundled plugins.

## v1.0 Mermaid Diagram Renderer

Mermaid ships in v1.0 as a bundled DiagramRenderer plugin. Implementation approach to be determined by Spike S14:
- Option A: mermaid-rs (pure Rust, if mature enough)
- Option B: Bundled mermaid CLI (Node.js — conflicts with single-binary goal)
- Option C: mermaid WASM module (compiled from JS, embedded)
- Option D: Headless browser rendering via Playwright/Chrome

Architect must resolve via S14 spike, respecting the single-binary constraint where possible.

## Crate Layout (17 crates)

```
crates/
├── slideforge-plugin-api/    # THE foundation — all 10 trait definitions
├── slideforge-syntax/        # Parser (core, NOT a plugin)
├── slideforge-eval/          # Evaluator (core, NOT a plugin)
├── slideforge-layout/        # Layout engine (core, NOT a plugin)
├── slideforge-data/          # Bundled DataSource plugins (json/csv/yaml/toml/http/xlsx/sqlite)
├── slideforge-pptx/          # Bundled Exporter plugin
├── slideforge-docx/          # Bundled Exporter plugin
├── slideforge-pdf/           # Bundled Exporter plugin
├── slideforge-html/          # Bundled Exporter plugin
├── slideforge-preview/       # Bundled Exporter/preview plugin
├── slideforge-charts/        # Bundled ChartRenderer plugin (plotters)
├── slideforge-diagrams/      # Bundled DiagramRenderer plugin (mermaid)
├── slideforge-math/          # Bundled MathRenderer plugin (pulldown-latex + KaTeX)
├── slideforge-validate/      # Bundled Validator plugins
├── slideforge-brand/         # Bundled BrandProvider plugin
├── slideforge-types/         # Bundled SlideType + SectionType plugins (31 types)
├── slideforge/               # Main library — assembles PluginRegistry
└── slideforge-cli/           # CLI binary
```

## Dog-Fooding Guarantee

Every bundled plugin uses the same trait API that external plugins will use. If any bundled plugin needs to bypass the API, the API is wrong and must be fixed. The built-in plugins ARE the test suite for the plugin API.
