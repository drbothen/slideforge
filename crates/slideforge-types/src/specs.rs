//! Spec types for chart, diagram, shape, image, and table content blocks.
//!
//! These types carry the data needed by renderer and exporter plugins to
//! produce output. They are intentionally minimal in this story; fields will
//! be expanded in subsequent stories (chart series, table cell formatting, etc.).

use std::sync::Arc;

use crate::span::SourceSpan;

/// Specification for a chart content block.
///
/// Full chart data (series, labels, axes) is defined in the chart-spec story.
/// This skeleton carries enough for the type-system tests.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChartSpec {
    /// The chart type keyword (e.g., `"bar"`, `"line"`, `"pie"`).
    pub chart_type: Arc<str>,

    /// Accessibility alt text. Required — compile error if absent (Phase 3 validator).
    pub alt: Arc<str>,

    /// Source location.
    pub span: SourceSpan,
}

/// Specification for a diagram content block (e.g., Mermaid).
///
/// The `source` field holds the raw diagram source (e.g., Mermaid syntax).
/// The [`crate::block::ContentBlock::Diagram`] variant wraps this spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagramSpec {
    /// The raw diagram source code (e.g., `"graph TD; A-->B"`).
    pub source: Arc<str>,

    /// Accessibility alt text. Required.
    pub alt: Arc<str>,

    /// Source location.
    pub span: SourceSpan,
}

/// Specification for a shape content block.
///
/// Shape types, positions, and fills are defined in the shape-spec story.
/// This skeleton carries the minimum needed now.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeSpec {
    /// The shape type keyword (e.g., `"rect"`, `"ellipse"`, `"arrow"`).
    pub shape_type: Arc<str>,

    /// Accessibility alt text. Required.
    pub alt: Arc<str>,

    /// Source location.
    pub span: SourceSpan,
}

/// Specification for an image content block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageSpec {
    /// Path to the image file, relative to the `.sf` source file.
    pub path: Arc<str>,

    /// Accessibility alt text. Required.
    pub alt: Arc<str>,

    /// Source location.
    pub span: SourceSpan,
}

/// Specification for a table content block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableSpec {
    /// Column header cells (each cell is a string for v1.0).
    pub headers: Vec<Arc<str>>,

    /// Table body rows. Each row is a sequence of cell strings.
    pub rows: Vec<Vec<Arc<str>>>,

    /// Accessibility alt text / caption. Required.
    pub alt: Arc<str>,

    /// Source location.
    pub span: SourceSpan,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_bc_1_01_specs_chart_spec_fields() {
        let spec = ChartSpec {
            chart_type: Arc::from("bar"),
            alt: Arc::from("sales by region"),
            span: SourceSpan::default(),
        };
        assert_eq!(spec.chart_type.as_ref(), "bar");
        assert_eq!(spec.alt.as_ref(), "sales by region");
    }

    #[test]
    fn test_bc_1_01_specs_diagram_spec_fields() {
        let spec = DiagramSpec {
            source: Arc::from("graph TD; A-->B"),
            alt: Arc::from("flow diagram"),
            span: SourceSpan::default(),
        };
        assert_eq!(spec.source.as_ref(), "graph TD; A-->B");
    }

    #[test]
    fn test_bc_1_01_specs_shape_spec_fields() {
        let spec = ShapeSpec {
            shape_type: Arc::from("rect"),
            alt: Arc::from("a rectangle"),
            span: SourceSpan::default(),
        };
        assert_eq!(spec.shape_type.as_ref(), "rect");
    }

    #[test]
    fn test_bc_1_01_specs_image_spec_fields() {
        let spec = ImageSpec {
            path: Arc::from("images/logo.png"),
            alt: Arc::from("company logo"),
            span: SourceSpan::default(),
        };
        assert_eq!(spec.path.as_ref(), "images/logo.png");
    }

    #[test]
    fn test_bc_1_01_specs_table_spec_fields() {
        let spec = TableSpec {
            headers: vec![Arc::from("Name"), Arc::from("Value")],
            rows: vec![vec![Arc::from("A"), Arc::from("1")]],
            alt: Arc::from("data table"),
            span: SourceSpan::default(),
        };
        assert_eq!(spec.headers.len(), 2);
        assert_eq!(spec.rows.len(), 1);
    }

    #[test]
    fn test_bc_1_01_specs_all_implement_hash() {
        use std::collections::HashSet;
        let chart = ChartSpec { chart_type: Arc::from("pie"), alt: Arc::from("x"), span: SourceSpan::default() };
        let mut set: HashSet<ChartSpec> = HashSet::new();
        set.insert(chart);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_01_specs_all_implement_clone() {
        let spec = ImageSpec { path: Arc::from("a.png"), alt: Arc::from("a"), span: SourceSpan::default() };
        let spec2 = spec.clone();
        assert_eq!(spec, spec2);
    }
}
