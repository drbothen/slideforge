//! Dog-food integration test — BC-5.02.002 verification.
//!
//! This test verifies that a minimal plugin can be written using ONLY the
//! public API of `slideforge-plugin-api` and `slideforge-types`, with no
//! access to internal workspace crates. The test is intentionally minimal:
//! it proves the API surface is self-contained, not that the plugin does
//! anything useful.

use slideforge_layout::{LaidOutDeck, LaidOutSlide};
use slideforge_plugin_api::{
    BrandError, BrandProvider, BrandSource, Canvas, ChartError, ChartRenderer, DataSource,
    DataSourceError, DataSourceOptions, Diagnostic, DiagramError, DiagramOptions, DiagramRenderer,
    ExportError, ExportOptions, Exporter, FieldDef, InlineError, InlineFormat, InlineOutputFormat,
    LayoutError, MathError, MathOutputFormat, MathRenderer, PluginRegistry, SectionBlock,
    SectionType, SlideType, Validator, ValidatorOptions,
};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, ChartSpec, Deck, InlineNode, MathNode, Slide, SourceSpan,
    Value,
};
use std::sync::Arc;

// ─────────────────────────────────────────────────────────────────────────────
// Minimal test plugins — one per surface
// Each plugin uses ONLY the public API of slideforge-plugin-api + slideforge-types
// ─────────────────────────────────────────────────────────────────────────────

struct TestDataSource;

impl DataSource for TestDataSource {
    fn id(&self) -> &'static str {
        "test"
    }

    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        Ok(Value::Null)
    }
}

struct TestExporter;

impl Exporter for TestExporter {
    fn id(&self) -> &'static str {
        "test-export"
    }

    fn extension(&self) -> &'static str {
        "test"
    }

    fn export(
        &self,
        _deck: &Deck,
        _laid_out: &LaidOutDeck,
        _brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        Ok(b"test-output".to_vec())
    }
}

struct TestChartRenderer;

impl ChartRenderer for TestChartRenderer {
    fn id(&self) -> &'static str {
        "test-chart"
    }

    fn render(&self, _spec: &ChartSpec, _brand: &Brand) -> Result<Vec<u8>, ChartError> {
        Ok(b"<svg/>".to_vec())
    }
}

struct TestDiagramRenderer;

impl DiagramRenderer for TestDiagramRenderer {
    fn id(&self) -> &'static str {
        "test-diagram"
    }

    fn render(&self, _source: &str, _opts: &DiagramOptions) -> Result<Vec<u8>, DiagramError> {
        Ok(b"<svg/>".to_vec())
    }
}

struct TestValidator;

impl Validator for TestValidator {
    fn id(&self) -> &'static str {
        "test-validator"
    }

    fn validate(&self, _deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        vec![]
    }
}

struct TestMathRenderer;

impl MathRenderer for TestMathRenderer {
    fn id(&self) -> &'static str {
        "test-math"
    }

    fn render(&self, _node: &MathNode, _format: MathOutputFormat) -> Result<Vec<u8>, MathError> {
        Ok(b"<math/>".to_vec())
    }
}

struct TestBrandProvider;

impl BrandProvider for TestBrandProvider {
    fn id(&self) -> &'static str {
        "test-brand"
    }

    fn load(&self, _source: &BrandSource) -> Result<Brand, BrandError> {
        Ok(Brand {
            name: Arc::from("test"),
            palette: BrandPalette {
                primary: Arc::from("#000000"),
                secondary: Arc::from("#ffffff"),
                accent: Arc::from("#ff0000"),
                neutral: Arc::from("#888888"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Arial"),
                body: Arc::from("Arial"),
                mono: Arc::from("Courier"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        })
    }
}

struct TestSlideType;

impl SlideType for TestSlideType {
    fn id(&self) -> &'static str {
        "test-slide"
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
            slide_type_keyword: std::sync::Arc::clone(&slide.slide_type),
            frames: vec![],
            speaker_notes: None,
            register_tags: vec![],
        register_content: vec![],
        })
    }
}

struct TestSectionType;

impl SectionType for TestSectionType {
    fn id(&self) -> &'static str {
        "test-section"
    }

    fn generate(&self, _slides: &[Slide]) -> Vec<SectionBlock> {
        vec![]
    }
}

struct TestInlineFormat;

impl InlineFormat for TestInlineFormat {
    fn id(&self) -> &'static str {
        "test-inline"
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
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// BC-5.02.002 postcondition 3: a plugin using only the public API compiles
/// and registers/looks up correctly.
#[test]
fn dog_food_data_source() {
    let mut registry = PluginRegistry::new();
    registry.register_data_source(Box::new(TestDataSource));
    let plugin = registry
        .lookup_data_source("test")
        .expect("registered plugin not found");
    assert_eq!(plugin.id(), "test");
    let result = plugin
        .load("test://data", &DataSourceOptions::default())
        .expect("load should succeed");
    assert_eq!(result, Value::Null);
}

#[test]
fn dog_food_exporter() {
    let mut registry = PluginRegistry::new();
    registry.register_exporter(Box::new(TestExporter));
    let plugin = registry
        .lookup_exporter("test-export")
        .expect("registered exporter not found");
    assert_eq!(plugin.id(), "test-export");
    assert_eq!(plugin.extension(), "test");
}

#[test]
fn dog_food_chart_renderer() {
    let mut registry = PluginRegistry::new();
    registry.register_chart_renderer(Box::new(TestChartRenderer));
    let plugin = registry
        .lookup_chart_renderer("test-chart")
        .expect("registered chart renderer not found");
    assert_eq!(plugin.id(), "test-chart");
}

#[test]
fn dog_food_diagram_renderer() {
    let mut registry = PluginRegistry::new();
    registry.register_diagram_renderer(Box::new(TestDiagramRenderer));
    let plugin = registry
        .lookup_diagram_renderer("test-diagram")
        .expect("registered diagram renderer not found");
    assert_eq!(plugin.id(), "test-diagram");
}

#[test]
fn dog_food_validator() {
    let mut registry = PluginRegistry::new();
    registry.register_validator(Box::new(TestValidator));
    let plugin = registry
        .lookup_validator("test-validator")
        .expect("registered validator not found");
    assert_eq!(plugin.id(), "test-validator");
}

#[test]
fn dog_food_math_renderer() {
    let mut registry = PluginRegistry::new();
    registry.register_math_renderer(Box::new(TestMathRenderer));
    let plugin = registry
        .lookup_math_renderer("test-math")
        .expect("registered math renderer not found");
    assert_eq!(plugin.id(), "test-math");
}

#[test]
fn dog_food_brand_provider() {
    let mut registry = PluginRegistry::new();
    registry.register_brand_provider(Box::new(TestBrandProvider));
    let plugin = registry
        .lookup_brand_provider("test-brand")
        .expect("registered brand provider not found");
    assert_eq!(plugin.id(), "test-brand");
    let brand = plugin
        .load(&BrandSource::TomlFile(Arc::from("brand.toml")))
        .expect("brand load should succeed");
    assert_eq!(brand.name.as_ref(), "test");
}

#[test]
fn dog_food_slide_type() {
    let mut registry = PluginRegistry::new();
    registry.register_slide_type(Box::new(TestSlideType));
    let plugin = registry
        .lookup_slide_type("test-slide")
        .expect("registered slide type not found");
    assert_eq!(plugin.id(), "test-slide");
    assert_eq!(plugin.layout_name(), "blank");
    assert!(plugin.required_fields().is_empty());
    assert!(plugin.optional_fields().is_empty());
}

#[test]
fn dog_food_section_type() {
    let mut registry = PluginRegistry::new();
    registry.register_section_type(Box::new(TestSectionType));
    let plugin = registry
        .lookup_section_type("test-section")
        .expect("registered section type not found");
    assert_eq!(plugin.id(), "test-section");
}

#[test]
fn dog_food_inline_format() {
    let mut registry = PluginRegistry::new();
    registry.register_inline_format(Box::new(TestInlineFormat));
    let plugin = registry
        .lookup_inline_format("test-inline")
        .expect("registered inline format not found");
    assert_eq!(plugin.id(), "test-inline");
}

/// AC-015: All 10 surfaces can be registered and looked up on a single registry.
#[test]
fn dog_food_all_10_surfaces_round_trip() {
    let mut registry = PluginRegistry::new();
    registry.register_data_source(Box::new(TestDataSource));
    registry.register_exporter(Box::new(TestExporter));
    registry.register_chart_renderer(Box::new(TestChartRenderer));
    registry.register_diagram_renderer(Box::new(TestDiagramRenderer));
    registry.register_validator(Box::new(TestValidator));
    registry.register_math_renderer(Box::new(TestMathRenderer));
    registry.register_brand_provider(Box::new(TestBrandProvider));
    registry.register_slide_type(Box::new(TestSlideType));
    registry.register_section_type(Box::new(TestSectionType));
    registry.register_inline_format(Box::new(TestInlineFormat));

    assert!(registry.lookup_data_source("test").is_some());
    assert!(registry.lookup_exporter("test-export").is_some());
    assert!(registry.lookup_chart_renderer("test-chart").is_some());
    assert!(registry.lookup_diagram_renderer("test-diagram").is_some());
    assert!(registry.lookup_validator("test-validator").is_some());
    assert!(registry.lookup_math_renderer("test-math").is_some());
    assert!(registry.lookup_brand_provider("test-brand").is_some());
    assert!(registry.lookup_slide_type("test-slide").is_some());
    assert!(registry.lookup_section_type("test-section").is_some());
    assert!(registry.lookup_inline_format("test-inline").is_some());
}

/// AC-014: `PluginRegistry` is Send + Sync — compile-time assertion.
#[test]
fn dog_food_registry_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginRegistry>();
}
