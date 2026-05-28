//! Internal chart types for `slideforge-charts`.
//!
//! These types are the richer internal representation used by chart renderer
//! functions. The public API accepts `slideforge_plugin_api::ChartRenderer::render`
//! which takes `slideforge_types::ChartSpec`, but that skeleton type does not
//! carry series data. This crate defines its own extended spec types for the
//! internal rendering pipeline.

use std::sync::Arc;

use thiserror::Error;

/// The seven chart types supported by `slideforge-charts` in v1.0.
///
/// Corresponds to the `type:` field in a `slide chart:` DSL block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChartType {
    /// Vertical bar chart with grouped series.
    Bar,
    /// Line chart with optional data points.
    Line,
    /// Pie (or donut) chart.
    Pie,
    /// X/Y scatter plot.
    Scatter,
    /// Area (filled line) chart.
    Area,
    /// Frequency distribution bars.
    Histogram,
    /// Stacked vertical bar chart.
    StackedBar,
}

impl ChartType {
    /// Parse a chart type keyword string into a [`ChartType`] variant.
    ///
    /// Returns `None` for unknown type keywords.
    #[must_use]
    pub fn from_keyword(kw: &str) -> Option<Self> {
        match kw {
            "bar" => Some(Self::Bar),
            "line" => Some(Self::Line),
            "pie" => Some(Self::Pie),
            "scatter" => Some(Self::Scatter),
            "area" => Some(Self::Area),
            "histogram" => Some(Self::Histogram),
            "stacked-bar" | "stacked_bar" => Some(Self::StackedBar),
            _ => None,
        }
    }

    /// Return the canonical keyword string for this chart type.
    #[must_use]
    pub fn as_keyword(&self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Line => "line",
            Self::Pie => "pie",
            Self::Scatter => "scatter",
            Self::Area => "area",
            Self::Histogram => "histogram",
            Self::StackedBar => "stacked-bar",
        }
    }
}

/// A single data point in a chart series.
///
/// For most chart types, `label` is the x-axis category and `value` is the
/// y-axis measurement. For scatter charts, `label` is used as a point
/// identifier and the numeric x-coordinate is encoded in the label when
/// needed by the renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct DataPoint {
    /// X-axis label or category name.
    pub label: Arc<str>,
    /// Y-axis (or radial) value.
    pub value: f64,
}

/// A named data series for chart rendering.
///
/// Each series maps to a distinct color in the chart, taken from the brand
/// palette's accent colors cycling if needed.
#[derive(Debug, Clone, PartialEq)]
pub struct DataSeries {
    /// The series name displayed in the legend.
    pub name: Arc<str>,
    /// The ordered data points for this series.
    pub points: Vec<DataPoint>,
}

/// The extended chart specification used internally by `slideforge-charts`.
///
/// This is distinct from [`slideforge_types::ChartSpec`] (the skeleton type in
/// the IR). The chart renderer constructs an [`InternalChartSpec`] from the IR
/// spec plus the evaluated data binding before delegating to a type-specific
/// renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct InternalChartSpec {
    /// The chart type to render.
    pub chart_type: ChartType,
    /// The data series to plot.
    pub data: Vec<DataSeries>,
    /// Optional chart title rendered above the chart area.
    pub title: Option<Arc<str>>,
    /// Optional x-axis label.
    pub x_label: Option<Arc<str>>,
    /// Optional y-axis label.
    pub y_label: Option<Arc<str>>,
    /// Alt text injected as `aria-label` and `<title>` in the SVG.
    pub alt: Arc<str>,
    /// Output width in pixels. Default: 800.
    pub width: u32,
    /// Output height in pixels. Default: 450.
    pub height: u32,
    /// Brand accent colors as CSS hex strings (e.g., `"#003766"`).
    ///
    /// Series colors are assigned from this slice in order, cycling if there
    /// are more series than colors. If empty, the fallback palette is used.
    pub accent_colors: Vec<Arc<str>>,
    /// Brand font family name for axis labels and legend text.
    ///
    /// The plotters `TextStyle` is constructed with this name. If the font is
    /// not available in the rendering context, plotters falls back to its
    /// built-in font.
    pub font_family: Arc<str>,
}

impl InternalChartSpec {
    /// Default output width (800 px, 16:9 proportion matching default slide).
    pub const DEFAULT_WIDTH: u32 = 800;
    /// Default output height (450 px).
    pub const DEFAULT_HEIGHT: u32 = 450;

    /// Fallback color palette used when `accent_colors` is empty.
    ///
    /// These colors satisfy WCAG AA contrast requirements on white backgrounds.
    pub const FALLBACK_PALETTE: &'static [&'static str] = &[
        "#003766", "#FF6F00", "#009E60", "#7030A0", "#FF0000", "#0070C0", "#00B050",
    ];
}

/// A newtype wrapping the SVG string produced by a chart renderer.
///
/// The string is a complete, self-contained SVG document (no external
/// references). Accessibility attributes (`aria-label`, `role="img"`, and a
/// `<title>` child element) are injected by
/// [`crate::accessibility::inject_aria_attributes`] before wrapping.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChartSvg(pub String);

impl ChartSvg {
    /// Consume the newtype and return the inner SVG string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    /// Return a reference to the inner SVG string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Error produced by the chart renderer.
#[derive(Debug, Error)]
pub enum ChartError {
    /// The requested chart type is not supported by this renderer.
    ///
    /// Maps to error class E-PAR-007.
    #[error(
        "unsupported chart type '{name}'; supported types: bar, line, pie, scatter, area, histogram, stacked-bar"
    )]
    UnsupportedType {
        /// The unknown chart type keyword.
        name: Arc<str>,
    },

    /// A required data field is absent in the data series.
    ///
    /// Maps to error class E-DAT-005.
    #[error("missing required data field '{field}' in chart data")]
    MissingDataField {
        /// The name of the missing field.
        field: Arc<str>,
    },

    /// An internal rendering error from the plotters backend or SVG
    /// post-processing.
    #[error("chart render failed: {message}")]
    RenderError {
        /// Description of the failure.
        message: Arc<str>,
    },

    /// Chart data evaluated to an empty collection before `render()` was called.
    ///
    /// Maps to error code `E-LAY-003`. This variant is produced by the
    /// empty-data guard in [`crate::validation`] when [`slideforge_types::Value::List`]
    /// is empty. The guard runs BEFORE [`crate::ChartRendererImpl::dispatch_and_process`]
    /// to satisfy BC-1.11.002 invariant 2 (renderer is never called with empty data).
    ///
    /// In strict mode: this error causes the build to fail (exit code 2, no output).
    /// In warn-only mode: the caller produces a [`slideforge_layout::FrameContent::ErrorSlidePlaceholder`].
    #[error("E-LAY-003: chart data is empty for slide '{slide_title}' (expression: {expression})")]
    EmptyData {
        /// The title of the chart slide where empty data was detected.
        slide_title: Arc<str>,
        /// The data binding expression from the DSL (e.g., `"{{ kpis.monthly }}"`).
        expression: Arc<str>,
        /// Source location of the data binding in the `.sf` file.
        span: slideforge_types::SourceSpan,
    },
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // BC-1.11.001 type tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_bc_1_11_001_chart_type_from_keyword_all_supported() {
        let cases = [
            ("bar", ChartType::Bar),
            ("line", ChartType::Line),
            ("pie", ChartType::Pie),
            ("scatter", ChartType::Scatter),
            ("area", ChartType::Area),
            ("histogram", ChartType::Histogram),
            ("stacked-bar", ChartType::StackedBar),
            ("stacked_bar", ChartType::StackedBar),
        ];
        for (kw, expected) in cases {
            assert_eq!(ChartType::from_keyword(kw), Some(expected), "keyword: {kw}");
        }
    }

    #[test]
    fn test_bc_1_11_001_chart_type_from_keyword_unknown_returns_none() {
        assert_eq!(ChartType::from_keyword("radar"), None);
        assert_eq!(ChartType::from_keyword("treemap"), None);
        assert_eq!(ChartType::from_keyword(""), None);
        assert_eq!(ChartType::from_keyword("BAR"), None); // case-sensitive
    }

    #[test]
    fn test_bc_1_11_001_chart_type_as_keyword_roundtrip() {
        let types = [
            ChartType::Bar,
            ChartType::Line,
            ChartType::Pie,
            ChartType::Scatter,
            ChartType::Area,
            ChartType::Histogram,
            ChartType::StackedBar,
        ];
        for ct in &types {
            let kw = ct.as_keyword();
            let parsed = ChartType::from_keyword(kw);
            assert_eq!(parsed.as_ref(), Some(ct), "roundtrip failed for {kw}");
        }
    }

    #[test]
    fn test_bc_1_11_001_chart_svg_newtype_into_string() {
        let svg = ChartSvg("<svg/>".to_owned());
        assert_eq!(svg.into_string(), "<svg/>");
    }

    #[test]
    fn test_bc_1_11_001_chart_svg_as_str() {
        let svg = ChartSvg("<svg/>".to_owned());
        assert_eq!(svg.as_str(), "<svg/>");
    }

    #[test]
    fn test_bc_1_11_001_chart_error_unsupported_type_message() {
        let err = ChartError::UnsupportedType {
            name: Arc::from("radar"),
        };
        assert!(err.to_string().contains("radar"));
        assert!(err.to_string().contains("unsupported chart type"));
    }

    #[test]
    fn test_bc_1_11_001_chart_error_missing_data_field_message() {
        let err = ChartError::MissingDataField {
            field: Arc::from("x"),
        };
        assert!(err.to_string().contains('x'));
        assert!(err.to_string().contains("missing required data field"));
    }

    #[test]
    fn test_bc_1_11_001_chart_error_render_error_message() {
        let err = ChartError::RenderError {
            message: Arc::from("backend failure"),
        };
        assert!(err.to_string().contains("backend failure"));
    }

    /// BC-1.11.002 — `ChartError::EmptyData` variant exists with the correct fields.
    ///
    /// Red Gate: passes at stub time because the variant is a type-level stub
    /// (it compiles). The real value is tested in `validation.rs` tests.
    #[test]
    fn test_bc_1_11_002_chart_error_empty_data_variant_exists() {
        use slideforge_types::SourceSpan;

        let err = ChartError::EmptyData {
            slide_title: Arc::from("Revenue Chart"),
            expression: Arc::from("{{ kpis.monthly }}"),
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("E-LAY-003"),
            "EmptyData error message must contain 'E-LAY-003'; got: {msg}"
        );
        assert!(
            msg.contains("Revenue Chart"),
            "EmptyData error message must contain the slide title; got: {msg}"
        );
    }

    /// BC-1.11.002 AC-001 — `ChartError::EmptyData` Display impl includes `E-LAY-003`.
    ///
    /// Traceability anchor: the named test for test vector 19 in STORY-032.
    ///
    /// This test is structural (verifies the thiserror `#[error(...)]` template)
    /// and passes at stub time. Its purpose is naming — providing a canonical
    /// `test_BC_S_SS_NNN_xxx()` identifier for the `EmptyData` display assertion.
    #[test]
    fn test_bc_1_11_002_chart_error_empty_data_displays_e_lay_003() {
        use slideforge_types::SourceSpan;

        let err = ChartError::EmptyData {
            slide_title: Arc::from("Q3 Dashboard"),
            expression: Arc::from("{{ kpis }}"),
            span: SourceSpan::default(),
        };
        let display = err.to_string();
        assert!(
            display.contains("E-LAY-003"),
            "ChartError::EmptyData Display must include 'E-LAY-003'; got: {display}"
        );
        // Also verify that the expression appears in the Display output, per the
        // thiserror template: "... (expression: {expression})"
        assert!(
            display.contains("kpis"),
            "ChartError::EmptyData Display must include the expression; got: {display}"
        );
    }

    #[test]
    fn test_bc_1_11_001_fallback_palette_has_seven_colors() {
        assert_eq!(InternalChartSpec::FALLBACK_PALETTE.len(), 7);
    }

    #[test]
    fn test_bc_1_11_001_fallback_palette_all_hex_format() {
        for color in InternalChartSpec::FALLBACK_PALETTE {
            assert!(
                color.starts_with('#') && color.len() == 7,
                "palette entry is not a 6-digit hex: {color}"
            );
        }
    }

    #[test]
    fn test_bc_1_11_001_default_dimensions() {
        assert_eq!(InternalChartSpec::DEFAULT_WIDTH, 800);
        assert_eq!(InternalChartSpec::DEFAULT_HEIGHT, 450);
    }

    #[test]
    fn test_bc_1_11_001_data_point_fields() {
        let pt = DataPoint {
            label: Arc::from("Q1"),
            value: 42.0,
        };
        assert_eq!(pt.label.as_ref(), "Q1");
        assert!((pt.value - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bc_1_11_001_data_series_fields() {
        let series = DataSeries {
            name: Arc::from("Revenue"),
            points: vec![
                DataPoint {
                    label: Arc::from("Jan"),
                    value: 100.0,
                },
                DataPoint {
                    label: Arc::from("Feb"),
                    value: 120.0,
                },
            ],
        };
        assert_eq!(series.name.as_ref(), "Revenue");
        assert_eq!(series.points.len(), 2);
    }
}
