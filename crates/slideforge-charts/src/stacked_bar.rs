//! Stacked bar chart renderer.
//!
//! Produces an SVG stacked vertical bar chart from an [`InternalChartSpec`]
//! using the `plotters` `SVGBackend`. Multiple series are stacked within each
//! x-axis category bar.

use std::sync::Arc;

use plotters::prelude::*;

use crate::bar::{inject_viewbox, series_color};
use crate::types::{ChartError, InternalChartSpec};

/// Render a stacked bar chart to an SVG string.
///
/// # Errors
///
/// Returns [`ChartError`] if the data is invalid or the plotters backend
/// fails to produce output.
pub fn render_stacked_bar(spec: &InternalChartSpec) -> Result<String, ChartError> {
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

        let n_categories = spec.data.first().map_or(0, |s| s.points.len());

        // For stacked bars, compute y-range from column sums to represent the full
        // stacked extent. FINDING-001 (Pass 2): support negative values by tracking
        // both positive stack tops and negative stack bottoms per category.
        let mut stack_min = 0.0_f64;
        let mut stack_max = 0.0_f64;
        for cat in 0..n_categories {
            let mut pos_sum = 0.0_f64;
            let mut neg_sum = 0.0_f64;
            for series in &spec.data {
                let v = series.points.get(cat).map_or(0.0, |p| p.value);
                if v >= 0.0 {
                    pos_sum += v;
                } else {
                    neg_sum += v;
                }
            }
            stack_max = stack_max.max(pos_sum);
            stack_min = stack_min.min(neg_sum);
        }
        let y_min = if stack_min < 0.0 { stack_min * 1.1 } else { 0.0 };
        let y_max = if stack_max > 0.0 { stack_max * 1.1 } else { 0.0 };
        // Ensure non-degenerate range.
        let (y_min, y_max) = if (y_max - y_min).abs() < f64::EPSILON {
            (y_min - 1.0, y_min + 1.0)
        } else {
            (y_min, y_max)
        };

        let font_name = spec.font_family.as_ref();

        let x_end = u32::try_from(n_categories).unwrap_or(u32::MAX);

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
            .x_labels(n_categories)
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

        // Draw stacked bars: accumulate bottom offsets per category.
        for (series_idx, series) in spec.data.iter().enumerate() {
            let color = series_color(&spec.accent_colors, series_idx);

            // Compute the cumulative base for this series (sum of all prior series).
            let data: Vec<(u32, f64, f64)> = series
                .points
                .iter()
                .enumerate()
                .map(|(cat_idx, pt)| {
                    let base: f64 = spec.data[..series_idx]
                        .iter()
                        .map(|s| s.points.get(cat_idx).map_or(0.0, |p| p.value))
                        .sum();
                    let x = u32::try_from(cat_idx).unwrap_or(u32::MAX);
                    (x, base, base + pt.value)
                })
                .collect();

            chart
                .draw_series(data.iter().map(|(x, base, top)| {
                    Rectangle::new([(*x, *base), (*x + 1, *top)], color.filled())
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
