//! Bar chart renderer (vertical, grouped series).
//!
//! Produces an SVG bar chart from an [`InternalChartSpec`] using the
//! `plotters` `SVGBackend`. Series colors are taken from
//! [`InternalChartSpec::accent_colors`], cycling through the fallback
//! palette if needed.

use std::sync::Arc;

use plotters::prelude::*;

use crate::types::{ChartError, InternalChartSpec};

/// Validate that all data point values in the spec are finite (not NaN or infinite).
///
/// FINDING-004: NaN or infinite values cause silent misrenders or panics in plotters.
///
/// # Errors
///
/// Returns [`ChartError::RenderError`] if any data point contains a non-finite value.
pub(crate) fn validate_data_finite(spec: &InternalChartSpec) -> Result<(), ChartError> {
    for series in &spec.data {
        for pt in &series.points {
            if !pt.value.is_finite() {
                return Err(ChartError::RenderError {
                    message: Arc::from(
                        "data contains NaN or infinite value — chart cannot be rendered",
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Validate that all series in the spec have at least one data point.
///
/// FINDING-002 (Pass 3): A non-empty data vec where every series has zero points
/// produces a degenerate chart (empty axes, invisible rendering). Reject early.
///
/// # Errors
///
/// Returns [`ChartError::MissingDataField`] with `field = "data.points"` if
/// every series has an empty points vec.
pub(crate) fn validate_points_non_empty(spec: &InternalChartSpec) -> Result<(), ChartError> {
    if spec.data.iter().all(|s| s.points.is_empty()) {
        return Err(ChartError::MissingDataField {
            field: Arc::from("data.points"),
        });
    }
    Ok(())
}

/// Parse a CSS hex color string (e.g., `"#003766"`) into a plotters `RGBColor`.
///
/// ## Fallback behavior (FINDING-009)
///
/// When the input is not a valid 6-digit hex color (e.g., a malformed brand
/// palette entry), this function returns `RGBColor(0, 55, 102)` — slideforge's
/// dark navy blue. This is an intentional graceful-degradation path: chart
/// output remains visible even with a bad brand config.
///
/// In debug builds, a `debug_assert!` will panic if the input is not a valid
/// 6-digit hex, so misconfigured test fixtures are caught immediately.
///
/// In production, the fallback color is applied silently. Callers that need
/// strict validation should validate the brand palette before calling this
/// function (e.g., in the brand-provider plugin).
fn parse_hex_color(hex: &str) -> RGBColor {
    let stripped = hex.trim_start_matches('#');
    if stripped.len() == 6 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        let r = u8::from_str_radix(&stripped[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&stripped[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&stripped[4..6], 16).unwrap_or(0);
        RGBColor(r, g, b)
    } else {
        // FINDING-009: Malformed color — use fallback and assert in debug builds.
        debug_assert!(
            false,
            "parse_hex_color: malformed color '{hex}' — expected 6-digit hex (e.g. '#003766'). \
             Using fallback RGBColor(0, 55, 102). Fix the brand palette configuration."
        );
        RGBColor(0, 55, 102) // fallback: slideforge dark blue
    }
}

/// Choose the color for series index `i` from `accent_colors`, cycling
/// through the fallback palette if needed.
pub(crate) fn series_color(accent_colors: &[Arc<str>], i: usize) -> RGBColor {
    let palette: Vec<&str> = if accent_colors.is_empty() {
        InternalChartSpec::FALLBACK_PALETTE.to_vec()
    } else {
        accent_colors.iter().map(Arc::as_ref).collect()
    };
    let hex = palette[i % palette.len()];
    parse_hex_color(hex)
}

/// Post-process a raw SVG string from plotters to ensure the `viewBox` attribute
/// is present with the canonical `"0 0 {width} {height}"` value, and that the
/// `width` and `height` attributes use the `px` suffix (e.g., `width="800px"`).
///
/// AC-004 requires absolute `px` dimensions. Plotters `SVGBackend` emits
/// bare integers (`width="800"`). This function:
/// 1. Replaces `width="{width}"` with `width="{width}px"` in the opening `<svg>` tag.
/// 2. Replaces `height="{height}"` with `height="{height}px"` in the opening `<svg>` tag.
/// 3. Injects `viewBox="0 0 {width} {height}"` if absent.
///
/// Only the opening `<svg` tag is modified (not internal `<rect>` or `<image>` elements).
pub(crate) fn inject_viewbox(svg: &str, width: u32, height: u32) -> String {
    // Find the extent of the opening <svg tag (from '<svg' to its first '>').
    let Some(svg_tag_start) = svg.find("<svg") else {
        return svg.to_owned();
    };
    let Some(tag_rel_close) = svg[svg_tag_start..].find('>') else {
        return svg.to_owned();
    };
    let tag_close = svg_tag_start + tag_rel_close;

    // Work on only the opening <svg ... > portion.
    let (before_svg, rest_from_svg) = svg.split_at(svg_tag_start);
    let (svg_tag, after_tag) = rest_from_svg.split_at(tag_close - svg_tag_start + 1);

    // Add px suffix to bare width and height attributes within the opening tag.
    let bare_width = format!("width=\"{width}\"");
    let px_width = format!("width=\"{width}px\"");
    let bare_height = format!("height=\"{height}\"");
    let px_height = format!("height=\"{height}px\"");

    let svg_tag = svg_tag.replace(&bare_width, &px_width);
    let svg_tag = svg_tag.replace(&bare_height, &px_height);

    // Inject viewBox if absent (plotters 0.3.x emits it, but guard for regressions).
    let svg_tag = if svg_tag.contains("viewBox=") {
        svg_tag
    } else {
        // Insert viewBox before the closing > of the tag.
        if let Some(close_pos) = svg_tag.rfind('>') {
            let (before_close, from_close) = svg_tag.split_at(close_pos);
            format!("{before_close} viewBox=\"0 0 {width} {height}\"{from_close}")
        } else {
            svg_tag
        }
    };

    format!("{before_svg}{svg_tag}{after_tag}")
}

/// Compute the y-axis range `(y_min, y_max)` from all data points in the spec.
///
/// FINDING-001 (Pass 2): The previous implementation hardcoded the lower bound at `0.0`,
/// which caused all-negative data to fall below the visible chart area.
///
/// Rules:
/// - `y_min`: if any value is negative, use `min_val * 1.1` (10% pad below); otherwise `0.0`.
/// - `y_max`: if any value is positive, use `max_val * 1.1` (10% pad above); otherwise `0.0`.
///   When all data is negative (or zero), `y_max` is set to `0.0` so the zero baseline stays
///   visible. The non-degenerate guard below will widen a degenerate `(0.0, 0.0)` range to
///   `(-1.0, 1.0)` when all values are exactly zero.
///
/// The returned range is always non-degenerate (`y_min < y_max`).
pub(crate) fn compute_y_range(spec: &InternalChartSpec) -> (f64, f64) {
    let all_points: Vec<f64> = spec
        .data
        .iter()
        .flat_map(|s| s.points.iter())
        .map(|p| p.value)
        .collect();

    if all_points.is_empty() {
        return (0.0, 1.0);
    }

    let min_val = all_points.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = all_points.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let y_min = if min_val < 0.0 { min_val * 1.1 } else { 0.0 };
    let y_max = if max_val > 0.0 { max_val * 1.1 } else { 0.0 };

    // Ensure range is non-degenerate (min < max).
    if (y_max - y_min).abs() < f64::EPSILON {
        (y_min - 1.0, y_min + 1.0)
    } else {
        (y_min, y_max)
    }
}

/// Render a vertical bar chart to an SVG string.
///
/// # Errors
///
/// Returns [`ChartError`] if the data is invalid or the plotters backend
/// fails to produce output.
#[allow(clippy::cast_possible_truncation)] // coordinate cast: chart dimensions < u32::MAX
pub fn render_bar(spec: &InternalChartSpec) -> Result<String, ChartError> {
    // FINDING-006: Guard empty data early.
    if spec.data.is_empty() {
        return Err(ChartError::MissingDataField {
            field: Arc::from("data"),
        });
    }
    // FINDING-002 (Pass 3): Guard non-empty data vec with all-empty points.
    validate_points_non_empty(spec)?;
    // FINDING-004: Validate all data points are finite.
    validate_data_finite(spec)?;

    let width = spec.width;
    let height = spec.height;

    let mut svg_buf = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_buf, (width, height)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| ChartError::RenderError {
            message: Arc::from(e.to_string().as_str()),
        })?;

        // Compute y-axis range from all data (FINDING-001 Pass 2: supports negative values).
        let (y_min, y_max) = compute_y_range(spec);

        // Number of categories (x-axis buckets) from first series.
        let n_categories = spec.data.first().map_or(0, |s| s.points.len());
        let n_series = spec.data.len();

        let font_name = spec.font_family.as_ref();

        let x_range_end = u32::try_from(n_categories * n_series.max(1)).unwrap_or(u32::MAX);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                spec.title.as_deref().unwrap_or(""),
                (font_name, 18).into_font(),
            )
            .margin(20u32)
            .x_label_area_size(40u32)
            .y_label_area_size(50u32)
            .build_cartesian_2d(0u32..x_range_end, y_min..y_max)
            .map_err(|e| ChartError::RenderError {
                message: Arc::from(e.to_string().as_str()),
            })?;

        chart
            .configure_mesh()
            .x_labels(n_categories)
            .x_label_formatter(&|v| {
                let cat_idx = (*v as usize) / n_series.max(1);
                spec.data
                    .first()
                    .and_then(|s| s.points.get(cat_idx))
                    .map_or_else(String::new, |p| p.label.as_ref().to_owned())
            })
            .y_desc(spec.y_label.as_deref().unwrap_or(""))
            .x_desc(spec.x_label.as_deref().unwrap_or(""))
            .axis_desc_style((font_name, 14).into_font())
            .label_style((font_name, 12).into_font())
            .draw()
            .map_err(|e| ChartError::RenderError {
                message: Arc::from(e.to_string().as_str()),
            })?;

        // Draw bars for each series.
        for (series_idx, series) in spec.data.iter().enumerate() {
            let color = series_color(&spec.accent_colors, series_idx);
            let filled = color.filled();

            let data: Vec<(u32, f64)> = series
                .points
                .iter()
                .enumerate()
                .map(|(cat_idx, pt)| {
                    let x = u32::try_from(cat_idx * n_series + series_idx).unwrap_or(u32::MAX);
                    (x, pt.value)
                })
                .collect();

            chart
                .draw_series(
                    data.iter()
                        .map(|(x, y)| Rectangle::new([(*x, 0.0), (*x + 1, *y)], filled)),
                )
                .map_err(|e| ChartError::RenderError {
                    message: Arc::from(e.to_string().as_str()),
                })?;
        }

        root.present().map_err(|e| ChartError::RenderError {
            message: Arc::from(e.to_string().as_str()),
        })?;
    }

    Ok(inject_viewbox(&svg_buf, width, height))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use crate::types::{ChartType, DataPoint, DataSeries, InternalChartSpec};

    fn three_point_spec() -> InternalChartSpec {
        InternalChartSpec {
            chart_type: ChartType::Bar,
            data: vec![DataSeries {
                name: Arc::from("Revenue"),
                points: vec![
                    DataPoint {
                        label: Arc::from("Jan"),
                        value: 100.0,
                    },
                    DataPoint {
                        label: Arc::from("Feb"),
                        value: 150.0,
                    },
                    DataPoint {
                        label: Arc::from("Mar"),
                        value: 120.0,
                    },
                ],
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("Revenue chart"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
            slide_title: Arc::from(""),
            expression: Arc::from(""),
            span: slideforge_types::SourceSpan::default(),
        }
    }

    /// FINDING-003: SVG output must contain width="800px" and height="450px".
    #[test]
    fn test_f031_003_bar_svg_has_px_dimensions() {
        let spec = three_point_spec();
        let svg = super::render_bar(&spec).unwrap();
        assert!(
            svg.contains("width=\"800px\""),
            "bar SVG must contain width=\"800px\"; got: {}",
            &svg[..svg.len().min(300)]
        );
        assert!(
            svg.contains("height=\"450px\""),
            "bar SVG must contain height=\"450px\"; got: {}",
            &svg[..svg.len().min(300)]
        );
    }

    /// FINDING-006: Empty data series must return `MissingDataField` error for bar chart.
    #[test]
    fn test_f031_006_bar_empty_data_returns_error() {
        let spec = InternalChartSpec {
            chart_type: ChartType::Bar,
            data: vec![],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("empty"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
            slide_title: Arc::from(""),
            expression: Arc::from(""),
            span: slideforge_types::SourceSpan::default(),
        };
        let result = super::render_bar(&spec);
        assert!(
            result.is_err(),
            "empty data must return an error for bar chart"
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("missing") || msg.contains("data"),
            "error must mention missing data; got: {msg}"
        );
    }

    /// FINDING-009: Malformed hex color falls back to dark blue (not panic in release).
    ///
    /// Note: in debug builds this would trigger a `debug_assert!` panic.
    /// This test verifies the production fallback path using release-mode logic.
    #[test]
    #[cfg(not(debug_assertions))]
    fn test_f031_009_malformed_hex_falls_back_to_dark_blue() {
        // In release mode, malformed colors silently fall back to RGBColor(0, 55, 102).
        let fallback = super::parse_hex_color("not-a-color");
        assert_eq!(fallback, plotters::prelude::RGBColor(0, 55, 102));
        let fallback_empty = super::parse_hex_color("");
        assert_eq!(fallback_empty, plotters::prelude::RGBColor(0, 55, 102));
    }

    /// FINDING-009: Valid 6-digit hex parses correctly.
    #[test]
    fn test_f031_009_valid_hex_parses_correctly() {
        use plotters::prelude::RGBColor;
        let color = super::parse_hex_color("#003766");
        assert_eq!(color, RGBColor(0x00, 0x37, 0x66));
        let color2 = super::parse_hex_color("#FF6F00");
        assert_eq!(color2, RGBColor(0xFF, 0x6F, 0x00));
    }

    /// FINDING-007: 3 series with only 2 brand colors must cycle — 3rd series uses colors[0].
    #[test]
    fn test_f031_007_color_cycling_third_series_uses_first_color() {
        let colors = vec![Arc::from("#003766"), Arc::from("#FF6F00")];
        let c0 = super::series_color(&colors, 0);
        let c2 = super::series_color(&colors, 2);
        // Index 0 and index 2 must be the same color (cycling with len=2)
        assert_eq!(c0, c2, "3rd series (index 2) must cycle back to colors[0]");
        let c1 = super::series_color(&colors, 1);
        assert_ne!(c0, c1, "series 0 and series 1 should have different colors");
    }
}
