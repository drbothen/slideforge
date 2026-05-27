//! Histogram renderer (frequency distribution bars).
//!
//! Produces an SVG histogram from an [`InternalChartSpec`] using the
//! `plotters` `SVGBackend`. The first series' values are treated as frequencies
//! for the labeled bins.

use std::sync::Arc;

use plotters::prelude::*;

use crate::bar::{inject_viewbox, series_color};
use crate::types::{ChartError, InternalChartSpec};

/// Render a histogram to an SVG string.
///
/// # Errors
///
/// Returns [`ChartError`] if the data is invalid or the plotters backend
/// fails to produce output.
pub fn render_histogram(spec: &InternalChartSpec) -> Result<String, ChartError> {
    let width = spec.width;
    let height = spec.height;

    let mut svg_buf = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_buf, (width, height)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| ChartError::RenderError {
            message: Arc::from(e.to_string().as_str()),
        })?;

        // Use the first series as frequency data (histogram treats each bin as
        // a labeled bar with contiguous bins — no gaps between bars).
        let series = spec.data.first().ok_or_else(|| ChartError::MissingDataField {
            field: Arc::from("data"),
        })?;

        let max_val = series
            .points
            .iter()
            .map(|p| p.value)
            .fold(0.0_f64, f64::max);
        let y_max = if max_val > 0.0 { max_val * 1.1 } else { 1.0 };

        let n_bins = series.points.len();
        let color = series_color(&spec.accent_colors, 0);
        let font_name = spec.font_family.as_ref();

        let x_end = u32::try_from(n_bins).unwrap_or(u32::MAX);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                spec.title.as_deref().unwrap_or(""),
                (font_name, 18).into_font(),
            )
            .margin(20u32)
            .x_label_area_size(40u32)
            .y_label_area_size(50u32)
            .build_cartesian_2d(0u32..x_end, 0.0..y_max)
            .map_err(|e| ChartError::RenderError {
                message: Arc::from(e.to_string().as_str()),
            })?;

        chart
            .configure_mesh()
            .x_labels(n_bins.min(10))
            .x_label_formatter(&|v| {
                series
                    .points
                    .get(*v as usize)
                    .map_or_else(String::new, |p| p.label.as_ref().to_owned())
            })
            .y_desc(spec.y_label.as_deref().unwrap_or("Frequency"))
            .x_desc(spec.x_label.as_deref().unwrap_or(""))
            .axis_desc_style((font_name, 14).into_font())
            .label_style((font_name, 12).into_font())
            .draw()
            .map_err(|e| ChartError::RenderError {
                message: Arc::from(e.to_string().as_str()),
            })?;

        // Draw contiguous histogram bars (no gap between bins).
        chart
            .draw_series(series.points.iter().enumerate().map(|(i, pt)| {
                let x = u32::try_from(i).unwrap_or(u32::MAX);
                Rectangle::new([(x, 0.0), (x + 1, pt.value)], color.filled())
            }))
            .map_err(|e| ChartError::RenderError {
                message: Arc::from(e.to_string().as_str()),
            })?;

        root.present().map_err(|e| ChartError::RenderError {
            message: Arc::from(e.to_string().as_str()),
        })?;
    }

    Ok(inject_viewbox(&svg_buf, width, height))
}
