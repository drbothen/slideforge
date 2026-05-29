//! [`PluginRegistry`] — the central container for all 10 plugin surfaces.
//!
//! The registry is constructed once at process start by the CLI entry point and
//! then shared immutably (wrapped in `Arc<PluginRegistry>`) across all pipeline
//! stages. Registration is synchronous and single-threaded during startup;
//! lookups are read-only and can occur from any thread.
//!
//! ## Design constraints
//!
//! - Stores `Box<dyn Trait + Send + Sync>` per surface — dynamic dispatch.
//! - Each surface uses `Vec<Box<dyn Trait>>` to allow multiple implementations.
//! - Lookups return `Option<&dyn Trait>` by trait `id()`.
//! - `PluginRegistry` is intentionally NOT `Clone` (`Box<dyn Trait>` is not
//!   `Clone`). Callers wrap in `Arc<PluginRegistry>` for shared ownership.
//! - No WASM loading or dynamic library loading in v1.0 — all plugins are
//!   statically compiled (plugin-architecture.md §v1.0 Plugin Loading).

use crate::traits::{
    BrandProvider, ChartRenderer, DataSource, DiagramRenderer, Exporter, InlineFormat,
    MathRenderer, SectionType, SlideType, Validator,
};

/// The central registry for all slideforge plugin implementations.
///
/// Constructed once at process start, then wrapped in `Arc<PluginRegistry>` and
/// shared across all pipeline stages. All registration methods take
/// `Box<dyn Trait + Send + Sync>` and append to the corresponding surface list.
///
/// ## Lookup semantics
///
/// - `lookup_*_by_id(id)` performs a linear search by `trait.id()` and returns
///   the first match.
/// - Multiple plugins with the same `id()` may be registered; the first
///   registered is returned on lookup (insertion order, EC-001).
/// - An empty registry returns `None` for all lookups (EC-003).
///
/// ## Thread safety
///
/// `PluginRegistry` is `Send + Sync` because all stored `Box<dyn Trait>` bounds
/// include `Send + Sync`. See the compile-time assertion in this module.
#[derive(Default)]
pub struct PluginRegistry {
    /// Registered data source plugins.
    data_sources: Vec<Box<dyn DataSource + Send + Sync>>,
    /// Registered exporter plugins.
    exporters: Vec<Box<dyn Exporter + Send + Sync>>,
    /// Registered chart renderer plugins.
    chart_renderers: Vec<Box<dyn ChartRenderer + Send + Sync>>,
    /// Registered diagram renderer plugins.
    diagram_renderers: Vec<Box<dyn DiagramRenderer + Send + Sync>>,
    /// Registered validator plugins.
    validators: Vec<Box<dyn Validator + Send + Sync>>,
    /// Registered math renderer plugins.
    math_renderers: Vec<Box<dyn MathRenderer + Send + Sync>>,
    /// Registered brand provider plugins.
    brand_providers: Vec<Box<dyn BrandProvider + Send + Sync>>,
    /// Registered slide type plugins.
    slide_types: Vec<Box<dyn SlideType + Send + Sync>>,
    /// Registered section type plugins.
    section_types: Vec<Box<dyn SectionType + Send + Sync>>,
    /// Registered inline format plugins.
    inline_formats: Vec<Box<dyn InlineFormat + Send + Sync>>,
}

// Compile-time assertion: PluginRegistry must be Send + Sync.
// If this function compiles, the assertion holds.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginRegistry>();
};

impl PluginRegistry {
    /// Create a new empty plugin registry.
    ///
    /// No plugins are registered by default. The CLI assembles the registry
    /// by calling each `register_*` method before passing it to the pipeline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 1: DataSource
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`DataSource`] plugin.
    ///
    /// Multiple data source plugins with different `id()` values may be
    /// registered. Registering two plugins with the same `id()` is allowed;
    /// the first is returned on lookup (EC-001).
    pub fn register_data_source(&mut self, plugin: Box<dyn DataSource + Send + Sync>) {
        self.data_sources.push(plugin);
    }

    /// Look up a [`DataSource`] by its `id()`.
    ///
    /// Returns the first registered plugin whose `id()` matches `id`, or `None`
    /// if no match is found (EC-002, EC-003).
    #[must_use]
    pub fn lookup_data_source(&self, id: &str) -> Option<&dyn DataSource> {
        self.data_sources
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn DataSource)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 2: Exporter
    // ──────────────────────────────────────────────────────────────────────────

    /// Register an [`Exporter`] plugin.
    pub fn register_exporter(&mut self, plugin: Box<dyn Exporter + Send + Sync>) {
        self.exporters.push(plugin);
    }

    /// Look up an [`Exporter`] by its `id()`.
    #[must_use]
    pub fn lookup_exporter(&self, id: &str) -> Option<&dyn Exporter> {
        self.exporters
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn Exporter)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 3: ChartRenderer
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`ChartRenderer`] plugin.
    pub fn register_chart_renderer(&mut self, plugin: Box<dyn ChartRenderer + Send + Sync>) {
        self.chart_renderers.push(plugin);
    }

    /// Look up a [`ChartRenderer`] by its `id()`.
    #[must_use]
    pub fn lookup_chart_renderer(&self, id: &str) -> Option<&dyn ChartRenderer> {
        self.chart_renderers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn ChartRenderer)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 4: DiagramRenderer
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`DiagramRenderer`] plugin.
    pub fn register_diagram_renderer(&mut self, plugin: Box<dyn DiagramRenderer + Send + Sync>) {
        self.diagram_renderers.push(plugin);
    }

    /// Look up a [`DiagramRenderer`] by its `id()`.
    #[must_use]
    pub fn lookup_diagram_renderer(&self, id: &str) -> Option<&dyn DiagramRenderer> {
        self.diagram_renderers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn DiagramRenderer)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 5: Validator
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`Validator`] plugin.
    pub fn register_validator(&mut self, plugin: Box<dyn Validator + Send + Sync>) {
        self.validators.push(plugin);
    }

    /// Look up a [`Validator`] by its `id()`.
    #[must_use]
    pub fn lookup_validator(&self, id: &str) -> Option<&dyn Validator> {
        self.validators
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn Validator)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 6: MathRenderer
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`MathRenderer`] plugin.
    pub fn register_math_renderer(&mut self, plugin: Box<dyn MathRenderer + Send + Sync>) {
        self.math_renderers.push(plugin);
    }

    /// Look up a [`MathRenderer`] by its `id()`.
    #[must_use]
    pub fn lookup_math_renderer(&self, id: &str) -> Option<&dyn MathRenderer> {
        self.math_renderers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn MathRenderer)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 7: BrandProvider
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`BrandProvider`] plugin.
    pub fn register_brand_provider(&mut self, plugin: Box<dyn BrandProvider + Send + Sync>) {
        self.brand_providers.push(plugin);
    }

    /// Look up a [`BrandProvider`] by its `id()`.
    #[must_use]
    pub fn lookup_brand_provider(&self, id: &str) -> Option<&dyn BrandProvider> {
        self.brand_providers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn BrandProvider)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 8: SlideType
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`SlideType`] plugin.
    pub fn register_slide_type(&mut self, plugin: Box<dyn SlideType + Send + Sync>) {
        self.slide_types.push(plugin);
    }

    /// Look up a [`SlideType`] by its keyword (`id()`).
    ///
    /// This is the primary lookup used by the layout engine when processing a
    /// `slide:` block. Returns the first registered slide type whose `id()`
    /// matches `keyword`.
    #[must_use]
    pub fn lookup_slide_type(&self, keyword: &str) -> Option<&dyn SlideType> {
        self.slide_types
            .iter()
            .find(|p| p.id() == keyword)
            .map(|p| p.as_ref() as &dyn SlideType)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 9: SectionType
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`SectionType`] plugin.
    pub fn register_section_type(&mut self, plugin: Box<dyn SectionType + Send + Sync>) {
        self.section_types.push(plugin);
    }

    /// Look up a [`SectionType`] by its `id()`.
    #[must_use]
    pub fn lookup_section_type(&self, id: &str) -> Option<&dyn SectionType> {
        self.section_types
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn SectionType)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 10: InlineFormat
    // ──────────────────────────────────────────────────────────────────────────

    /// Register an [`InlineFormat`] plugin.
    pub fn register_inline_format(&mut self, plugin: Box<dyn InlineFormat + Send + Sync>) {
        self.inline_formats.push(plugin);
    }

    /// Look up an [`InlineFormat`] by its `id()`.
    #[must_use]
    pub fn lookup_inline_format(&self, id: &str) -> Option<&dyn InlineFormat> {
        self.inline_formats
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn InlineFormat)
    }
}

#[cfg(test)]
#[allow(clippy::unnecessary_literal_bound)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::traits::{
        BrandError, BrandSource, ChartError, DataSourceError, DataSourceOptions, DiagramError,
        DiagramOptions, ExportError, ExportOptions, InlineError, InlineOutputFormat, LayoutError,
        MathError, MathOutputFormat, ValidatorOptions,
    };
    use slideforge_layout::{LaidOutDeck, LaidOutSlide, PageSize};
    use slideforge_types::{
        Brand, BrandFonts, BrandPalette, ChartSpec, Deck, DeckMetadata, InlineNode, MathNode,
        OrderedMap, Slide, SourceSpan, Value,
    };

    // ── Stub implementations for each surface ─────────────────────────────────

    struct StubDataSource;
    impl DataSource for StubDataSource {
        fn id(&self) -> &str {
            "stub-data"
        }
        fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
            Ok(Value::Null)
        }
    }

    struct StubExporter;
    impl Exporter for StubExporter {
        fn id(&self) -> &str {
            "stub-export"
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

    struct StubChartRenderer;
    impl ChartRenderer for StubChartRenderer {
        fn id(&self) -> &str {
            "stub-chart"
        }
        fn render(&self, _spec: &ChartSpec, _brand: &Brand) -> Result<Vec<u8>, ChartError> {
            Ok(vec![])
        }
    }

    struct StubDiagramRenderer;
    impl DiagramRenderer for StubDiagramRenderer {
        fn id(&self) -> &str {
            "stub-diagram"
        }
        fn render(&self, _source: &str, _opts: &DiagramOptions) -> Result<Vec<u8>, DiagramError> {
            Ok(vec![])
        }
    }

    struct StubValidator;
    impl Validator for StubValidator {
        fn id(&self) -> &str {
            "stub-validator"
        }
        fn validate(
            &self,
            _deck: &Deck,
            _opts: &ValidatorOptions,
        ) -> Vec<crate::traits::Diagnostic> {
            vec![]
        }
    }

    struct StubMathRenderer;
    impl MathRenderer for StubMathRenderer {
        fn id(&self) -> &str {
            "stub-math"
        }
        fn render(
            &self,
            _node: &MathNode,
            _format: MathOutputFormat,
        ) -> Result<Vec<u8>, MathError> {
            Ok(vec![])
        }
    }

    struct StubBrandProvider;
    impl BrandProvider for StubBrandProvider {
        fn id(&self) -> &str {
            "stub-brand"
        }
        fn load(&self, _source: &BrandSource) -> Result<Brand, BrandError> {
            Ok(Brand {
                name: std::sync::Arc::from("stub"),
                palette: BrandPalette {
                    primary: std::sync::Arc::from("#000000"),
                    secondary: std::sync::Arc::from("#ffffff"),
                    accent: std::sync::Arc::from("#ff0000"),
                    neutral: std::sync::Arc::from("#888888"),
                },
                fonts: BrandFonts {
                    heading: std::sync::Arc::from("Arial"),
                    body: std::sync::Arc::from("Arial"),
                    mono: std::sync::Arc::from("Courier"),
                },
                layouts: vec![],
                span: SourceSpan::default(),
            })
        }
    }

    struct StubSlideType;
    impl SlideType for StubSlideType {
        fn id(&self) -> &'static str {
            "stub-slide"
        }
        fn required_fields(&self) -> &[crate::traits::FieldDef] {
            &[]
        }
        fn optional_fields(&self) -> &[crate::traits::FieldDef] {
            &[]
        }
        fn layout_name(&self) -> &'static str {
            "blank"
        }
        fn lay_out(
            &self,
            slide: &Slide,
            _brand: &Brand,
            _canvas: crate::traits::Canvas,
        ) -> Result<LaidOutSlide, LayoutError> {
            Ok(LaidOutSlide {
                source_index: 0,
                slide_type_keyword: std::sync::Arc::clone(&slide.slide_type),
                frames: vec![],
                speaker_notes: None,
                register_tags: vec![],
            })
        }
    }

    struct StubSectionType;
    impl SectionType for StubSectionType {
        fn id(&self) -> &str {
            "stub-section"
        }
        fn generate(&self, _slides: &[Slide]) -> Vec<crate::traits::SectionBlock> {
            vec![]
        }
    }

    struct StubInlineFormat;
    impl InlineFormat for StubInlineFormat {
        fn id(&self) -> &str {
            "stub-inline"
        }
        fn render(
            &self,
            _node: &InlineNode,
            _format: InlineOutputFormat,
        ) -> Result<String, InlineError> {
            Ok(String::new())
        }
    }

    // ── Helper to build a minimal Deck for tests ───────────────────────────────

    fn stub_deck() -> Deck {
        Deck {
            slides: vec![],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: None,
                slideforge_version: std::sync::Arc::from("0.1.0"),
                lang: None,
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        }
    }

    fn stub_laid_out_deck() -> LaidOutDeck {
        LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![],
            sections: vec![],
            warnings: vec![],
        }
    }

    fn stub_brand() -> Brand {
        Brand {
            name: std::sync::Arc::from("test"),
            palette: BrandPalette {
                primary: std::sync::Arc::from("#003087"),
                secondary: std::sync::Arc::from("#0066CC"),
                accent: std::sync::Arc::from("#FF6B35"),
                neutral: std::sync::Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: std::sync::Arc::from("Calibri Light"),
                body: std::sync::Arc::from("Calibri"),
                mono: std::sync::Arc::from("Courier New"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    // ── AC-003 (EC-003): empty registry returns None for all lookups ───────────

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_data_source_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_data_source("anything").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_exporter_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_exporter("pptx").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_chart_renderer_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_chart_renderer("plotters").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_diagram_renderer_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_diagram_renderer("mermaid").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_validator_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_validator("overflow").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_math_renderer_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_math_renderer("pulldown-latex").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_brand_provider_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_brand_provider("default").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_slide_type_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_slide_type("title").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_section_type_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_section_type("toc").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_inline_format_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_inline_format("default").is_none());
    }

    // ── Register-then-lookup round trips for all 10 surfaces ─────────────────

    #[test]
    fn test_bc_5_02_001_register_and_lookup_data_source() {
        let mut registry = PluginRegistry::new();
        registry.register_data_source(Box::new(StubDataSource));
        let plugin = registry.lookup_data_source("stub-data");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().id(), "stub-data");
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_exporter() {
        let mut registry = PluginRegistry::new();
        registry.register_exporter(Box::new(StubExporter));
        let plugin = registry.lookup_exporter("stub-export");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().extension(), "bin");
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_chart_renderer() {
        let mut registry = PluginRegistry::new();
        registry.register_chart_renderer(Box::new(StubChartRenderer));
        let plugin = registry.lookup_chart_renderer("stub-chart");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_diagram_renderer() {
        let mut registry = PluginRegistry::new();
        registry.register_diagram_renderer(Box::new(StubDiagramRenderer));
        let plugin = registry.lookup_diagram_renderer("stub-diagram");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_validator() {
        let mut registry = PluginRegistry::new();
        registry.register_validator(Box::new(StubValidator));
        let plugin = registry.lookup_validator("stub-validator");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_math_renderer() {
        let mut registry = PluginRegistry::new();
        registry.register_math_renderer(Box::new(StubMathRenderer));
        let plugin = registry.lookup_math_renderer("stub-math");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_brand_provider() {
        let mut registry = PluginRegistry::new();
        registry.register_brand_provider(Box::new(StubBrandProvider));
        let plugin = registry.lookup_brand_provider("stub-brand");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_slide_type() {
        let mut registry = PluginRegistry::new();
        registry.register_slide_type(Box::new(StubSlideType));
        let plugin = registry.lookup_slide_type("stub-slide");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().layout_name(), "blank");
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_section_type() {
        let mut registry = PluginRegistry::new();
        registry.register_section_type(Box::new(StubSectionType));
        let plugin = registry.lookup_section_type("stub-section");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_inline_format() {
        let mut registry = PluginRegistry::new();
        registry.register_inline_format(Box::new(StubInlineFormat));
        let plugin = registry.lookup_inline_format("stub-inline");
        assert!(plugin.is_some());
    }

    // ── AC-015: all 10 surfaces registered and accessible ────────────────────

    #[test]
    fn test_bc_5_02_001_all_10_surfaces_register_and_lookup() {
        let mut registry = PluginRegistry::new();

        registry.register_data_source(Box::new(StubDataSource));
        registry.register_exporter(Box::new(StubExporter));
        registry.register_chart_renderer(Box::new(StubChartRenderer));
        registry.register_diagram_renderer(Box::new(StubDiagramRenderer));
        registry.register_validator(Box::new(StubValidator));
        registry.register_math_renderer(Box::new(StubMathRenderer));
        registry.register_brand_provider(Box::new(StubBrandProvider));
        registry.register_slide_type(Box::new(StubSlideType));
        registry.register_section_type(Box::new(StubSectionType));
        registry.register_inline_format(Box::new(StubInlineFormat));

        assert!(registry.lookup_data_source("stub-data").is_some());
        assert!(registry.lookup_exporter("stub-export").is_some());
        assert!(registry.lookup_chart_renderer("stub-chart").is_some());
        assert!(registry.lookup_diagram_renderer("stub-diagram").is_some());
        assert!(registry.lookup_validator("stub-validator").is_some());
        assert!(registry.lookup_math_renderer("stub-math").is_some());
        assert!(registry.lookup_brand_provider("stub-brand").is_some());
        assert!(registry.lookup_slide_type("stub-slide").is_some());
        assert!(registry.lookup_section_type("stub-section").is_some());
        assert!(registry.lookup_inline_format("stub-inline").is_some());
    }

    // ── EC-001: duplicate id — first registration returned ────────────────────

    struct StubDataSourceDuplicate;
    impl DataSource for StubDataSourceDuplicate {
        fn id(&self) -> &str {
            "stub-data" // same id as StubDataSource
        }
        fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
            Ok(Value::Int(42)) // returns different value to detect which plugin is used
        }
    }

    #[test]
    fn test_bc_5_02_001_ec001_duplicate_id_first_returned() {
        let mut registry = PluginRegistry::new();
        registry.register_data_source(Box::new(StubDataSource));
        registry.register_data_source(Box::new(StubDataSourceDuplicate));

        // Both are stored; first registered (StubDataSource) is returned on lookup.
        let plugin = registry.lookup_data_source("stub-data").unwrap();
        let result = plugin
            .load("x", &DataSourceOptions::default())
            .expect("load should succeed");
        // StubDataSource returns Null; StubDataSourceDuplicate returns Int(42).
        // The first registration should win.
        assert_eq!(result, Value::Null);
    }

    // ── Functional call-through tests ─────────────────────────────────────────

    #[test]
    fn test_bc_5_02_001_exporter_can_export_after_lookup() {
        let mut registry = PluginRegistry::new();
        registry.register_exporter(Box::new(StubExporter));
        let exporter = registry.lookup_exporter("stub-export").unwrap();
        let deck = stub_deck();
        let laid_out = stub_laid_out_deck();
        let brand = stub_brand();
        let result = exporter.export(&deck, &laid_out, &brand, &ExportOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_bc_5_02_001_math_renderer_can_render_after_lookup() {
        let mut registry = PluginRegistry::new();
        registry.register_math_renderer(Box::new(StubMathRenderer));
        let renderer = registry.lookup_math_renderer("stub-math").unwrap();
        let node = MathNode {
            latex: std::sync::Arc::from("x^2"),
            display: false,
            span: SourceSpan::default(),
        };
        let result = renderer.render(&node, MathOutputFormat::MathMl);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bc_5_02_001_registry_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PluginRegistry>();
    }
}
