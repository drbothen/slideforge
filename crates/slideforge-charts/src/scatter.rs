//! Scatter chart renderer.
//!
//! Produces an SVG x/y scatter plot from an [`InternalChartSpec`] using the
//! `plotters` `SVGBackend`. Data point `value` fields supply the y-coordinate;
//! x-coordinates are taken from the point index.

use std::sync::Arc;

use plotters::prelude::*;

use crate::bar::{inject_viewbox, series_color};
use crate::types::{ChartError, InternalChartSpec};

/// Render a scatter chart to an SVG string.
///
/// # Errors
///
/// Returns [`ChartError`] if required data fields are missing or the plotters
/// backend fails to produce output.
#[allow(clippy::cast_precision_loss)] // index-to-f64 cast: bounded by data length
pub fn render_scatter(spec: &InternalChartSpec) -> Result<String, ChartError> {
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
        let x_end = n_points as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                spec.title.as_deref().unwrap_or(""),
                (font_name, 18).into_font(),
            )
            .margin(20u32)
            .x_label_area_size(40u32)
            .y_label_area_size(50u32)
            .build_cartesian_2d(0.0..x_end, y_min..y_max)
            .map_err(|e| ChartError::RenderError {
                message: Arc::from(e.to_string().as_str()),
            })?;

        chart
            .configure_mesh()
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

            let data: Vec<(f64, f64)> = series
                .points
                .iter()
                .enumerate()
                .map(|(i, pt)| (i as f64, pt.value))
                .collect();

            chart
                .draw_series(data.iter().map(|(x, y)| Circle::new((*x, *y), 5i32, color.filled())))
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
