---
document_type: architecture-section
section: plugin-architecture
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Plugin Architecture

## Plugin Principle (DI-008, ADR-006)

All functionality — including bundled features — flows through plugin traits defined
in `slideforge-plugin-api`. There is no "built-in code" vs "plugin code": only plugin
code. The bundled plugins ARE the test suite for the plugin API. If a bundled plugin
needs to bypass the API, the API is wrong and must be fixed (q3-decision-final.md).

## 10 Extensibility Surfaces

| # | Trait | Owner Crate | v1.0 Bundled Plugins |
|---|-------|-------------|---------------------|
| 1 | `DataSource` | slideforge-data | json, csv, yaml, toml, http, xlsx, sqlite |
| 2 | `Exporter` | slideforge-pptx/docx/pdf/preview | pptx, docx, pdf, html, preview |
| 3 | `ChartRenderer` | slideforge-charts | plotters (bar, line, pie, scatter, etc.) |
| 4 | `DiagramRenderer` | slideforge-diagrams | mermaid (via mermaid-rs-renderer v0.2.2) |
| 5 | `Validator` | slideforge-validate | canvas-overflow, wcag-contrast, alt-text, color-name, bullet-length, weight-normalization |
| 6 | `MathRenderer` | slideforge-math | pulldown-latex + KaTeX (OMML via spike S-MATH-01) |
| 7 | `BrandProvider` | slideforge-brand | file-based (.pptx, .toml) |
| 8 | `SlideType` | slideforge-types | 31 built-in types (CL-01 through CL-20 + 11 standard) |
| 9 | `SectionType` | slideforge-types | auto-generated + manual sections |
| 10 | `InlineFormat` | slideforge-types | bold, italic, code, link, math, footnote, cross-ref |

## Plugin Registry

The `slideforge` root crate assembles the `PluginRegistry` using all bundled plugins.
Registry lookup is by `id()` string returned by each trait implementation.

```rust
// slideforge/src/registry.rs
pub fn default_registry() -> PluginRegistry {
    let mut r = PluginRegistry::new();
    r.register_data_source(Box::new(JsonDataSource));
    r.register_exporter(Box::new(PptxExporter));
    r.register_chart_renderer(Box::new(PlottersRenderer));
    r.register_diagram_renderer(Box::new(MermaidRenderer));
    // ... all 31 slide types ...
    r
}
```

Registry is constructed once at process startup (or test setup). All lookups
are immutable borrows after initialization. The registry is `Send + Sync`.

## v1.0 Plugin Loading: Static

All v1.0 plugins are statically compiled into the binary. The plugin architecture
is internal to v1.0; users see a single binary. Dynamic loading (WASI modules or
shared libraries) is a v2 feature.

## Dog-Fooding Invariant

The CI pipeline verifies the dog-fooding guarantee by checking that no bundled
plugin crate imports internal types from another bundled plugin crate (they may
only import from `slideforge-plugin-api` and `slideforge-types`). Violation of
this rule is a CI blocking error.

## Key Trait Signatures

```rust
pub trait Exporter: Send + Sync {
    fn id(&self) -> &str;
    fn export(&self, deck: &Deck, laid_out: &LaidOutDeck,
              brand: &Brand, opts: &ExportOptions) -> Result<Vec<u8>>;
}

pub trait SlideType: Send + Sync {
    fn id(&self) -> &str;
    fn layout(&self, slide: &Slide, brand: &Brand, canvas: &Canvas) -> Result<LaidOutSlide>;
    fn validate(&self, slide: &Slide) -> Vec<Diagnostic>;
    fn required_fields(&self) -> &[FieldSpec];
}
```

Full trait signatures are in `interface-definitions.md §6` and `q3-decision-final.md`.
