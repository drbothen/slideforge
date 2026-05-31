//! [`ChartRenderer`] trait — renders a [`ChartSpec`] to SVG bytes.
//!
//! A `ChartRenderer` plugin takes a [`ChartSpec`] (chart type, series, labels,
//! axes) and a resolved [`Brand`] and produces an SVG byte buffer. The built-in
//! chart renderer uses the `plotters` crate (pure Rust, no native deps). External
//! plugins can register alternative renderers (e.g., a server-side Vega renderer).

use slideforge_types::{Brand, ChartSpec};
use thiserror::Error;

/// Error returned by [`ChartRenderer::render`].
///
/// `#[non_exhaustive]` allows adding variants in minor releases without
/// breaking external plugin authors who pattern-match this enum (SemVer hygiene).
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ChartError {
    /// The chart type in the spec is not supported by this renderer.
    #[error("unsupported chart type '{chart_type}'")]
    UnsupportedChartType {
        /// The chart type keyword from the spec (e.g., `"treemap"`).
        chart_type: String,
    },

    /// The chart spec contains invalid or inconsistent data (e.g., mismatched
    /// series lengths, missing required fields).
    #[error("invalid chart spec: {message}")]
    InvalidSpec {
        /// A description of the spec problem.
        message: String,
    },

    /// An internal rendering error occurred (e.g., an SVG serialization failure).
    #[error("chart rendering failed: {message}")]
    RenderError {
        /// A description of the rendering failure.
        message: String,
    },
}

/// A plugin that renders a [`ChartSpec`] to SVG bytes.
///
/// The SVG bytes are embedded in PPTX (as an OLE object or image) and rendered
/// directly in HTML and PDF output. The SVG produced must be self-contained (no
/// external font or image references that depend on the file system).
///
/// Register implementations with [`crate::PluginRegistry::register_chart_renderer`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
pub trait ChartRenderer: Send + Sync {
    /// A unique identifier for this chart renderer plugin (e.g., `"plotters"`).
    fn id(&self) -> &str;

    /// Render `spec` to SVG bytes using `brand` colors and typography.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError`] when the chart cannot be rendered.
    fn render(&self, spec: &ChartSpec, brand: &Brand) -> Result<Vec<u8>, ChartError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_003_chart_renderer_trait_is_send_sync() {
        assert_send_sync::<dyn ChartRenderer>();
    }

    #[test]
    fn test_bc_5_02_003_chart_error_unsupported_chart_type() {
        let err = ChartError::UnsupportedChartType {
            chart_type: "treemap".to_owned(),
        };
        assert!(err.to_string().contains("unsupported chart type"));
    }

    #[test]
    fn test_bc_5_02_003_chart_error_invalid_spec() {
        let err = ChartError::InvalidSpec {
            message: "series length mismatch".to_owned(),
        };
        assert!(err.to_string().contains("invalid chart spec"));
    }

    #[test]
    fn test_bc_5_02_003_chart_error_render_error() {
        let err = ChartError::RenderError {
            message: "SVG output error".to_owned(),
        };
        assert!(err.to_string().contains("chart rendering failed"));
    }
}
