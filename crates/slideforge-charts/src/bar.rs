//! Bar chart renderer (vertical, grouped series).
//!
//! Produces an SVG bar chart from an [`InternalChartSpec`] using the
//! `plotters` `SVGBackend`. Series colors are taken from
//! [`InternalChartSpec::accent_colors`], cycling through the fallback
//! palette if needed.

use std::sync::Arc;

use plotters::prelude::*;

use crate::types::{ChartError, InternalChartSpec};

/// Parse a CSS hex color string (e.g., `"#003766"`) into a plotters `RGBColor`.
///
/// Returns a dark fallback color if the string is malformed.
fn parse_hex_color(hex: &str) -> RGBColor {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        RGBColor(r, g, b)
    } else {
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
/// is present with the canonical `"0 0 {width} {height}"` value.
///
/// Plotters `SVGBackend` already emits a `viewBox` attribute. If it is absent
/// (e.g., backend regression), this function injects it before the `>` of the
/// opening `<svg` tag so the chart scales correctly when embedded in PPTX or HTML.
pub(crate) fn inject_viewbox(svg: &str, width: u32, height: u32) -> String {
    // If viewBox is already present (plotters 0.3.x emits it), return as-is.
    if svg.contains("viewBox=") {
        return svg.to_owned();
    }
    // Inject viewBox before the closing > of the opening <svg tag.
    if let Some(tag_end) = svg.find('>') {
        let (tag, rest) = svg.split_at(tag_end);
        format!("{tag} viewBox=\"0 0 {width} {height}\">{rest_tail}", rest_tail = &rest[1..])
    } else {
        svg.to_owned()
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
    let width = spec.width;
    let height = spec.height;

    let mut svg_buf = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_buf, (width, height)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| ChartError::RenderError {
            message: Arc::from(e.to_string().as_str()),
        })?;

        // Compute y-axis range from all data.
        let max_val = spec
            .data
            .iter()
            .flat_map(|s| s.points.iter())
            .map(|p| p.value)
            .fold(0.0_f64, f64::max);
        let y_max = if max_val > 0.0 { max_val * 1.1 } else { 1.0 };

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
            .build_cartesian_2d(0u32..x_range_end, 0.0..y_max)
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
                .draw_series(data.iter().map(|(x, y)| {
                    Rectangle::new([(*x, 0.0), (*x + 1, *y)], filled)
                }))
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
