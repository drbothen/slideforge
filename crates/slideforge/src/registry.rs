//! Bundled plugin registration — `register_bundled_plugins` and `default_registry`.
//!
//! This module is the **composition root** for all 10 plugin surfaces. It calls
//! `PluginRegistryBuilder::register_*()` for every bundled plugin implementation
//! and then finalizes the registry with `build()`.
//!
//! ## Architecture compliance
//!
//! - No plugin logic lives here — only registration calls.
//! - The `PluginRegistryBuilder` type is defined in `slideforge-plugin-api`
//!   (STORY-083); this module only CALLS it.
//! - All `register_*` calls use `Box<dyn Trait>` dispatch — the compiler
//!   enforces the dog-fooding guarantee (BC-5.02.002) because the builder API
//!   accepts only `Box<dyn Trait + Send + Sync>`, not concrete struct types.
//!
//! ## Traceability
//!
//! - BC-5.02.001 postcondition 1: all 10 surfaces registered
//! - BC-5.02.002 invariant 1: no private function calls across crate boundaries
//! - STORY-049 AC-001, AC-003, AC-004

use slideforge_plugin_api::{PluginRegistry, PluginRegistryBuilder, RegistryError};

// ── Surface 1: DataSource ─────────────────────────────────────────────────────
use slideforge_data::{FileDataSource, HttpDataSource, SqliteDataSource, XlsxDataSource};

// ── Surface 2: Exporter ───────────────────────────────────────────────────────
use slideforge_docx::DocxExporter;
use slideforge_pdf::PdfExporter;
use slideforge_pptx::PptxExporter;
// Note: HtmlExporter is deferred to STORY-050 (slideforge-html excluded from workspace, Phase 4).

// ── Surface 3: ChartRenderer ──────────────────────────────────────────────────
use slideforge_charts::ChartRendererImpl;

// ── Surface 4: DiagramRenderer ────────────────────────────────────────────────
use slideforge_diagrams::DiagramRendererImpl;

// ── Surface 5: Validator ──────────────────────────────────────────────────────
use slideforge_validate::{
    AltTextValidator, CanvasOverflowValidator, LabelCheckValidator, LangValidator,
    ValidationConfig, ZeroSlideValidator,
};

// ── Surface 6: MathRenderer ───────────────────────────────────────────────────
use slideforge_math::MathRendererImpl;

// ── Surface 7: BrandProvider ──────────────────────────────────────────────────
use slideforge_brand::{BrandLoader, BrandSynthesizer};

// ── Surfaces 8, 9, 10: SlideType, SectionType, InlineFormat ──────────────────
// All three surface implementations live in slideforge-plugin-api.
use slideforge_plugin_api::slide_types::{
    agenda::AgendaSlideType, bio::BioSlideType, blank::BlankSlideType, chart::ChartSlideType,
    closing::ClosingSlideType, code_sample::CodeSampleSlideType, comparison::ComparisonSlideType,
    content::ContentSlideType, diagram::DiagramSlideType,
    executive_summary::ExecutiveSummarySlideType, financials::FinancialsSlideType,
    image::ImageSlideType, kpi_dashboard::KpiDashboardSlideType, matrix::MatrixSlideType,
    org_chart::OrgChartSlideType, problem_statement::ProblemStatementSlideType,
    process_flow::ProcessFlowSlideType, quote::QuoteSlideType,
    recommendation::RecommendationSlideType, risk_register::RiskRegisterSlideType,
    roadmap::RoadmapSlideType, screenshot::ScreenshotSlideType,
    section_break::SectionBreakSlideType, stat_callout::StatCalloutSlideType,
    survey_results::SurveyResultsSlideType, team::TeamSlideType, timeline::TimelineSlideType,
    title::TitleSlideType, toc::TocSlideType, two_col::TwoColSlideType, video::VideoSlideType,
};
use slideforge_plugin_api::{
    AppendixSectionType, ApprovalSectionType, DefaultInlineFormat, ExecutiveSummarySectionType,
    GlossarySectionType, MethodologySectionType, RiskRegisterSectionType, ScopeSectionType,
};

/// Register all 10 bundled plugin surfaces into `builder`.
///
/// Adds every bundled plugin implementation to the corresponding surface
/// via the public `register_*` API. Does not call `build()` — the caller
/// decides when to finalize.
///
/// After this call, `builder` has at least one plugin registered for each
/// of the 10 required surfaces. Calling `builder.build()` will return
/// `Ok(PluginRegistry)`.
///
/// ## Traceability
///
/// - BC-5.02.001 postcondition 1: all 10 surfaces registered
/// - STORY-049 AC-001
pub fn register_bundled_plugins(builder: &mut PluginRegistryBuilder) {
    // ── Surface 1: DataSource (4 bundled implementations) ────────────────────
    // JSON, CSV, YAML, TOML via file extension detection (FileDataSource);
    // HTTP remote sources; XLSX spreadsheets; SQLite queries.
    // Default base path is "." — callers configure paths at runtime.
    builder.register_data_source(Box::new(FileDataSource::new(".")));
    builder.register_data_source(Box::new(HttpDataSource::new("")));
    builder.register_data_source(Box::new(XlsxDataSource::new("")));
    builder.register_data_source(Box::new(SqliteDataSource::new("", "")));

    // ── Surface 2: Exporter (3 bundled implementations; HTML deferred to STORY-050) ──
    builder.register_exporter(Box::new(PptxExporter::new()));
    builder.register_exporter(Box::new(DocxExporter));
    builder.register_exporter(Box::new(PdfExporter::new()));

    // ── Surface 3: ChartRenderer (1 bundled implementation — plotters-backed) ──
    builder.register_chart_renderer(Box::new(ChartRendererImpl::new()));

    // ── Surface 4: DiagramRenderer (1 bundled implementation — mermaid-rs) ──
    builder.register_diagram_renderer(Box::new(DiagramRendererImpl::new()));

    // ── Surface 5: Validator (5 bundled implementations) ─────────────────────
    builder.register_validator(Box::new(AltTextValidator));
    builder.register_validator(Box::new(ZeroSlideValidator));
    builder.register_validator(Box::new(CanvasOverflowValidator::from_config(
        &ValidationConfig::default(),
    )));
    builder.register_validator(Box::new(LabelCheckValidator));
    builder.register_validator(Box::new(LangValidator));

    // ── Surface 6: MathRenderer (1 bundled implementation — pulldown-latex) ──
    builder.register_math_renderer(Box::new(MathRendererImpl::new()));

    // ── Surface 7: BrandProvider (2 bundled implementations) ─────────────────
    // BrandLoader: file-based provider (loads from .toml brand config).
    // BrandSynthesizer: synthesizes brand config from minimal inputs.
    builder.register_brand_provider(Box::new(BrandLoader::new()));
    builder.register_brand_provider(Box::new(BrandSynthesizer));

    // ── Surface 8: SlideType (31 bundled implementations) ────────────────────
    builder.register_slide_type(Box::new(TitleSlideType::new()));
    builder.register_slide_type(Box::new(SectionBreakSlideType::new()));
    builder.register_slide_type(Box::new(ContentSlideType::new()));
    builder.register_slide_type(Box::new(TwoColSlideType::new()));
    builder.register_slide_type(Box::new(ImageSlideType::new()));
    builder.register_slide_type(Box::new(BlankSlideType::new()));
    builder.register_slide_type(Box::new(AgendaSlideType::new()));
    builder.register_slide_type(Box::new(TocSlideType::new()));
    builder.register_slide_type(Box::new(QuoteSlideType::new()));
    builder.register_slide_type(Box::new(TeamSlideType::new()));
    builder.register_slide_type(Box::new(BioSlideType::new()));
    builder.register_slide_type(Box::new(ExecutiveSummarySlideType::new()));
    builder.register_slide_type(Box::new(ProblemStatementSlideType::new()));
    builder.register_slide_type(Box::new(RecommendationSlideType::new()));
    builder.register_slide_type(Box::new(RiskRegisterSlideType::new()));
    builder.register_slide_type(Box::new(TimelineSlideType::new()));
    builder.register_slide_type(Box::new(StatCalloutSlideType::new()));
    builder.register_slide_type(Box::new(ComparisonSlideType::new()));
    builder.register_slide_type(Box::new(ProcessFlowSlideType::new()));
    builder.register_slide_type(Box::new(MatrixSlideType::new()));
    builder.register_slide_type(Box::new(FinancialsSlideType::new()));
    builder.register_slide_type(Box::new(KpiDashboardSlideType::new()));
    builder.register_slide_type(Box::new(ChartSlideType::new()));
    builder.register_slide_type(Box::new(DiagramSlideType::new()));
    builder.register_slide_type(Box::new(ScreenshotSlideType::new()));
    builder.register_slide_type(Box::new(CodeSampleSlideType::new()));
    builder.register_slide_type(Box::new(VideoSlideType::new()));
    builder.register_slide_type(Box::new(SurveyResultsSlideType::new()));
    builder.register_slide_type(Box::new(OrgChartSlideType::new()));
    builder.register_slide_type(Box::new(RoadmapSlideType::new()));
    builder.register_slide_type(Box::new(ClosingSlideType::new()));

    // ── Surface 9: SectionType (7 bundled implementations) ───────────────────
    // All 7 section types are unit structs from slideforge-plugin-api/src/section_types/.
    builder.register_section_type(Box::new(ExecutiveSummarySectionType));
    builder.register_section_type(Box::new(RiskRegisterSectionType));
    builder.register_section_type(Box::new(MethodologySectionType));
    builder.register_section_type(Box::new(ScopeSectionType));
    builder.register_section_type(Box::new(ApprovalSectionType));
    builder.register_section_type(Box::new(AppendixSectionType));
    builder.register_section_type(Box::new(GlossarySectionType));

    // ── Surface 10: InlineFormat (1 bundled implementation) ──────────────────
    // DefaultInlineFormat covers all 12 InlineNode variants × 3 output formats.
    builder.register_inline_format(Box::new(DefaultInlineFormat));
}

/// Build a fully-assembled `PluginRegistry` with all bundled plugins registered.
///
/// Equivalent to:
/// ```rust,no_run
/// # use slideforge::registry::register_bundled_plugins;
/// # use slideforge_plugin_api::{PluginRegistry, PluginRegistryBuilder, RegistryError};
/// let mut builder = PluginRegistryBuilder::default();
/// register_bundled_plugins(&mut builder);
/// let registry = builder.build()?;
/// # Ok::<PluginRegistry, RegistryError>(registry)
/// ```
///
/// ## Errors
///
/// Returns [`RegistryError::MissingSurface`] if any required surface has no
/// registrations. This should never happen with bundled plugins but is
/// possible if `register_bundled_plugins` is patched incorrectly.
///
/// ## Traceability
///
/// - STORY-049 AC-001: `default_registry().surface_count() == 10`
pub fn default_registry() -> Result<PluginRegistry, RegistryError> {
    let mut builder = PluginRegistryBuilder::default();
    register_bundled_plugins(&mut builder);
    builder.build()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use slideforge_plugin_api::SURFACE_NAMES;

    // ── AC-001: register_bundled_plugins + build → Ok(registry) with surface_count == 10 ──

    /// STORY-049 AC-001 / BC-5.02.001 postcondition 1:
    /// `register_bundled_plugins` followed by `builder.build()` returns
    /// `Ok(registry)` where `registry.surface_count() == 10`.
    ///
    /// This test FAILS on todo!() bodies — the real assertion requires
    /// `register_bundled_plugins` to actually call all 10 `register_*` methods.
    #[test]
    fn test_bc_5_02_001_register_bundled_plugins_build_returns_ok_with_10_surfaces() {
        let mut builder = PluginRegistryBuilder::default();
        register_bundled_plugins(&mut builder);
        let registry = builder
            .build()
            .expect("register_bundled_plugins must register all 10 surfaces");

        assert_eq!(
            registry.surface_count(),
            10,
            "AC-001: surface_count() must be 10 after register_bundled_plugins; \
             got: {}",
            registry.surface_count()
        );
    }

    /// STORY-049 AC-001: each surface has at least 1 implementation.
    ///
    /// Verifies via `surface_names()` that all 10 canonical surface names are
    /// present in declaration order.
    #[test]
    fn test_bc_5_02_001_register_bundled_plugins_all_10_surface_names_present() {
        let mut builder = PluginRegistryBuilder::default();
        register_bundled_plugins(&mut builder);
        let registry = builder
            .build()
            .expect("register_bundled_plugins must register all 10 surfaces");

        let names = registry.surface_names();
        assert_eq!(
            names.len(),
            10,
            "AC-001: surface_names() must return 10 names; got: {names:?}"
        );

        let expected: Vec<&'static str> = SURFACE_NAMES.to_vec();
        assert_eq!(
            names, expected,
            "AC-001: surface_names() must return canonical SURFACE_NAMES in declaration order"
        );
    }

    // ── AC-001 via default_registry() convenience function ────────────────────

    /// STORY-049 AC-001: `default_registry()` returns `Ok(registry)` with all
    /// 10 surfaces registered.
    #[test]
    fn test_bc_5_02_001_default_registry_returns_ok_with_10_surfaces() {
        let registry =
            default_registry().expect("default_registry() must return Ok(PluginRegistry)");

        assert_eq!(
            registry.surface_count(),
            10,
            "AC-001: default_registry() surface_count must be 10; got: {}",
            registry.surface_count()
        );
    }

    // ── AC-002: empty builder returns Err(MissingSurface) — not panic, not Ok ──

    /// STORY-049 AC-002 / BC-5.02.001 invariant 3:
    /// `PluginRegistryBuilder::default().build()` (with zero registrations) returns
    /// `Err(RegistryError::MissingSurface { .. })`, exercised from the root crate.
    ///
    /// This test verifies the behavior works end-to-end from the root crate's
    /// perspective (STORY-083 already tests this in slideforge-plugin-api; this
    /// test confirms the re-exported error path works from slideforge).
    #[test]
    fn test_bc_5_02_001_empty_builder_returns_err_missing_surface_from_root_crate() {
        use slideforge_plugin_api::RegistryError;

        let result = PluginRegistryBuilder::default().build();
        // PluginRegistry does not implement Debug, so we cannot use {result:?}.
        // Instead, check the specific error variant pattern.
        let is_missing_surface = matches!(result, Err(RegistryError::MissingSurface { .. }));
        assert!(
            is_missing_surface,
            "AC-002: empty PluginRegistryBuilder::default().build() must return Err(MissingSurface)"
        );
    }

    // ── AC-004: surface_count() == 10 and surface_names() == canonical names ──

    /// STORY-049 AC-004 / BC-5.02.001 postcondition 2:
    /// the fully-assembled registry has `surface_count() == 10` and
    /// `surface_names()` equals the 10 canonical names in declaration order.
    #[test]
    fn test_bc_5_02_001_surface_count_and_surface_names_on_default_registry() {
        let registry =
            default_registry().expect("default_registry() must return Ok(PluginRegistry)");

        // AC-004a: surface_count()
        assert_eq!(
            registry.surface_count(),
            10,
            "AC-004: surface_count() must be 10"
        );

        // AC-004b: surface_names() in canonical declaration order
        let names = registry.surface_names();
        let expected = vec![
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
        assert_eq!(
            names, expected,
            "AC-004: surface_names() must return canonical names in SURFACE_NAMES order; \
             got: {names:?}"
        );
    }

    // ── EC-004: two bundled plugins for same surface — both registered ─────────

    /// BC-5.02.001 EC-004: multiple plugins registered for the same surface
    /// (e.g., `BrandLoader` + `BrandSynthesizer` for `BrandProvider`) must both be
    /// accessible via their canonical ids.
    ///
    /// - `BrandLoader` id = `"slideforge-brand/default"` — handles `TomlFile`,
    ///   `PptxFile`, and `DocxFile` `BrandSource` variants.
    /// - `BrandSynthesizer` id = `"slideforge-brand-synthesizer"` — handles
    ///   `TomlFile` only (synthesizes from toml config); `PptxFile`/`DocxFile` return
    ///   `Err(NotImplemented)`.
    ///
    /// Both must be `Some` after `register_bundled_plugins` to prove EC-004
    /// (multi-registration on one surface).
    #[test]
    fn test_bc_5_02_001_ec004_multiple_brand_providers_registered() {
        let registry =
            default_registry().expect("default_registry() must return Ok(PluginRegistry)");

        // EC-004a: BrandLoader must be accessible by its canonical id.
        let brand_loader = registry.lookup_brand_provider("slideforge-brand/default");
        assert!(
            brand_loader.is_some(),
            "EC-004: BrandLoader must be accessible via id 'slideforge-brand/default'; \
             returned None — check register_bundled_plugins BrandProvider registrations"
        );

        // EC-004b: BrandSynthesizer must be accessible by its canonical id.
        let brand_synthesizer = registry.lookup_brand_provider("slideforge-brand-synthesizer");
        assert!(
            brand_synthesizer.is_some(),
            "EC-004: BrandSynthesizer must be accessible via id 'slideforge-brand-synthesizer'; \
             returned None — check register_bundled_plugins BrandProvider registrations"
        );

        // surface_count must still be 10 (counts surfaces, not plugins).
        assert_eq!(
            registry.surface_count(),
            10,
            "EC-004: surface_count must remain 10 after registering 2 BrandProviders \
             (surface count, not plugin count)"
        );
    }

    // ── M4: assert bundled DataSource ids (EC-002 regression guard) ──────────

    /// BC-5.02.001 EC-002 regression guard: the four bundled `DataSource`
    /// plugins must remain accessible by their canonical ids after
    /// `register_bundled_plugins`. Silently dropping one (e.g. by renaming
    /// the plugin or missing a registration) would cause EC-002 failures at
    /// runtime without triggering a compilation error.
    ///
    /// Canonical bundled ids:
    /// - `"file"` — `FileDataSource` (JSON, CSV, YAML, TOML by extension)
    /// - `"http"` — `HttpDataSource` (HTTP/HTTPS remote sources)
    /// - `"xlsx"` — `XlsxDataSource` (Excel spreadsheets)
    /// - `"sqlite"` — `SqliteDataSource` (`SQLite` query results)
    #[test]
    fn test_bc_5_02_001_m4_bundled_data_source_ids_all_registered() {
        let registry =
            default_registry().expect("default_registry() must return Ok(PluginRegistry)");

        let expected_ids = ["file", "http", "xlsx", "sqlite"];
        for id in &expected_ids {
            assert!(
                registry.lookup_data_source(id).is_some(),
                "M4: bundled DataSource with id '{id}' must be registered; \
                 lookup_data_source returned None — \
                 check register_bundled_plugins DataSource registrations"
            );
        }
    }

    // ── EC-003: partial builder (only DataSource registered) → MissingSurface ──

    /// BC-5.02.001 EC-003 edge case: a builder with only 1-of-10 surfaces
    /// registered must return `Err(MissingSurface)` naming the second surface
    /// in declaration order ("Exporter").
    #[test]
    #[allow(unreachable_patterns)]
    fn test_bc_5_02_001_partial_builder_missing_exporter_reports_error() {
        use slideforge_plugin_api::RegistryError;

        let mut builder = PluginRegistryBuilder::default();
        // Register only DataSource (surface 0) — Exporter (surface 1) is missing.
        builder.register_data_source(Box::new(FileDataSource::new(".")));

        let result = builder.build();
        match result {
            Err(RegistryError::MissingSurface { surface }) => {
                assert_eq!(
                    surface, "Exporter",
                    "EC-003: first missing surface after DataSource must be Exporter; got: {surface:?}"
                );
            },
            Ok(_) => panic!("partial builder must return Err(MissingSurface), got Ok"),
            Err(other) => {
                panic!("expected MissingSurface variant, got: {other:?}")
            },
        }
    }
}
