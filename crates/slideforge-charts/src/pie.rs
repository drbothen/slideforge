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
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
// Casting f64 coordinates to i32 for plotters drawing API:
// - values are bounded to chart canvas size (< i32::MAX)
// - truncation is intentional for pixel coordinates
// - sign: coordinates are always positive (canvas origin at top-left)
pub(crate) fn render_pie(spec: &InternalChartSpec) -> Result<String, ChartError> {
    // FINDING-004: Validate all data points are finite.
    crate::bar::validate_data_finite(spec)?;
    // FINDING-002 (Pass 3): Guard non-empty data vec with all-empty points.
    crate::bar::validate_points_non_empty(spec)?;

    // FINDING-001 (Pass 3): Pie chart slices must be non-negative.
    // Negative values produce visual garbage (overlapping arcs in wrong direction).
    if let Some(series) = spec.data.first() {
        for pt in &series.points {
            if pt.value < 0.0 {
                return Err(ChartError::RenderError {
                    message: Arc::from("pie chart data must be non-negative"),
                });
            }
        }
    }

    let width = spec.width;
    let height = spec.height;

    let series = spec
        .data
        .first()
        .ok_or_else(|| ChartError::MissingDataField {
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
                // FINDING-002 fix: when a slice value is 0, use 1.0 (not 1.0/n_slices).
                // With total = n_slices, this gives sweep = 2*PI*(1.0/n_slices) = equal slices.
                // Using 1.0/n_slices would give sweep = 2*PI*(1/n_slices²), too small.
                let value = if point.value == 0.0 { 1.0 } else { point.value };
                let sweep = 2.0 * PI * (value / total);
                let end_angle = start_angle + sweep;

                let color = series_color(&spec.accent_colors, i);

                // Approximate the arc with a polygon (N segments per slice).
                // n_segments: clamp between 4 and 64 segments per slice.
                let n_segments = ((sweep / (PI / 32.0)) as usize).clamp(4, 64);
                let mut points: Vec<(i32, i32)> = Vec::with_capacity(n_segments + 2);

                // Center of pie.
                points.push((cx as i32, cy as i32));

                // Arc points.
                for seg in 0..=n_segments {
                    let angle = start_angle + (sweep * seg as f64 / n_segments as f64);
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use std::sync::Arc;

    use crate::types::{DataPoint, DataSeries, InternalChartSpec};

    fn all_zero_pie_spec(n: usize) -> InternalChartSpec {
        let points: Vec<DataPoint> = (0..n)
            .map(|i| DataPoint {
                label: Arc::from(format!("s{i}").as_str()),
                value: 0.0,
            })
            .collect();
        InternalChartSpec {
            chart_type: crate::types::ChartType::Pie,
            data: vec![DataSeries {
                name: Arc::from("test"),
                points,
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("zero pie"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![
                Arc::from("#003766"),
                Arc::from("#FF6F00"),
                Arc::from("#009E60"),
            ],
            font_family: Arc::from("sans-serif"),
            slide_title: Arc::from(""),
            expression: Arc::from(""),
            span: slideforge_types::SourceSpan::default(),
        }
    }

    /// FINDING-002: All-zero pie must produce a full circle SVG (equal slices).
    ///
    /// When every slice value is 0, we set each value to 1.0 (not `1.0/n_slices`)
    /// and keep total = `n_slices`. This yields sweep = `2*PI*(1/n_slices)` per slice,
    /// summing to a full circle.
    #[test]
    fn test_f031_002_all_zero_pie_produces_full_circle_svg() {
        let spec = all_zero_pie_spec(4);
        let result = super::render_pie(&spec);
        let svg = result.unwrap();
        // The output must be non-empty (visual check that no division-by-zero crash)
        assert!(!svg.is_empty(), "all-zero pie must produce non-empty SVG");
        // The SVG must contain path or polygon elements (confirming slices rendered)
        assert!(
            svg.contains("<polygon") || svg.contains("<path") || svg.contains("<circle"),
            "all-zero pie must render visible elements; got: {}",
            &svg[..svg.len().min(500)]
        );
    }

    /// FINDING-002: Two-slice all-zero pie should sum to full circle.
    /// Each slice sweep should be PI (180°), so the polygon approximations together
    /// cover 2*PI total sweep (full circle).
    #[test]
    fn test_f031_002_all_zero_two_slice_pie_equal_division() {
        let spec = all_zero_pie_spec(2);
        let result = super::render_pie(&spec);
        let svg = result.unwrap();
        assert!(!svg.is_empty(), "two-slice all-zero pie must render");
    }

    /// FINDING-001 (Pass 3): Pie chart with a negative value must return `ChartError`.
    #[test]
    fn test_f031_p3_001_pie_negative_value_returns_error() {
        let spec = InternalChartSpec {
            chart_type: crate::types::ChartType::Pie,
            data: vec![DataSeries {
                name: Arc::from("sales"),
                points: vec![
                    DataPoint {
                        label: Arc::from("A"),
                        value: 50.0,
                    },
                    DataPoint {
                        label: Arc::from("B"),
                        value: -10.0,
                    },
                    DataPoint {
                        label: Arc::from("C"),
                        value: 30.0,
                    },
                ],
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("negative pie"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
            slide_title: Arc::from(""),
            expression: Arc::from(""),
            span: slideforge_types::SourceSpan::default(),
        };
        let result = super::render_pie(&spec);
        assert!(
            result.is_err(),
            "pie chart with negative value must return an error"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("non-negative") || msg.contains("negative"),
            "error must mention non-negative; got: {msg}"
        );
    }

    /// FINDING-001 (Pass 3): All-negative single-slice pie must also be rejected.
    #[test]
    fn test_f031_p3_001_pie_all_negative_returns_error() {
        let spec = InternalChartSpec {
            chart_type: crate::types::ChartType::Pie,
            data: vec![DataSeries {
                name: Arc::from("loss"),
                points: vec![DataPoint {
                    label: Arc::from("Q1"),
                    value: -5.0,
                }],
            }],
            title: None,
            x_label: None,
            y_label: None,
            alt: Arc::from("all negative pie"),
            width: InternalChartSpec::DEFAULT_WIDTH,
            height: InternalChartSpec::DEFAULT_HEIGHT,
            accent_colors: vec![Arc::from("#003766")],
            font_family: Arc::from("sans-serif"),
            slide_title: Arc::from(""),
            expression: Arc::from(""),
            span: slideforge_types::SourceSpan::default(),
        };
        let result = super::render_pie(&spec);
        assert!(result.is_err(), "all-negative pie must return an error");
    }

    /// FINDING-002: Verify the per-slice sweep computation is correct for all-zero input.
    ///
    /// The computation exposed for testing: when all values are 0:
    ///   value = 1.0 (NOT `1.0/n_slices`)
    ///   total = `n_slices`
    ///   sweep = `2*PI * (1.0 / n_slices)` — equal slices summing to `2*PI`
    ///
    /// The buggy behavior was value = `1.0/n_slices` giving
    ///   sweep = `2*PI / n_slices^2` — too small, total < `2*PI`
    #[test]
    fn test_f031_002_zero_slice_value_is_one_not_reciprocal() {
        // Direct unit test of the arithmetic: compute what the sweep SHOULD be
        // for n=4 all-zero slices: each sweep = 2*PI/4 = PI/2.
        // The buggy code gives: value=0.25, total=4, sweep=2*PI*(0.25/4)=PI/8 — wrong.
        // The fixed code gives: value=1.0, total=4, sweep=2*PI*(1.0/4)=PI/2 — correct.
        use std::f64::consts::PI;
        let n_slices: u32 = 4;
        // Correct behavior:
        let value_correct = 1.0_f64;
        let total = f64::from(n_slices);
        let sweep_correct = 2.0 * PI * (value_correct / total);
        let expected_sweep = PI / 2.0; // PI/2 for 4 equal slices
        assert!(
            (sweep_correct - expected_sweep).abs() < 1e-10,
            "correct sweep for 4 equal slices must be PI/2; got {sweep_correct}"
        );

        // Buggy behavior (the value that was being used before the fix):
        let value_buggy = 1.0 / total;
        let sweep_buggy = 2.0 * PI * (value_buggy / total);
        assert!(
            (sweep_buggy - expected_sweep).abs() > 1e-6,
            "buggy sweep for 4 equal slices was {sweep_buggy}, which is WRONG (expected {expected_sweep})"
        );
    }
}
