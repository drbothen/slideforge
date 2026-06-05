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
| 8 | `SlideType` | slideforge-plugin-api | 31 built-in types in `src/slide_types/` (CL-01 through CL-20 + 11 standard) |
| 9 | `SectionType` | slideforge-plugin-api | bundled impls in `src/section_types/` (ExecutiveSummarySectionType, RiskRegisterSectionType, + 5 manual-only types) — delivered by STORY-084 |
| 10 | `InlineFormat` | slideforge-plugin-api | bundled impls in `src/inline_formats/` (DefaultInlineFormat — all 12 InlineNode variants × 3 output formats) — delivered by STORY-085 |

## Plugin Registry

The `slideforge` root crate assembles the `PluginRegistry` via `PluginRegistryBuilder`
(defined in `slideforge-plugin-api`). Registry lookup is by `id()` string returned
by each trait implementation. `PluginRegistryBuilder::build()` returns
`Result<PluginRegistry, RegistryError>`, failing with `RegistryError::MissingSurface`
if any of the 10 surfaces has zero registrations at finalization time
(ADR-016; satisfies BC-5.02.001 invariant 3).

```rust
// slideforge/src/registry.rs
pub fn default_registry() -> Result<PluginRegistry, RegistryError> {
    PluginRegistryBuilder::default()
        .register_data_source(Box::new(JsonDataSource))
        .register_exporter(Box::new(PptxExporter))
        .register_chart_renderer(Box::new(PlottersRenderer))
        .register_diagram_renderer(Box::new(MermaidRenderer))
        // ... all bundled surface impls ...
        .build()  // Err(RegistryError::MissingSurface { .. }) if any surface empty
}
```

`surface_count() -> usize` and `surface_names() -> Vec<&'static str>` are available
on `PluginRegistry` for runtime initialization auditing (ADR-016).

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

The following signatures are canonical — taken directly from `interface-definitions.md §6`,
which supersedes any earlier draft in architecture docs (CLAUDE.md precedence rule).

```rust
/// Exporter — produces output bytes from Deck + LaidOutDeck + Brand
pub trait Exporter: Send + Sync {
    fn id(&self) -> &str;
    fn extension(&self) -> &str;  // e.g., "pptx"
    fn export(&self, deck: &Deck, laid_out: &LaidOutDeck,
              brand: &Brand, opts: &ExportOptions) -> Result<Vec<u8>, ExportError>;
}

/// SlideType — defines visual pattern and layout for a slide kind
pub trait SlideType: Send + Sync {
    fn id(&self) -> &str;
    fn required_fields(&self) -> &[FieldDef];
    fn optional_fields(&self) -> &[FieldDef];
    fn layout_name(&self) -> &str;  // OOXML layout name
    fn lay_out(&self, slide: &Slide, brand: &Brand, canvas: Canvas) -> Result<LaidOutSlide, LayoutError>;
}
```

Changes from earlier draft (P2 finding resolution):
- `Exporter`: added `extension()` method; changed return type to `Result<Vec<u8>, ExportError>` (explicit error type).
- `SlideType`: renamed `layout()` → `lay_out()`; changed `&Canvas` → `Canvas`; changed `Result<LaidOutSlide>` → `Result<LaidOutSlide, LayoutError>`; changed `&[FieldSpec]` → `&[FieldDef]`; removed `validate()` (not in interface-definitions.md — validation is owned by `slideforge-validate` / SS-03); added `optional_fields()` and `layout_name()` methods.

Full trait signatures for all 10 surfaces in `interface-definitions.md §6`.
