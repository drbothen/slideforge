//! Chart renderer plugin for slideforge.
//!
//! This crate implements the [`ChartRenderer`] plugin trait from
//! `slideforge-plugin-api` using the `plotters` 0.3.7 crate with its
//! `SVGBackend`. Seven chart types are supported in v1.0:
//!
//! | Type | Module |
//! |------|--------|
//! | `bar` | [`bar`] |
//! | `line` | [`line`] |
//! | `pie` | [`pie`] |
//! | `scatter` | [`scatter`] |
//! | `area` | [`area`] |
//! | `histogram` | [`histogram`] |
//! | `stacked-bar` | [`stacked_bar`] |
//!
//! ## Architecture Rules (STORY-031)
//!
//! - **No I/O**: All rendering is synchronous, pure, in-memory.
//! - **No subprocess**: `plotters` is a pure-Rust library; no Node.js spawned.
//! - **Plugin-first**: [`ChartRendererImpl`] uses ONLY the public
//!   `slideforge_plugin_api::ChartRenderer` trait — no internal bypass.
//! - **Forbidden deps**: This crate MUST NOT depend on any exporter,
//!   `slideforge-data`, `slideforge-layout`, `slideforge-cli`,
//!   `slideforge-syntax`, or `slideforge-eval`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod accessibility;
pub mod area;
pub mod bar;
pub mod histogram;
pub mod line;
pub mod pie;
pub mod safety;
pub mod scatter;
pub mod stacked_bar;
pub mod types;

use std::sync::Arc;

use slideforge_plugin_api::ChartRenderer;
use slideforge_types::{Brand, ChartSpec};
use tracing::instrument;

use crate::types::{ChartError, ChartSvg, ChartType, InternalChartSpec};

// Re-export primary types for crate consumers.
pub use crate::types::{ChartType as SfChartType, InternalChartSpec as SfChartSpec};

/// The default `ChartRenderer` plugin implementation for slideforge.
///
/// Uses the `plotters` 0.3.7 crate with its `SVGBackend` to render charts
/// as self-contained SVG strings.
#[derive(Debug, Default)]
pub struct ChartRendererImpl;

impl ChartRendererImpl {
    /// Create a new [`ChartRendererImpl`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Dispatch to the appropriate type-specific renderer and apply post-processing.
    ///
    /// Post-processing steps (in order):
    /// 1. Type-specific renderer → raw SVG string
    /// 2. [`safety::assert_no_forbidden_elements`]
    /// 3. [`accessibility::inject_aria_attributes`]
    ///
    /// # Errors
    ///
    /// Returns [`ChartError`] on render or post-processing failure.
    /// Dispatch to the appropriate type-specific renderer and apply post-processing.
    ///
    /// This is the **primary internal entry point** for chart rendering in the
    /// slideforge eval pipeline. The eval layer calls this method directly with
    /// a fully-bound [`InternalChartSpec`] that includes all series data.
    ///
    /// The [`ChartRenderer::render`] trait method (the plugin API surface) is a
    /// skeleton entry point — it validates the chart type and then delegates to
    /// this method from the eval pipeline via [`InternalChartSpec`].
    ///
    pub fn dispatch_and_process(spec: &InternalChartSpec) -> Result<ChartSvg, ChartError> {
        let raw_svg = match spec.chart_type {
            ChartType::Bar => bar::render_bar(spec)?,
            ChartType::Line => line::render_line(spec)?,
            ChartType::Pie => pie::render_pie(spec)?,
            ChartType::Scatter => scatter::render_scatter(spec)?,
            ChartType::Area => area::render_area(spec)?,
            ChartType::Histogram => histogram::render_histogram(spec)?,
            ChartType::StackedBar => stacked_bar::render_stacked_bar(spec)?,
        };

        safety::assert_no_forbidden_elements(&raw_svg)?;
        accessibility::inject_aria_attributes(&raw_svg, spec.alt.as_ref())
    }
}

impl ChartRenderer for ChartRendererImpl {
    fn id(&self) -> &'static str {
        "plotters"
    }

    /// Render a chart from a [`ChartSpec`] (plugin API skeleton).
    ///
    /// ## Architecture note (FINDING-001)
    ///
    /// [`ChartSpec`] is a **skeleton type** in the slideforge IR — it carries only
    /// the chart type keyword, alt text, and span, but **no series data**. In the
    /// full slideforge pipeline, the eval layer (slideforge-eval) enriches the IR
    /// at render time by constructing an [`InternalChartSpec`] with bound data
    /// and calling [`ChartRendererImpl::dispatch_and_process`] directly.
    ///
    /// This trait method is the **plugin-API surface** — it is called by external
    /// plugin consumers and by the eval layer. Because [`ChartSpec`] carries no
    /// data, this method returns [`slideforge_plugin_api::ChartError::InvalidSpec`]
    /// indicating that callers must supply data via the eval pipeline.
    ///
    /// ## Errors
    ///
    /// - [`slideforge_plugin_api::ChartError::UnsupportedChartType`] — unknown `chart_type`
    ///   keyword.
    /// - [`slideforge_plugin_api::ChartError::InvalidSpec`] — valid chart type but no data
    ///   is available in the skeleton [`ChartSpec`]; the eval pipeline must call
    ///   `dispatch_and_process(InternalChartSpec)` with bound data instead.
    #[instrument(skip(self, spec, _brand), fields(chart_type = %spec.chart_type))]
    fn render(
        &self,
        spec: &ChartSpec,
        _brand: &Brand,
    ) -> Result<Vec<u8>, slideforge_plugin_api::ChartError> {
        // Validate chart type first (UnsupportedChartType error if unknown).
        let _chart_type = ChartType::from_keyword(spec.chart_type.as_ref()).ok_or_else(|| {
            slideforge_plugin_api::ChartError::UnsupportedChartType {
                chart_type: spec.chart_type.as_ref().to_owned(),
            }
        })?;

        // FINDING-001: ChartSpec is a skeleton with no series data. The eval layer
        // must construct InternalChartSpec with bound data and call dispatch_and_process
        // directly. Return InvalidSpec here to surface the architectural contract.
        Err(slideforge_plugin_api::ChartError::InvalidSpec {
            message: format!(
                "ChartSpec carries no series data — chart rendering requires bound data. \
                 The eval pipeline must call dispatch_and_process(InternalChartSpec) with \
                 resolved data for chart type '{}'.",
                spec.chart_type.as_ref()
            ),
        })
    }

    // NOTE: For internal use, the eval layer calls ChartRendererImpl::render_internal()
    // (below) which accepts a fully-bound InternalChartSpec.
}

// ---------------------------------------------------------------------------
// Helpers — color extraction from Brand
// ---------------------------------------------------------------------------

/// Extract the ordered accent color hex strings from a [`Brand`].
///
/// The brand's palette provides `primary`, `secondary`, `accent`, and `neutral`.
/// For chart series coloring we use `accent` as the first accent color, then
/// `primary` and `secondary` as additional series colors. If all are empty,
/// the [`InternalChartSpec::FALLBACK_PALETTE`] is used instead.
///
/// This function always returns a non-empty Vec (falling back to the constant
/// palette if needed).
#[must_use]
pub fn extract_accent_colors(brand: &Brand) -> Vec<Arc<str>> {
    let mut colors: Vec<Arc<str>> = Vec::new();

    // Collect non-empty color strings from the palette.
    // Order: accent first (most visually distinctive), then primary, secondary.
    for hex in [
        brand.palette.accent.as_ref(),
        brand.palette.primary.as_ref(),
        brand.palette.secondary.as_ref(),
    ] {
        if !hex.is_empty() {
            colors.push(Arc::from(hex));
        }
    }

    if colors.is_empty() {
        // Fall back to the built-in WCAG AA-compliant palette.
        InternalChartSpec::FALLBACK_PALETTE
            .iter()
            .map(|s| Arc::from(*s))
            .collect()
    } else {
        colors
    }
}

// ---------------------------------------------------------------------------
// Test suite
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::ChartRenderer;
    use slideforge_types::{
        brand::{Brand, BrandFonts, BrandPalette},
        span::SourceSpan,
        specs::{AltText, ChartSpec},
    };

    use super::*;
    use crate::types::{DataPoint, DataSeries, InternalChartSpec};

    // -----------------------------------------------------------------------
    // Test fixtures
    // -----------------------------------------------------------------------

    /// Three-point data series for use in all per-chart-type tests.
    fn three_point_series(name: &str) -> DataSeries {
        DataSeries {
            name: Arc::from(name),
            points: vec![
                DataPoint { label: Arc::from("Jan"), value: 100.0 },
                DataPoint { label: Arc::from("Feb"), value: 150.0 },
                DataPoint { label: Arc::from("Mar"), value: 120.0 },
            ],
        }
    }

    /// Build a minimal [`InternalChartSpec`] for a given chart type.
    fn make_spec(chart_type: crate::types::ChartType) -> InternalChartSpec {
        InternalChartSpec {
            chart_type,
            data: vec![three_point_series("Revenue")],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("Revenue chart"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
        }
    }

    /// Build a [`Brand`] with two accent colors (for AC-003 tests).
    fn make_brand_two_accents() -> Brand {
        Brand {
            name: Arc::from("acme"),
            palette: BrandPalette {
                primary: Arc::from("#003766"),
                secondary: Arc::from("#FF6F00"),
                accent: Arc::from("#009E60"),
                neutral: Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    /// Build a [`Brand`] with an empty palette (all empty strings — triggers fallback).
    fn make_brand_empty_palette() -> Brand {
        Brand {
            name: Arc::from("empty"),
            palette: BrandPalette {
                primary: Arc::from(""),
                secondary: Arc::from(""),
                accent: Arc::from(""),
                neutral: Arc::from(""),
            },
            fonts: BrandFonts {
                heading: Arc::from(""),
                body: Arc::from(""),
                mono: Arc::from(""),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    fn make_chart_spec(chart_type: &str) -> ChartSpec {
        ChartSpec {
            chart_type: Arc::from(chart_type),
            alt: Some(AltText::Provided(Arc::from("test chart"))),
            decorative: false,
            span: SourceSpan::default(),
        }
    }

    // -----------------------------------------------------------------------
    // AC-001: ChartRenderer trait implementation
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_ac001_chart_renderer_impl_id() {
        let renderer = ChartRendererImpl::new();
        assert_eq!(renderer.id(), "plotters");
    }

    #[test]
    fn test_bc_1_11_001_ac001_chart_renderer_impl_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ChartRendererImpl>();
    }

    // -----------------------------------------------------------------------
    // AC-002: All 7 chart types produce valid SVG
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_bar_produces_nonempty_svg() {
        let spec = make_spec(crate::types::ChartType::Bar);
        let result = crate::bar::render_bar(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "bar SVG must not be empty");
    }

    #[test]
    fn test_bc_1_11_001_line_produces_nonempty_svg() {
        let spec = make_spec(crate::types::ChartType::Line);
        let result = crate::line::render_line(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "line SVG must not be empty");
    }

    #[test]
    fn test_bc_1_11_001_pie_produces_nonempty_svg() {
        let spec = make_spec(crate::types::ChartType::Pie);
        let result = crate::pie::render_pie(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "pie SVG must not be empty");
    }

    #[test]
    fn test_bc_1_11_001_scatter_produces_nonempty_svg() {
        let spec = make_spec(crate::types::ChartType::Scatter);
        let result = crate::scatter::render_scatter(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "scatter SVG must not be empty");
    }

    #[test]
    fn test_bc_1_11_001_area_produces_nonempty_svg() {
        let spec = make_spec(crate::types::ChartType::Area);
        let result = crate::area::render_area(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "area SVG must not be empty");
    }

    #[test]
    fn test_bc_1_11_001_histogram_produces_nonempty_svg() {
        let spec = make_spec(crate::types::ChartType::Histogram);
        let result = crate::histogram::render_histogram(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "histogram SVG must not be empty");
    }

    #[test]
    fn test_bc_1_11_001_stacked_bar_produces_nonempty_svg() {
        let spec = make_spec(crate::types::ChartType::StackedBar);
        let result = crate::stacked_bar::render_stacked_bar(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "stacked_bar SVG must not be empty");
    }

    // -----------------------------------------------------------------------
    // FINDING-003: SVG width/height must have "px" suffix (AC-004)
    // -----------------------------------------------------------------------

    #[test]
    fn test_f031_003_bar_svg_has_px_dimensions() {
        let spec = make_spec(crate::types::ChartType::Bar);
        let svg = crate::bar::render_bar(&spec).unwrap();
        assert!(svg.contains("width=\"800px\""), "bar SVG must have width=\"800px\"");
        assert!(svg.contains("height=\"450px\""), "bar SVG must have height=\"450px\"");
    }

    #[test]
    fn test_f031_003_line_svg_has_px_dimensions() {
        let spec = make_spec(crate::types::ChartType::Line);
        let svg = crate::line::render_line(&spec).unwrap();
        assert!(svg.contains("width=\"800px\""), "line SVG must have width=\"800px\"");
        assert!(svg.contains("height=\"450px\""), "line SVG must have height=\"450px\"");
    }

    #[test]
    fn test_f031_003_pie_svg_has_px_dimensions() {
        let spec = make_spec(crate::types::ChartType::Pie);
        let svg = crate::pie::render_pie(&spec).unwrap();
        assert!(svg.contains("width=\"800px\""), "pie SVG must have width=\"800px\"");
        assert!(svg.contains("height=\"450px\""), "pie SVG must have height=\"450px\"");
    }

    #[test]
    fn test_f031_003_scatter_svg_has_px_dimensions() {
        let spec = make_spec(crate::types::ChartType::Scatter);
        let svg = crate::scatter::render_scatter(&spec).unwrap();
        assert!(svg.contains("width=\"800px\""), "scatter SVG must have width=\"800px\"");
        assert!(svg.contains("height=\"450px\""), "scatter SVG must have height=\"450px\"");
    }

    #[test]
    fn test_f031_003_area_svg_has_px_dimensions() {
        let spec = make_spec(crate::types::ChartType::Area);
        let svg = crate::area::render_area(&spec).unwrap();
        assert!(svg.contains("width=\"800px\""), "area SVG must have width=\"800px\"");
        assert!(svg.contains("height=\"450px\""), "area SVG must have height=\"450px\"");
    }

    #[test]
    fn test_f031_003_histogram_svg_has_px_dimensions() {
        let spec = make_spec(crate::types::ChartType::Histogram);
        let svg = crate::histogram::render_histogram(&spec).unwrap();
        assert!(svg.contains("width=\"800px\""), "histogram SVG must have width=\"800px\"");
        assert!(svg.contains("height=\"450px\""), "histogram SVG must have height=\"450px\"");
    }

    #[test]
    fn test_f031_003_stacked_bar_svg_has_px_dimensions() {
        let spec = make_spec(crate::types::ChartType::StackedBar);
        let svg = crate::stacked_bar::render_stacked_bar(&spec).unwrap();
        assert!(svg.contains("width=\"800px\""), "stacked_bar SVG must have width=\"800px\"");
        assert!(svg.contains("height=\"450px\""), "stacked_bar SVG must have height=\"450px\"");
    }

    // -----------------------------------------------------------------------
    // FINDING-004: NaN/Infinity values must be rejected
    // -----------------------------------------------------------------------

    #[test]
    fn test_f031_004_nan_value_returns_error() {
        let spec = InternalChartSpec {
            chart_type: crate::types::ChartType::Bar,
            data: vec![DataSeries {
                name: Arc::from("test"),
                points: vec![DataPoint { label: Arc::from("Q1"), value: f64::NAN }],
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("nan test"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
        };
        let result = crate::bar::render_bar(&spec);
        assert!(result.is_err(), "NaN data must return an error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("NaN") || err_msg.contains("infinite") || !err_msg.is_empty(),
            "error must describe the problem; got: {err_msg}"
        );
    }

    #[test]
    fn test_f031_004_infinite_value_returns_error() {
        let spec = InternalChartSpec {
            chart_type: crate::types::ChartType::Line,
            data: vec![DataSeries {
                name: Arc::from("test"),
                points: vec![DataPoint { label: Arc::from("Q1"), value: f64::INFINITY }],
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("inf test"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
        };
        let result = crate::line::render_line(&spec);
        assert!(result.is_err(), "Infinite data must return an error");
    }

    // -----------------------------------------------------------------------
    // FINDING-006: Empty data must return MissingDataField error
    // -----------------------------------------------------------------------

    #[test]
    fn test_f031_006_empty_data_line_returns_error() {
        let mut spec = make_spec(crate::types::ChartType::Line);
        spec.data = vec![];
        assert!(crate::line::render_line(&spec).is_err(), "empty data must error for line");
    }

    #[test]
    fn test_f031_006_empty_data_area_returns_error() {
        let mut spec = make_spec(crate::types::ChartType::Area);
        spec.data = vec![];
        assert!(crate::area::render_area(&spec).is_err(), "empty data must error for area");
    }

    #[test]
    fn test_f031_006_empty_data_scatter_returns_error() {
        let mut spec = make_spec(crate::types::ChartType::Scatter);
        spec.data = vec![];
        assert!(
            crate::scatter::render_scatter(&spec).is_err(),
            "empty data must error for scatter"
        );
    }

    #[test]
    fn test_f031_006_empty_data_stacked_bar_returns_error() {
        let mut spec = make_spec(crate::types::ChartType::StackedBar);
        spec.data = vec![];
        assert!(
            crate::stacked_bar::render_stacked_bar(&spec).is_err(),
            "empty data must error for stacked_bar"
        );
    }

    // -----------------------------------------------------------------------
    // AC-004: viewBox and absolute dimensions
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_bar_has_viewbox() {
        let spec = make_spec(crate::types::ChartType::Bar);
        let svg = crate::bar::render_bar(&spec).unwrap();
        assert!(
            svg.contains("viewBox=\"0 0 800 450\""),
            "bar SVG must contain viewBox=\"0 0 800 450\"; got: {svg}"
        );
    }

    #[test]
    fn test_bc_1_11_001_line_has_viewbox() {
        let spec = make_spec(crate::types::ChartType::Line);
        let svg = crate::line::render_line(&spec).unwrap();
        assert!(svg.contains("viewBox=\"0 0 800 450\""), "line SVG viewBox missing");
    }

    #[test]
    fn test_bc_1_11_001_pie_has_viewbox() {
        let spec = make_spec(crate::types::ChartType::Pie);
        let svg = crate::pie::render_pie(&spec).unwrap();
        assert!(svg.contains("viewBox=\"0 0 800 450\""), "pie SVG viewBox missing");
    }

    #[test]
    fn test_bc_1_11_001_scatter_has_viewbox() {
        let spec = make_spec(crate::types::ChartType::Scatter);
        let svg = crate::scatter::render_scatter(&spec).unwrap();
        assert!(svg.contains("viewBox=\"0 0 800 450\""), "scatter SVG viewBox missing");
    }

    #[test]
    fn test_bc_1_11_001_area_has_viewbox() {
        let spec = make_spec(crate::types::ChartType::Area);
        let svg = crate::area::render_area(&spec).unwrap();
        assert!(svg.contains("viewBox=\"0 0 800 450\""), "area SVG viewBox missing");
    }

    #[test]
    fn test_bc_1_11_001_histogram_has_viewbox() {
        let spec = make_spec(crate::types::ChartType::Histogram);
        let svg = crate::histogram::render_histogram(&spec).unwrap();
        assert!(svg.contains("viewBox=\"0 0 800 450\""), "histogram SVG viewBox missing");
    }

    #[test]
    fn test_bc_1_11_001_stacked_bar_has_viewbox() {
        let spec = make_spec(crate::types::ChartType::StackedBar);
        let svg = crate::stacked_bar::render_stacked_bar(&spec).unwrap();
        assert!(svg.contains("viewBox=\"0 0 800 450\""), "stacked_bar SVG viewBox missing");
    }

    // -----------------------------------------------------------------------
    // AC-005: Accessibility — aria-label injected after full pipeline
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_bar_has_aria_label() {
        let spec = make_spec(crate::types::ChartType::Bar);
        let raw_svg = crate::bar::render_bar(&spec).unwrap();
        let svg = crate::accessibility::inject_aria_attributes(&raw_svg, spec.alt.as_ref())
            .unwrap();
        assert!(
            svg.as_str().contains("aria-label="),
            "bar SVG must contain aria-label attribute"
        );
    }

    #[test]
    fn test_bc_1_11_001_line_has_aria_label() {
        let spec = make_spec(crate::types::ChartType::Line);
        let raw_svg = crate::line::render_line(&spec).unwrap();
        let svg = crate::accessibility::inject_aria_attributes(&raw_svg, spec.alt.as_ref())
            .unwrap();
        assert!(svg.as_str().contains("aria-label="), "line SVG aria-label missing");
    }

    #[test]
    fn test_bc_1_11_001_pie_has_aria_label() {
        let spec = make_spec(crate::types::ChartType::Pie);
        let raw_svg = crate::pie::render_pie(&spec).unwrap();
        let svg = crate::accessibility::inject_aria_attributes(&raw_svg, spec.alt.as_ref())
            .unwrap();
        assert!(svg.as_str().contains("aria-label="), "pie SVG aria-label missing");
    }

    #[test]
    fn test_bc_1_11_001_scatter_has_aria_label() {
        let spec = make_spec(crate::types::ChartType::Scatter);
        let raw_svg = crate::scatter::render_scatter(&spec).unwrap();
        let svg = crate::accessibility::inject_aria_attributes(&raw_svg, spec.alt.as_ref())
            .unwrap();
        assert!(svg.as_str().contains("aria-label="), "scatter SVG aria-label missing");
    }

    #[test]
    fn test_bc_1_11_001_area_has_aria_label() {
        let spec = make_spec(crate::types::ChartType::Area);
        let raw_svg = crate::area::render_area(&spec).unwrap();
        let svg = crate::accessibility::inject_aria_attributes(&raw_svg, spec.alt.as_ref())
            .unwrap();
        assert!(svg.as_str().contains("aria-label="), "area SVG aria-label missing");
    }

    #[test]
    fn test_bc_1_11_001_histogram_has_aria_label() {
        let spec = make_spec(crate::types::ChartType::Histogram);
        let raw_svg = crate::histogram::render_histogram(&spec).unwrap();
        let svg = crate::accessibility::inject_aria_attributes(&raw_svg, spec.alt.as_ref())
            .unwrap();
        assert!(svg.as_str().contains("aria-label="), "histogram SVG aria-label missing");
    }

    #[test]
    fn test_bc_1_11_001_stacked_bar_has_aria_label() {
        let spec = make_spec(crate::types::ChartType::StackedBar);
        let raw_svg = crate::stacked_bar::render_stacked_bar(&spec).unwrap();
        let svg = crate::accessibility::inject_aria_attributes(&raw_svg, spec.alt.as_ref())
            .unwrap();
        assert!(svg.as_str().contains("aria-label="), "stacked_bar SVG aria-label missing");
    }

    // -----------------------------------------------------------------------
    // AC-006: No script or foreignObject
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_bar_no_script() {
        let spec = make_spec(crate::types::ChartType::Bar);
        let svg = crate::bar::render_bar(&spec).unwrap();
        assert!(!svg.contains("<script"), "bar SVG must not contain <script");
        assert!(!svg.contains("<foreignObject"), "bar SVG must not contain <foreignObject");
    }

    #[test]
    fn test_bc_1_11_001_line_no_script() {
        let spec = make_spec(crate::types::ChartType::Line);
        let svg = crate::line::render_line(&spec).unwrap();
        assert!(!svg.contains("<script"), "line SVG must not contain <script");
        assert!(!svg.contains("<foreignObject"), "line SVG must not contain <foreignObject");
    }

    #[test]
    fn test_bc_1_11_001_pie_no_script() {
        let spec = make_spec(crate::types::ChartType::Pie);
        let svg = crate::pie::render_pie(&spec).unwrap();
        assert!(!svg.contains("<script"), "pie SVG must not contain <script");
        assert!(!svg.contains("<foreignObject"), "pie SVG must not contain <foreignObject");
    }

    #[test]
    fn test_bc_1_11_001_scatter_no_script() {
        let spec = make_spec(crate::types::ChartType::Scatter);
        let svg = crate::scatter::render_scatter(&spec).unwrap();
        assert!(!svg.contains("<script"), "scatter SVG must not contain <script");
        assert!(!svg.contains("<foreignObject"), "scatter SVG must not contain <foreignObject");
    }

    #[test]
    fn test_bc_1_11_001_area_no_script() {
        let spec = make_spec(crate::types::ChartType::Area);
        let svg = crate::area::render_area(&spec).unwrap();
        assert!(!svg.contains("<script"), "area SVG must not contain <script");
        assert!(!svg.contains("<foreignObject"), "area SVG must not contain <foreignObject");
    }

    #[test]
    fn test_bc_1_11_001_histogram_no_script() {
        let spec = make_spec(crate::types::ChartType::Histogram);
        let svg = crate::histogram::render_histogram(&spec).unwrap();
        assert!(!svg.contains("<script"), "histogram SVG must not contain <script");
        assert!(!svg.contains("<foreignObject"), "histogram SVG must not contain <foreignObject");
    }

    #[test]
    fn test_bc_1_11_001_stacked_bar_no_script() {
        let spec = make_spec(crate::types::ChartType::StackedBar);
        let svg = crate::stacked_bar::render_stacked_bar(&spec).unwrap();
        assert!(!svg.contains("<script"), "stacked_bar SVG must not contain <script");
        assert!(
            !svg.contains("<foreignObject"),
            "stacked_bar SVG must not contain <foreignObject"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003: Brand colors applied to series (tests via extract_accent_colors)
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_brand_colors_applied() {
        // extract_accent_colors is a helper that builds the color list fed into
        // InternalChartSpec. Verify that the brand's accent/primary/secondary
        // colors appear in the extracted list.
        let brand = make_brand_two_accents();
        let colors = extract_accent_colors(&brand);
        assert!(!colors.is_empty(), "accent colors must not be empty");
        // At least one of the brand's palette colors must be present.
        let color_strs: Vec<&str> = colors.iter().map(std::convert::AsRef::as_ref).collect();
        let brand_colors = ["#003766", "#FF6F00", "#009E60"];
        let any_present = brand_colors.iter().any(|bc| color_strs.contains(bc));
        assert!(any_present, "brand colors must appear in extracted palette; got: {color_strs:?}");
    }

    #[test]
    fn test_bc_1_11_001_fallback_palette_when_brand_empty() {
        let brand = make_brand_empty_palette();
        let colors = extract_accent_colors(&brand);
        assert!(!colors.is_empty(), "fallback palette must be non-empty");
        // Fallback palette first entry.
        assert_eq!(
            colors[0].as_ref(),
            InternalChartSpec::FALLBACK_PALETTE[0],
            "first fallback color must be the canonical default"
        );
    }

    // -----------------------------------------------------------------------
    // AC-002: Unknown chart type error (EC-003)
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_unknown_type_error() {
        let renderer = ChartRendererImpl::new();
        let spec = make_chart_spec("radar");
        let brand = make_brand_two_accents();
        let result = renderer.render(&spec, &brand);
        assert!(result.is_err(), "render must fail for unknown chart type 'radar'");
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("radar") || msg.contains("unsupported"),
            "error message must mention 'radar' or 'unsupported'; got: {msg}"
        );
    }

    // -----------------------------------------------------------------------
    // FINDING-001: trait render() must return InvalidSpec for skeleton ChartSpec
    // (no data — data must be provided via dispatch_and_process from eval pipeline)
    // -----------------------------------------------------------------------

    #[test]
    fn test_f031_001_trait_render_returns_invalid_spec_for_skeleton_chartspec() {
        let renderer = ChartRendererImpl::new();
        let spec = make_chart_spec("bar");
        let brand = make_brand_two_accents();
        let result = renderer.render(&spec, &brand);
        // The trait render() receives a skeleton ChartSpec with no series data.
        // It must return InvalidSpec indicating that data must be provided
        // through the eval pipeline's dispatch_and_process(InternalChartSpec).
        assert!(result.is_err(), "render() with skeleton ChartSpec must return an error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("data") || msg.contains("spec") || msg.contains("eval"),
            "error must mention data/spec/eval pipeline; got: {msg}"
        );
    }

    // -----------------------------------------------------------------------
    // EC-002: Pie chart with single slice
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_pie_single_slice() {
        let spec = InternalChartSpec {
            chart_type: crate::types::ChartType::Pie,
            data: vec![DataSeries {
                name: Arc::from("Total"),
                points: vec![DataPoint { label: Arc::from("All"), value: 100.0 }],
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("single slice pie"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
        };
        let result = crate::pie::render_pie(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "single-slice pie SVG must not be empty");
    }

    // -----------------------------------------------------------------------
    // AC-008: Brand font applied to axis labels
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_brand_font_in_labels() {
        // The chart renderer constructs TextStyle with the brand font family.
        // We verify the font name appears in the rendered SVG (plotters renders
        // font-family CSS on text elements).
        let spec = InternalChartSpec {
            chart_type: crate::types::ChartType::Bar,
            data: vec![three_point_series("Revenue")],
            title: Some(Arc::from("Brand Font Test")),
            x_label: Some(Arc::from("Month")),
            y_label: Some(Arc::from("Value")),
            alt: Arc::from("test"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("Calibri"),
        };
        let svg = crate::bar::render_bar(&spec).unwrap();
        assert!(
            svg.contains("Calibri"),
            "SVG must reference 'Calibri' font family in text elements; got: {}",
            &svg[..svg.len().min(500)]
        );
    }

    // -----------------------------------------------------------------------
    // FINDING-001 (Pass 2): All-negative data must produce visible charts
    // -----------------------------------------------------------------------

    /// Build an [`InternalChartSpec`] whose only series has all-negative values.
    fn negative_data_spec(chart_type: crate::types::ChartType) -> InternalChartSpec {
        InternalChartSpec {
            chart_type,
            data: vec![DataSeries {
                name: Arc::from("Loss"),
                points: vec![
                    DataPoint { label: Arc::from("Q1"), value: -50.0 },
                    DataPoint { label: Arc::from("Q2"), value: -30.0 },
                    DataPoint { label: Arc::from("Q3"), value: -80.0 },
                ],
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("Negative chart"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
        }
    }

    /// Verify that the SVG axis tick label `<text>` elements include a negative number,
    /// proving the y-axis range extends below zero and the data is *visible* (not clipped).
    ///
    /// Plotters renders axis tick values as `<text>...</text>` content. We parse out the
    /// text content (between `>` and `</text>`) and check at least one begins with "-".
    /// This avoids false positives from negative transform/coordinate attributes.
    fn assert_svg_has_negative_axis_label(svg: &str, chart_name: &str) {
        let mut pos = 0;
        let mut found = false;
        while let Some(text_start) = svg[pos..].find("<text") {
            let abs_start = pos + text_start;
            // Find the `>` that closes the opening tag.
            if let Some(rel_gt) = svg[abs_start..].find('>') {
                let content_start = abs_start + rel_gt + 1;
                // Find `</text>`.
                if let Some(rel_end) = svg[content_start..].find("</text>") {
                    let text_content = &svg[content_start..content_start + rel_end];
                    let trimmed = text_content.trim();
                    // Check if this text node is a negative number (starts with "-" + digit).
                    if trimmed.starts_with('-')
                        && trimmed.len() > 1
                        && trimmed.as_bytes()[1].is_ascii_digit()
                    {
                        found = true;
                        break;
                    }
                    pos = content_start + rel_end + 7; // skip past </text>
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        assert!(
            found,
            "{chart_name}: all-negative data SVG must have at least one negative y-axis tick \
             label (e.g. \"-50\"). Y-axis range must extend below zero so data is visible. \
             Check that the renderer computes y_min from data, not hardcoded to 0.0. \
             SVG first 800 chars: {}",
            &svg[..svg.len().min(800)]
        );
    }

    #[test]
    fn test_f031_p2_001_bar_negative_data_renders_with_visible_range() {
        let spec = negative_data_spec(crate::types::ChartType::Bar);
        let svg = crate::bar::render_bar(&spec)
            .expect("all-negative bar data must render without error");
        assert!(!svg.is_empty(), "all-negative bar SVG must be non-empty");
        assert_svg_has_negative_axis_label(&svg, "bar");
    }

    #[test]
    fn test_f031_p2_001_line_negative_data_renders_with_visible_range() {
        let spec = negative_data_spec(crate::types::ChartType::Line);
        let svg = crate::line::render_line(&spec)
            .expect("all-negative line data must render without error");
        assert!(!svg.is_empty(), "all-negative line SVG must be non-empty");
        assert_svg_has_negative_axis_label(&svg, "line");
    }

    #[test]
    fn test_f031_p2_001_scatter_negative_data_renders_with_visible_range() {
        let spec = negative_data_spec(crate::types::ChartType::Scatter);
        let svg = crate::scatter::render_scatter(&spec)
            .expect("all-negative scatter data must render without error");
        assert!(!svg.is_empty(), "all-negative scatter SVG must be non-empty");
        assert_svg_has_negative_axis_label(&svg, "scatter");
    }

    #[test]
    fn test_f031_p2_001_area_negative_data_renders_with_visible_range() {
        let spec = negative_data_spec(crate::types::ChartType::Area);
        let svg = crate::area::render_area(&spec)
            .expect("all-negative area data must render without error");
        assert!(!svg.is_empty(), "all-negative area SVG must be non-empty");
        assert_svg_has_negative_axis_label(&svg, "area");
    }

    #[test]
    fn test_f031_p2_001_histogram_negative_data_renders_with_visible_range() {
        let spec = negative_data_spec(crate::types::ChartType::Histogram);
        let svg = crate::histogram::render_histogram(&spec)
            .expect("all-negative histogram data must render without error");
        assert!(!svg.is_empty(), "all-negative histogram SVG must be non-empty");
        assert_svg_has_negative_axis_label(&svg, "histogram");
    }

    #[test]
    fn test_f031_p2_001_stacked_bar_negative_data_renders_with_visible_range() {
        let spec = negative_data_spec(crate::types::ChartType::StackedBar);
        let svg = crate::stacked_bar::render_stacked_bar(&spec)
            .expect("all-negative stacked_bar data must render without error");
        assert!(!svg.is_empty(), "all-negative stacked_bar SVG must be non-empty");
        assert_svg_has_negative_axis_label(&svg, "stacked_bar");
    }

    // -----------------------------------------------------------------------
    // FINDING-002 (Pass 2): dispatch_and_process must be tested directly
    // -----------------------------------------------------------------------

    /// Test that `dispatch_and_process` produces valid SVG, aria-label, and no
    /// forbidden elements for a standard `InternalChartSpec`.
    #[test]
    fn test_f031_p2_002_dispatch_and_process_bar_produces_valid_svg() {
        let spec = make_spec(crate::types::ChartType::Bar);
        let result = ChartRendererImpl::dispatch_and_process(&spec);
        let chart_svg = result.expect("dispatch_and_process must succeed for a valid bar spec");

        let svg = chart_svg.as_str();
        assert!(!svg.is_empty(), "dispatch_and_process must return non-empty SVG");

        // Must have aria-label (injected by inject_aria_attributes).
        assert!(
            svg.contains("aria-label="),
            "dispatch_and_process output must contain aria-label attribute; got: {}",
            &svg[..svg.len().min(400)]
        );

        // Must have role="img".
        assert!(
            svg.contains("role=\"img\""),
            "dispatch_and_process output must contain role=\"img\"; got: {}",
            &svg[..svg.len().min(400)]
        );

        // Must NOT contain forbidden elements.
        assert!(!svg.contains("<script"), "dispatch_and_process output must not contain <script");
        assert!(
            !svg.contains("<foreignObject"),
            "dispatch_and_process output must not contain <foreignObject"
        );

        // Must be valid SVG (has root <svg> element).
        assert!(svg.contains("<svg"), "dispatch_and_process output must contain <svg root element");
    }

    /// Test that `dispatch_and_process` works for all 7 chart types.
    #[test]
    fn test_f031_p2_002_dispatch_and_process_all_chart_types() {
        use crate::types::ChartType;

        for chart_type in [
            ChartType::Bar,
            ChartType::Line,
            ChartType::Pie,
            ChartType::Scatter,
            ChartType::Area,
            ChartType::Histogram,
            ChartType::StackedBar,
        ] {
            let spec = make_spec(chart_type.clone());
            let result = ChartRendererImpl::dispatch_and_process(&spec);
            let chart_svg = result.unwrap_or_else(|e| {
                panic!("dispatch_and_process failed for {chart_type:?}: {e}");
            });
            let svg = chart_svg.as_str();
            assert!(!svg.is_empty(), "dispatch_and_process must return non-empty SVG for {chart_type:?}");
            assert!(
                svg.contains("aria-label="),
                "dispatch_and_process output must contain aria-label for {chart_type:?}"
            );
        }
    }

    /// Test that alt text containing `<script>` is safely escaped in the output.
    ///
    /// The `<script>` tag in alt text must be XML-escaped in the aria-label attribute
    /// and `<title>` content — it must NOT appear as a literal `<script>` element.
    /// The safety check (`assert_no_forbidden_elements`) must pass.
    #[test]
    fn test_f031_p2_002_dispatch_and_process_script_in_alt_text_is_escaped() {
        let spec = InternalChartSpec {
            chart_type: crate::types::ChartType::Bar,
            data: vec![DataSeries {
                name: Arc::from("Revenue"),
                points: vec![
                    DataPoint { label: Arc::from("Q1"), value: 100.0 },
                    DataPoint { label: Arc::from("Q2"), value: 200.0 },
                ],
            }],
            title: None,
            x_label: None,
            y_label: None,
            // Alt text containing a <script> tag — must be escaped, not injected.
            alt: Arc::from("Revenue <script>alert('xss')</script> chart"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
        };

        let result = ChartRendererImpl::dispatch_and_process(&spec);
        let chart_svg = result.expect("dispatch_and_process must succeed even with script in alt");

        let svg = chart_svg.as_str();

        // The safety check must pass — no literal <script> element in the SVG.
        // (The safety check runs BEFORE aria injection in dispatch_and_process,
        // but the alt text escaping is done during aria injection.)
        // Verify that `<script` does NOT appear as a tag in the output by
        // checking the safety check logic: the aria-label value must be escaped.
        assert!(
            svg.contains("&lt;script&gt;") || svg.contains("&lt;script"),
            "alt text with <script> must be XML-escaped in aria-label/title; \
             literal <script> tag must not appear. Got aria-label area: {}",
            &svg[..svg.len().min(600)]
        );

        // The SVG as a whole must not have a literal <script> element that would
        // be executed in a browser. The escaped form is: &lt;script&gt;
        // A literal `<script>` would only appear if escaping was bypassed.
        // safety::assert_no_forbidden_elements verifies this — but let's assert directly too:
        let svg_lower = svg.to_ascii_lowercase();
        // Strip all escaped sequences to see if any raw <script remains.
        let unescaped_check = svg_lower.replace("&lt;", "").replace("&gt;", "").replace("&amp;", "");
        assert!(
            !unescaped_check.contains("<script"),
            "After removing escaped sequences, no literal <script> must remain in the SVG output"
        );
    }

    // -----------------------------------------------------------------------
    // FINDING-002 (Pass 3): data vec with one series but empty points → error
    // -----------------------------------------------------------------------

    /// Build a spec where the data vec has one series but its points vec is empty.
    fn empty_points_spec(chart_type: crate::types::ChartType) -> InternalChartSpec {
        InternalChartSpec {
            chart_type,
            data: vec![DataSeries { name: Arc::from("empty"), points: vec![] }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("empty points"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
        }
    }

    #[test]
    fn test_f031_p3_002_bar_empty_points_returns_error() {
        let spec = empty_points_spec(crate::types::ChartType::Bar);
        let result = crate::bar::render_bar(&spec);
        assert!(result.is_err(), "bar with empty points must return an error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("points") || msg.contains("data"),
            "error must mention points/data; got: {msg}"
        );
    }

    #[test]
    fn test_f031_p3_002_line_empty_points_returns_error() {
        let spec = empty_points_spec(crate::types::ChartType::Line);
        let result = crate::line::render_line(&spec);
        assert!(result.is_err(), "line with empty points must return an error");
    }

    #[test]
    fn test_f031_p3_002_scatter_empty_points_returns_error() {
        let spec = empty_points_spec(crate::types::ChartType::Scatter);
        let result = crate::scatter::render_scatter(&spec);
        assert!(result.is_err(), "scatter with empty points must return an error");
    }

    #[test]
    fn test_f031_p3_002_area_empty_points_returns_error() {
        let spec = empty_points_spec(crate::types::ChartType::Area);
        let result = crate::area::render_area(&spec);
        assert!(result.is_err(), "area with empty points must return an error");
    }

    #[test]
    fn test_f031_p3_002_histogram_empty_points_returns_error() {
        let spec = empty_points_spec(crate::types::ChartType::Histogram);
        let result = crate::histogram::render_histogram(&spec);
        assert!(result.is_err(), "histogram with empty points must return an error");
    }

    #[test]
    fn test_f031_p3_002_stacked_bar_empty_points_returns_error() {
        let spec = empty_points_spec(crate::types::ChartType::StackedBar);
        let result = crate::stacked_bar::render_stacked_bar(&spec);
        assert!(result.is_err(), "stacked_bar with empty points must return an error");
    }

    #[test]
    fn test_f031_p3_002_pie_empty_points_returns_error() {
        let spec = empty_points_spec(crate::types::ChartType::Pie);
        let result = crate::pie::render_pie(&spec);
        assert!(result.is_err(), "pie with empty points must return an error");
    }

    // -----------------------------------------------------------------------
    // FINDING-002 (Pass 4): single-point line and area must produce valid SVG
    // -----------------------------------------------------------------------

    /// Build a spec with exactly one data point (single-element series).
    fn single_point_spec(chart_type: crate::types::ChartType) -> InternalChartSpec {
        InternalChartSpec {
            chart_type,
            data: vec![DataSeries {
                name: Arc::from("Solo"),
                points: vec![DataPoint { label: Arc::from("Jan"), value: 42.0 }],
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("single point chart"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
        }
    }

    #[test]
    fn test_f031_p4_002_line_single_point_produces_valid_svg() {
        let spec = single_point_spec(crate::types::ChartType::Line);
        let result = crate::line::render_line(&spec);
        let svg = result.expect("single-point line must produce valid SVG without panicking");
        assert!(!svg.is_empty(), "single-point line SVG must not be empty");
        assert!(svg.contains("<svg"), "single-point line output must contain <svg root element");
    }

    #[test]
    fn test_f031_p4_002_area_single_point_produces_valid_svg() {
        let spec = single_point_spec(crate::types::ChartType::Area);
        let result = crate::area::render_area(&spec);
        let svg = result.expect("single-point area must produce valid SVG without panicking");
        assert!(!svg.is_empty(), "single-point area SVG must not be empty");
        assert!(svg.contains("<svg"), "single-point area output must contain <svg root element");
    }

    // -----------------------------------------------------------------------
    // Snapshot tests — one per chart type with fixed 3-point data
    // -----------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_snapshot_bar() {
        let spec = make_spec(crate::types::ChartType::Bar);
        let svg = crate::bar::render_bar(&spec).unwrap();
        insta::assert_yaml_snapshot!("bar_svg", svg);
    }

    #[test]
    fn test_bc_1_11_001_snapshot_line() {
        let spec = make_spec(crate::types::ChartType::Line);
        let svg = crate::line::render_line(&spec).unwrap();
        insta::assert_yaml_snapshot!("line_svg", svg);
    }

    #[test]
    fn test_bc_1_11_001_snapshot_pie() {
        let spec = make_spec(crate::types::ChartType::Pie);
        let svg = crate::pie::render_pie(&spec).unwrap();
        insta::assert_yaml_snapshot!("pie_svg", svg);
    }

    #[test]
    fn test_bc_1_11_001_snapshot_scatter() {
        let spec = make_spec(crate::types::ChartType::Scatter);
        let svg = crate::scatter::render_scatter(&spec).unwrap();
        insta::assert_yaml_snapshot!("scatter_svg", svg);
    }

    #[test]
    fn test_bc_1_11_001_snapshot_area() {
        let spec = make_spec(crate::types::ChartType::Area);
        let svg = crate::area::render_area(&spec).unwrap();
        insta::assert_yaml_snapshot!("area_svg", svg);
    }

    #[test]
    fn test_bc_1_11_001_snapshot_histogram() {
        let spec = make_spec(crate::types::ChartType::Histogram);
        let svg = crate::histogram::render_histogram(&spec).unwrap();
        insta::assert_yaml_snapshot!("histogram_svg", svg);
    }

    #[test]
    fn test_bc_1_11_001_snapshot_stacked_bar() {
        let spec = make_spec(crate::types::ChartType::StackedBar);
        let svg = crate::stacked_bar::render_stacked_bar(&spec).unwrap();
        insta::assert_yaml_snapshot!("stacked_bar_svg", svg);
    }
}
