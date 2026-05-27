//! Pie chart renderer.
//!
//! Produces an SVG pie chart from an [`InternalChartSpec`] using the
//! `plotters` `SVGBackend`. The first [`crate::types::DataSeries`] supplies
//! the slice values.

use std::f64::consts::PI;
use std::sync::Arc;

use plotters::prelude::*;

use crate::bar::{inject_viewbox, series_color};
use crate::types::{ChartError, InternalChartSpec};

/// Render a pie chart to an SVG string.
///
/// For a single-slice input (EC-002), a full circle arc is rendered.
///
/// # Errors
///
/// Returns [`ChartError`] if the data is invalid or the plotters backend
/// fails to produce output.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]
// Casting f64 coordinates to i32 for plotters drawing API:
// - values are bounded to chart canvas size (< i32::MAX)
// - truncation is intentional for pixel coordinates
// - sign: coordinates are always positive (canvas origin at top-left)
pub fn render_pie(spec: &InternalChartSpec) -> Result<String, ChartError> {
    let width = spec.width;
    let height = spec.height;

    let series = spec.data.first().ok_or_else(|| ChartError::MissingDataField {
        field: Arc::from("data"),
    })?;

    let total: f64 = series.points.iter().map(|p| p.value).sum();
    // If total is zero, treat all slices as equal.
    let n_slices = series.points.len();
    let total = if total == 0.0 { n_slices as f64 } else { total };

    let mut svg_buf = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_buf, (width, height)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| ChartError::RenderError {
            message: Arc::from(e.to_string().as_str()),
        })?;

        // Calculate pie center and radius.
        let cx = f64::from(width) / 2.0;
        let cy = f64::from(height) / 2.0;
        // Leave margin for title and legend.
        let radius = (cy.min(cx) * 0.75).max(1.0);

        let font_name = spec.font_family.as_ref();

        // Draw chart title if provided.
        if let Some(title) = spec.title.as_deref() {
            let title_style = TextStyle::from((font_name, 18).into_font()).color(&BLACK);
            let cx_i32 = i32::try_from(width / 2).unwrap_or(i32::MAX);
            root.draw_text(title, &title_style, (cx_i32 - 60, 10))
                .map_err(|e| ChartError::RenderError {
                    message: Arc::from(e.to_string().as_str()),
                })?;
        }

        if n_slices == 1 {
            // Single slice: render a full circle.
            let color = series_color(&spec.accent_colors, 0);
            root.draw(&Circle::new(
                (cx as i32, cy as i32),
                radius as i32,
                color.filled(),
            ))
            .map_err(|e| ChartError::RenderError {
                message: Arc::from(e.to_string().as_str()),
            })?;
        } else {
            // Multi-slice pie: render polygon approximation of each arc.
            let mut start_angle = -PI / 2.0; // start from 12 o'clock

            for (i, point) in series.points.iter().enumerate() {
                let value = if point.value == 0.0 {
                    1.0 / n_slices as f64
                } else {
                    point.value
                };
                let sweep = 2.0 * PI * (value / total);
                let end_angle = start_angle + sweep;

                let color = series_color(&spec.accent_colors, i);

                // Approximate the arc with a polygon (N segments per slice).
                // n_segments: clamp between 4 and 64 segments per slice.
                let n_segments =
                    ((sweep / (PI / 32.0)) as usize).clamp(4, 64);
                let mut points: Vec<(i32, i32)> = Vec::with_capacity(n_segments + 2);

                // Center of pie.
                points.push((cx as i32, cy as i32));

                // Arc points.
                for seg in 0..=n_segments {
                    let angle =
                        start_angle + (sweep * seg as f64 / n_segments as f64);
                    let px = cx + radius * angle.cos();
                    let py = cy + radius * angle.sin();
                    points.push((px as i32, py as i32));
                }

                root.draw(&Polygon::new(points, color.filled()))
                    .map_err(|e| ChartError::RenderError {
                        message: Arc::from(e.to_string().as_str()),
                    })?;

                start_angle = end_angle;
            }
        }

        root.present().map_err(|e| ChartError::RenderError {
            message: Arc::from(e.to_string().as_str()),
        })?;
    }

    Ok(inject_viewbox(&svg_buf, width, height))
}
