//! Line chart renderer.
//!
//! Produces an SVG line chart from an [`InternalChartSpec`] using the
//! `plotters` `SVGBackend`. Each [`crate::types::DataSeries`] becomes a
//! separate colored line.

use std::sync::Arc;

use plotters::prelude::*;

use crate::bar::{inject_viewbox, series_color};
use crate::types::{ChartError, InternalChartSpec};

/// Render a line chart to an SVG string.
///
/// # Errors
///
/// Returns [`ChartError`] if the data is invalid or the plotters backend
/// fails to produce output.
#[allow(clippy::cast_possible_truncation)] // coordinate index cast: n_points bounded by data len
pub fn render_line(spec: &InternalChartSpec) -> Result<String, ChartError> {
    // FINDING-006: Guard empty data early.
    if spec.data.is_empty() {
        return Err(ChartError::MissingDataField { field: Arc::from("data") });
    }
    // FINDING-002 (Pass 3): Guard non-empty data vec with all-empty points.
    crate::bar::validate_points_non_empty(spec)?;
    // FINDING-004: Validate all data points are finite.
    crate::bar::validate_data_finite(spec)?;

    let width = spec.width;
    let height = spec.height;

    let mut svg_buf = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_buf, (width, height)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| ChartError::RenderError {
            message: Arc::from(e.to_string().as_str()),
        })?;

        // FINDING-001 (Pass 2): compute y-range from data, supporting negative values.
        let (y_min, y_max) = crate::bar::compute_y_range(spec);

        let n_points = spec.data.first().map_or(1, |s| s.points.len());
        let font_name = spec.font_family.as_ref();

        let x_end = u32::try_from(n_points.saturating_sub(1).max(1)).unwrap_or(u32::MAX);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                spec.title.as_deref().unwrap_or(""),
                (font_name, 18).into_font(),
            )
            .margin(20u32)
            .x_label_area_size(40u32)
            .y_label_area_size(50u32)
            .build_cartesian_2d(0u32..x_end, y_min..y_max)
            .map_err(|e| ChartError::RenderError {
                message: Arc::from(e.to_string().as_str()),
            })?;

        chart
            .configure_mesh()
            .x_labels(n_points.min(10))
            .x_label_formatter(&|v| {
                spec.data
                    .first()
                    .and_then(|s| s.points.get(*v as usize))
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

        for (series_idx, series) in spec.data.iter().enumerate() {
            let color = series_color(&spec.accent_colors, series_idx);

            let data: Vec<(u32, f64)> = series
                .points
                .iter()
                .enumerate()
                .map(|(i, pt)| (u32::try_from(i).unwrap_or(u32::MAX), pt.value))
                .collect();

            chart
                .draw_series(LineSeries::new(data, color.stroke_width(2)))
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
