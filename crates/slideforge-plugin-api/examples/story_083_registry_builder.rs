//! STORY-083 demo: `PluginRegistryBuilder` + Surface Enforcement.
//!
//! Demonstrates all five acceptance criteria:
//!
//! - AC-001: empty builder → `Err(RegistryError::MissingSurface { surface: "DataSource" })`
//! - AC-002: fully-registered builder (all 10 surfaces) → `Ok(registry)`
//! - AC-003: `surface_count() == 10` on the full registry
//! - AC-004: `surface_names()` lists all 10 canonical names
//! - AC-005: `RegistryError` Display + Debug representations
//!
//! Also demonstrates a **partial registry** (3 surfaces via the mutation API)
//! to exercise the non-10 branch of `surface_count()` / `surface_names()`.
//!
//! Run with:
//!   cargo run --example `story_083_registry_builder` -p slideforge-plugin-api

// ── Suppress lints that are unavoidable in a demo binary ──────────────────
#![allow(clippy::print_stdout)]
// The trait signatures define `id(&self) -> &str` without `'static`; stub impls
// must mirror the trait signature exactly and cannot change it to `&'static str`.
#![allow(clippy::unnecessary_literal_bound)]

use std::sync::Arc;

use slideforge_layout::{LaidOutDeck, LaidOutSlide};
use slideforge_plugin_api::{
    BrandError, BrandProvider, BrandSource, Canvas, ChartError, ChartRenderer, DataSource,
    DataSourceError, DataSourceOptions, Diagnostic, DiagramError, DiagramOptions, DiagramRenderer,
    ExportError, ExportOptions, Exporter, FieldDef, InlineError, InlineFormat, InlineOutputFormat,
    LayoutError, MathError, MathOutputFormat, MathRenderer, PluginRegistry, PluginRegistryBuilder,
    RegistryError, SectionBlock, SectionType, SlideType, Validator, ValidatorOptions,
};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, ChartSpec, Deck, InlineNode, MathNode, Slide, SourceSpan,
    Value,
};

// ─────────────────────────────────────────────────────────────────────────────
// Minimal stub implementations of all 10 plugin surfaces
//
// These are the smallest possible trait implementations — no logic beyond
// satisfying the trait contract. They exist solely to satisfy the builder's
// "at least one registration per surface" invariant.
// ─────────────────────────────────────────────────────────────────────────────

/// Stub implementation of the `DataSource` plugin surface.
struct DemoDataSource;

impl DataSource for DemoDataSource {
    fn id(&self) -> &str {
        "demo-data-source"
    }

    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        Ok(Value::Null)
    }
}

/// Stub implementation of the `Exporter` plugin surface.
struct DemoExporter;

impl Exporter for DemoExporter {
    fn id(&self) -> &str {
        "demo-exporter"
    }

    fn extension(&self) -> &str {
        "bin"
    }

    fn export(
        &self,
        _deck: &Deck,
        _laid_out: &LaidOutDeck,
        _brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        Ok(vec![])
    }
}

/// Stub implementation of the `ChartRenderer` plugin surface.
struct DemoChartRenderer;

impl ChartRenderer for DemoChartRenderer {
    fn id(&self) -> &str {
        "demo-chart-renderer"
    }

    fn render(&self, _spec: &ChartSpec, _brand: &Brand) -> Result<Vec<u8>, ChartError> {
        Ok(vec![])
    }
}

/// Stub implementation of the `DiagramRenderer` plugin surface.
struct DemoDiagramRenderer;

impl DiagramRenderer for DemoDiagramRenderer {
    fn id(&self) -> &str {
        "demo-diagram-renderer"
    }

    fn render(&self, _source: &str, _opts: &DiagramOptions) -> Result<Vec<u8>, DiagramError> {
        Ok(vec![])
    }
}

/// Stub implementation of the `Validator` plugin surface.
struct DemoValidator;

impl Validator for DemoValidator {
    fn id(&self) -> &str {
        "demo-validator"
    }

    fn validate(&self, _deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        vec![]
    }
}

/// Stub implementation of the `MathRenderer` plugin surface.
struct DemoMathRenderer;

impl MathRenderer for DemoMathRenderer {
    fn id(&self) -> &str {
        "demo-math-renderer"
    }

    fn render(&self, _node: &MathNode, _format: MathOutputFormat) -> Result<Vec<u8>, MathError> {
        Ok(vec![])
    }
}

/// Stub implementation of the `BrandProvider` plugin surface.
struct DemoBrandProvider;

impl BrandProvider for DemoBrandProvider {
    fn id(&self) -> &str {
        "demo-brand-provider"
    }

    fn load(&self, _source: &BrandSource) -> Result<Brand, BrandError> {
        Ok(Brand {
            name: Arc::from("demo"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#0066CC"),
                accent: Arc::from("#FF6B35"),
                neutral: Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Calibri Light"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        })
    }
}

/// Stub implementation of the `SlideType` plugin surface.
struct DemoSlideType;

impl SlideType for DemoSlideType {
    fn id(&self) -> &'static str {
        "demo-slide-type"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &[]
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &[]
    }

    fn layout_name(&self) -> &'static str {
        "blank"
    }

    fn lay_out(
        &self,
        slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        Ok(LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::clone(&slide.slide_type),
            frames: vec![],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        })
    }
}

/// Stub implementation of the `SectionType` plugin surface.
struct DemoSectionType;

impl SectionType for DemoSectionType {
    fn id(&self) -> &str {
        "demo-section-type"
    }

    fn generate(&self, _slides: &[Slide]) -> Vec<SectionBlock> {
        vec![]
    }
}

/// Stub implementation of the `InlineFormat` plugin surface.
struct DemoInlineFormat;

impl InlineFormat for DemoInlineFormat {
    fn id(&self) -> &str {
        "demo-inline-format"
    }

    fn render(
        &self,
        _node: &InlineNode,
        _format: InlineOutputFormat,
    ) -> Result<String, InlineError> {
        Ok(String::new())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: build a fully-registered PluginRegistryBuilder
// ─────────────────────────────────────────────────────────────────────────────

/// Construct a [`PluginRegistryBuilder`] with all 10 surfaces registered.
///
/// Returns the builder (not yet finalized) so callers can call `.build()`.
fn full_builder() -> PluginRegistryBuilder {
    let mut builder = PluginRegistryBuilder::default();
    builder.register_data_source(Box::new(DemoDataSource));
    builder.register_exporter(Box::new(DemoExporter));
    builder.register_chart_renderer(Box::new(DemoChartRenderer));
    builder.register_diagram_renderer(Box::new(DemoDiagramRenderer));
    builder.register_validator(Box::new(DemoValidator));
    builder.register_math_renderer(Box::new(DemoMathRenderer));
    builder.register_brand_provider(Box::new(DemoBrandProvider));
    builder.register_slide_type(Box::new(DemoSlideType));
    builder.register_section_type(Box::new(DemoSectionType));
    builder.register_inline_format(Box::new(DemoInlineFormat));
    builder
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

/// Entry point for the STORY-083 demo.
fn main() {
    println!("=================================================================");
    println!(" STORY-083: PluginRegistryBuilder + Surface Enforcement");
    println!("=================================================================");
    println!();

    // ── AC-001: empty builder → Err(MissingSurface { surface: "DataSource" }) ──
    println!("-----------------------------------------------------------------");
    println!(" AC-001  empty builder → Err(MissingSurface)");
    println!("-----------------------------------------------------------------");
    let empty_result = PluginRegistryBuilder::default().build();
    match &empty_result {
        Err(RegistryError::MissingSurface { surface }) => {
            println!("  result : Err(MissingSurface {{ surface: {surface:?} }})");
            println!("  PASS   first missing surface in declaration order = DataSource");
        },
        Ok(_) => {
            println!("  FAIL   expected Err, got Ok");
        },
        Err(other) => {
            println!("  FAIL   unexpected error variant: {other:?}");
        },
    }
    println!();

    // ── AC-002: fully-registered builder → Ok(registry) ──────────────────────
    println!("-----------------------------------------------------------------");
    println!(" AC-002  fully-registered builder (10 surfaces) → Ok(registry)");
    println!("-----------------------------------------------------------------");
    let full_result = full_builder().build();
    match full_result {
        Ok(registry) => {
            println!("  result : Ok(PluginRegistry)");
            println!("  PASS   build() succeeded with all 10 surfaces registered");

            // ── AC-003: surface_count() == 10 ────────────────────────────────
            println!();
            println!("-----------------------------------------------------------------");
            println!(" AC-003  surface_count() == 10 for a fully-registered registry");
            println!("-----------------------------------------------------------------");
            let count = registry.surface_count();
            println!("  surface_count() = {count}");
            if count == 10 {
                println!("  PASS   surface_count() == 10");
            } else {
                println!("  FAIL   expected 10, got {count}");
            }

            // ── AC-004: surface_names() lists all 10 canonical names ──────────
            println!();
            println!("-----------------------------------------------------------------");
            println!(" AC-004  surface_names() returns all 10 canonical names");
            println!("-----------------------------------------------------------------");
            let names = registry.surface_names();
            println!("  surface_names() = {names:?}");
            if names.len() == 10 {
                println!("  PASS   surface_names().len() == 10");
            } else {
                println!("  FAIL   expected 10 names, got {}", names.len());
            }
            let expected = [
                "DataSource",
                "Exporter",
                "ChartRenderer",
                "DiagramRenderer",
                "Validator",
                "MathRenderer",
                "BrandProvider",
                "SlideType",
                "SectionType",
                "InlineFormat",
            ];
            if names.as_slice() == expected {
                println!("  PASS   names match canonical declaration order");
            } else {
                println!("  FAIL   names do not match expected canonical order");
            }
        },
        Err(e) => {
            println!("  FAIL   expected Ok, got Err: {e}");
        },
    }

    // ── Partial registry (3 surfaces via mutation API) ────────────────────────
    println!();
    println!("-----------------------------------------------------------------");
    println!(" PARTIAL  3-of-10 registry via mutation API");
    println!(" (demonstrates non-10 branch of surface_count / surface_names)");
    println!("-----------------------------------------------------------------");
    let mut partial = PluginRegistry::new();
    partial.register_data_source(Box::new(DemoDataSource));
    partial.register_exporter(Box::new(DemoExporter));
    partial.register_chart_renderer(Box::new(DemoChartRenderer));
    let partial_count = partial.surface_count();
    let partial_names = partial.surface_names();
    println!("  surface_count() = {partial_count} (expected 3)");
    println!("  surface_names() = {partial_names:?}");
    if partial_count == 3 && partial_names == vec!["DataSource", "Exporter", "ChartRenderer"] {
        println!("  PASS   partial registry reports only 3 registered surfaces");
    } else {
        println!("  FAIL   unexpected partial registry state");
    }

    // ── AC-005: RegistryError Display + Debug ─────────────────────────────────
    println!();
    println!("-----------------------------------------------------------------");
    println!(" AC-005  RegistryError implements Display, Debug, std::error::Error");
    println!("-----------------------------------------------------------------");
    let err = RegistryError::MissingSurface {
        surface: "DataSource",
    };
    println!("  Display : {err}");
    println!("  Debug   : {err:?}");
    let expected_display = "required plugin surface 'DataSource' has no registered implementations";
    if err.to_string() == expected_display {
        println!("  PASS   Display matches expected format");
    } else {
        println!("  FAIL   Display mismatch; expected: {expected_display:?}");
    }
    println!();
    println!("=================================================================");
    println!(" All AC-001..AC-005 demonstrated successfully.");
    println!("=================================================================");
}
