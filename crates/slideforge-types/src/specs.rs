//! Spec types for chart, diagram, shape, image, and table content blocks.
//!
//! These types carry the data needed by renderer and exporter plugins to
//! produce output. They are intentionally minimal in this story; fields will
//! be expanded in subsequent stories (chart series, table cell formatting, etc.).

use std::sync::Arc;

use crate::span::SourceSpan;

/// The alt text state of a visual element.
///
/// Visual elements (images, charts, diagrams, shapes) require alt text for
/// WCAG AA accessibility. The validator enforces that every visual element
/// has either non-empty alt text or is explicitly marked decorative.
///
/// - `Provided(s)` — non-empty, non-whitespace alt text supplied by the author.
/// - `Decorative` — element explicitly marked `decorative: true`; the validator
///   emits an empty alt string and a PDF Artifact tag in export.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AltText {
    /// Non-empty, non-whitespace alt text provided by the author.
    Provided(Arc<str>),
    /// Element explicitly marked `decorative: true`.
    Decorative,
}

/// Specification for a chart content block.
///
/// Full chart data (series, labels, axes) is defined in the chart-spec story.
/// This skeleton carries enough for the type-system tests.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChartSpec {
    /// The chart type keyword (e.g., `"bar"`, `"line"`, `"pie"`).
    pub chart_type: Arc<str>,

    /// Accessibility alt text state.
    ///
    /// `None` means the author has not supplied alt text or `decorative: true`;
    /// the validator will emit an `E-A11-001` diagnostic.
    /// `Some(AltText::Provided(_))` means valid alt text is present.
    /// `Some(AltText::Decorative)` means the author explicitly opted out of alt text.
    pub alt: Option<AltText>,

    /// When `true`, the author set `decorative: true` on this element.
    ///
    /// If both `alt` is `Some(AltText::Provided(_))` AND `decorative` is `true`,
    /// the validator emits `W-A11-001` (alt text provided but decorative wins).
    pub decorative: bool,

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

    /// Accessibility alt text state. See [`ChartSpec::alt`] for semantics.
    pub alt: Option<AltText>,

    /// When `true`, the author set `decorative: true` on this element.
    pub decorative: bool,

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

    /// Accessibility alt text state. See [`ChartSpec::alt`] for semantics.
    pub alt: Option<AltText>,

    /// When `true`, the author set `decorative: true` on this element.
    pub decorative: bool,

    /// Source location.
    pub span: SourceSpan,
}

/// Specification for an image content block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageSpec {
    /// Path to the image file, relative to the `.sf` source file.
    pub path: Arc<str>,

    /// Accessibility alt text state. See [`ChartSpec::alt`] for semantics.
    pub alt: Option<AltText>,

    /// When `true`, the author set `decorative: true` on this element.
    pub decorative: bool,

    /// Source location.
    pub span: SourceSpan,
}

/// Specification for a table content block.
///
/// Tables are always content elements — they are never decorative — so there
/// is no `decorative` field. Alt text on a table is treated as a caption.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableSpec {
    /// Column header cells (each cell is a string for v1.0).
    pub headers: Vec<Arc<str>>,

    /// Table body rows. Each row is a sequence of cell strings.
    pub rows: Vec<Vec<Arc<str>>>,

    /// Accessibility alt text / caption.
    ///
    /// `None` means the author did not supply a caption. Tables are non-visual
    /// text content and are not subject to alt-text validation.
    pub alt: Option<AltText>,

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
            alt: Some(AltText::Provided(Arc::from("sales by region"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        assert_eq!(spec.chart_type.as_ref(), "bar");
        assert!(matches!(&spec.alt, Some(AltText::Provided(s)) if s.as_ref() == "sales by region"));
    }

    #[test]
    fn test_bc_1_01_specs_diagram_spec_fields() {
        let spec = DiagramSpec {
            source: Arc::from("graph TD; A-->B"),
            alt: Some(AltText::Provided(Arc::from("flow diagram"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        assert_eq!(spec.source.as_ref(), "graph TD; A-->B");
    }

    #[test]
    fn test_bc_1_01_specs_shape_spec_fields() {
        let spec = ShapeSpec {
            shape_type: Arc::from("rect"),
            alt: Some(AltText::Provided(Arc::from("a rectangle"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        assert_eq!(spec.shape_type.as_ref(), "rect");
    }

    #[test]
    fn test_bc_1_01_specs_image_spec_fields() {
        let spec = ImageSpec {
            path: Arc::from("images/logo.png"),
            alt: Some(AltText::Provided(Arc::from("company logo"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        assert_eq!(spec.path.as_ref(), "images/logo.png");
    }

    #[test]
    fn test_bc_1_01_specs_table_spec_fields() {
        let spec = TableSpec {
            headers: vec![Arc::from("Name"), Arc::from("Value")],
            rows: vec![vec![Arc::from("A"), Arc::from("1")]],
            alt: Some(AltText::Provided(Arc::from("data table"))),
            span: SourceSpan::default(),
        };
        assert_eq!(spec.headers.len(), 2);
        assert_eq!(spec.rows.len(), 1);
    }

    #[test]
    fn test_bc_1_01_specs_all_implement_hash() {
        use std::collections::HashSet;
        let chart = ChartSpec {
            chart_type: Arc::from("pie"),
            alt: Some(AltText::Provided(Arc::from("pie chart"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let mut set: HashSet<ChartSpec> = HashSet::new();
        set.insert(chart);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_01_specs_all_implement_clone() {
        let spec = ImageSpec {
            path: Arc::from("a.png"),
            alt: Some(AltText::Provided(Arc::from("an image"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let spec2 = spec.clone();
        assert_eq!(spec, spec2);
    }

    #[test]
    fn test_bc_1_01_specs_alt_text_none_means_missing() {
        let spec = ImageSpec {
            path: Arc::from("logo.png"),
            alt: None,
            decorative: false,
            span: SourceSpan::default(),
        };
        assert!(spec.alt.is_none());
        assert!(!spec.decorative);
    }

    #[test]
    fn test_bc_1_01_specs_decorative_alt_text() {
        let spec = ImageSpec {
            path: Arc::from("bg.png"),
            alt: None,
            decorative: true,
            span: SourceSpan::default(),
        };
        assert!(spec.alt.is_none());
        assert!(spec.decorative);
    }

    #[test]
    fn test_bc_1_01_specs_alt_text_decorative_variant() {
        let alt = AltText::Decorative;
        assert!(matches!(alt, AltText::Decorative));
    }

    #[test]
    fn test_bc_1_01_specs_alt_text_provided_variant() {
        let alt = AltText::Provided(Arc::from("a logo"));
        assert!(matches!(alt, AltText::Provided(_)));
    }
}
