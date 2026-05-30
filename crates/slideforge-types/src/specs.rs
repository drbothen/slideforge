//! Spec types for chart, diagram, shape, image, and table content blocks.
//!
//! These types carry the data needed by renderer and exporter plugins to
//! produce output. They are intentionally minimal in this story; fields will
//! be expanded in subsequent stories (chart series, table cell formatting, etc.).

use std::sync::Arc;

use crate::inline::InlineNode;
use crate::shape_types::{FillSpec, ShapeType};
use crate::span::SourceSpan;

/// A usvg-normalized SVG string that is guaranteed PPTX-safe.
///
/// This type is the output of the mandatory normalization pass (STORY-034,
/// BC-1.12.003) that runs after every successful diagram render. The normalized
/// form guarantees:
///
/// - No `<foreignObject>` elements.
/// - No `<script>` elements.
/// - No CSS `@keyframes` or class-based `<style>` blocks.
/// - Absolute pixel `width` and `height` on the root `<svg>` element.
/// - No `<use>` elements (all `href="#symbol"` references inlined).
///
/// ## Placement in slideforge-types
///
/// `NormalizedDiagramSvg` lives in `slideforge-types` (a leaf crate with no
/// workspace crate dependencies) so that both `slideforge-diagrams` (the
/// producer) and `slideforge-layout` (the consumer) can reference it without
/// creating a circular dependency.
///
/// `slideforge-diagrams` produces `NormalizedDiagramSvg` values via
/// `usvg_normalize`. `slideforge-layout` stores them in
/// `FrameContent::Diagram(NormalizedDiagramSvg)`.
///
/// ## IR compatibility
///
/// Uses `Arc<str>` (not `String`) so that cloning is cheap and the type
/// satisfies `Hash + Eq + Clone` for comemo compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedDiagramSvg(Arc<str>);

impl NormalizedDiagramSvg {
    /// Construct a `NormalizedDiagramSvg` from a pre-normalized SVG string.
    ///
    /// # Contract
    ///
    /// **The caller is responsible for ensuring the SVG has been normalized
    /// through `slideforge-diagrams::usvg_normalize` before calling this
    /// constructor.** The field is private to enforce that construction goes
    /// through a named entry point rather than tuple-struct literal syntax
    /// (`NormalizedDiagramSvg(raw_string)`), which makes the normalization
    /// requirement explicit at every callsite.
    ///
    /// Within the slideforge pipeline:
    /// - The **only legitimate production callsite** is
    ///   `slideforge_diagrams::normalize::usvg_normalize`, which performs the
    ///   full usvg round-trip before wrapping the result.
    /// - Test code may call this constructor with synthetic SVG strings in
    ///   unit tests that verify downstream consumers (layout, exporters), where
    ///   the SVG content is controlled and normalization guarantees are
    ///   asserted separately.
    ///
    /// **DO NOT CALL FROM EXPORTERS** — exporters receive `NormalizedDiagramSvg`
    /// values produced by the diagram rendering pipeline and must not
    /// construct them from raw strings.
    #[must_use]
    pub fn from_normalized_string(svg: Arc<str>) -> Self {
        Self(svg)
    }

    /// Return a reference to the inner SVG string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return `true` if the SVG string is non-empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Documentation of the placeholder contract for exporter consumers.
    ///
    /// Exporters that receive a [`NormalizedDiagramSvg`] value produced by
    /// [`NormalizedDiagramSvg::empty_placeholder`] MUST call
    /// [`NormalizedDiagramSvg::is_placeholder`] before embedding the SVG into
    /// an output format. A placeholder is an empty string and is NOT a valid
    /// PPTX-safe SVG. If `is_placeholder()` returns `true`, the exporter must
    /// substitute an error-slide indicator instead.
    ///
    /// The eval layer is responsible for replacing placeholder values with
    /// real [`NormalizedDiagramSvg`] values before exporters run. If a
    /// placeholder reaches an exporter, it is a pipeline bug, not a user error.
    pub const PLACEHOLDER_DOC: &'static str = "NormalizedDiagramSvg::empty_placeholder() returns an empty string. \
         Exporters MUST check is_placeholder() before embedding. \
         A placeholder reaching an exporter is a pipeline bug.";

    /// Return an empty-string placeholder `NormalizedDiagramSvg` for use in
    /// region-map templates and layout fixtures where the actual diagram SVG is
    /// not yet available.
    ///
    /// # Safety Contract
    ///
    /// **Exporters MUST call [`is_placeholder`][Self::is_placeholder] before
    /// embedding the returned value.** An empty-string placeholder is NOT a
    /// valid PPTX-safe SVG. If `is_placeholder()` returns `true`, the exporter
    /// must render an error-slide indicator instead of attempting to embed the
    /// empty string.
    ///
    /// The eval layer replaces placeholder values with real
    /// `NormalizedDiagramSvg` before exporters run. A placeholder that reaches
    /// an exporter without being replaced is a pipeline bug.
    ///
    /// See [`PLACEHOLDER_DOC`][Self::PLACEHOLDER_DOC] for the full contract.
    #[must_use]
    pub fn empty_placeholder() -> Self {
        Self(Arc::from(""))
    }

    /// Return `true` if this value is a placeholder (empty string) rather than
    /// a real normalized SVG.
    ///
    /// Exporters MUST check this before embedding a `NormalizedDiagramSvg`.
    /// See [`empty_placeholder`][Self::empty_placeholder] for the contract.
    #[must_use]
    pub fn is_placeholder(&self) -> bool {
        self.0.is_empty()
    }
}

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
    /// the validator emits `W-A11-002` (alt takes precedence over decorative, per
    /// BC-3.04.001 v1.5.2 Invariant 11 / F-P18-HIGH-001).
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

/// A position measurement in a user-facing unit (inches or em).
///
/// Stored as an integer multiple of `1/1000` of the named unit to avoid `f64`:
///
/// - `Inches(milliinches)` — e.g., `0.5in` is stored as `Inches(500)`.
///   EMU conversion: `(milliinches * 914_400) / 1_000`.
/// - `Em(milliem)` — e.g., `1em` is stored as `Em(1000)`.
///   EMU conversion: `(milliem * brand_em_in_emu) / 1_000`.
///
/// Implements `Debug + Clone + PartialEq + Eq + Hash` for comemo compatibility
/// (DI-010, CLAUDE.md hash+eq+clone rule).
///
/// See BC-3.04.001 postcondition 1 for the authoritative unit table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeUnit {
    /// Measurement in inches, stored as thousandths of an inch (`inches × 1000`).
    ///
    /// Example: `0.5in` → `Inches(500)`.
    /// EMU conversion: `(milliinches * 914_400) / 1_000`.
    Inches(i64),
    /// Measurement in em units, stored as thousandths of an em (`em × 1000`).
    ///
    /// Example: `2em` → `Em(2000)`.
    /// EMU conversion: `(milliem * brand_em_in_emu) / 1_000`.
    Em(i64),
}

/// The position and size of a shape in user-declared units.
///
/// Carried by [`ShapeSpec`] through the semantic IR. The layout pass converts
/// each field to integer EMU via `slideforge_layout::shapes::unit_to_emu`.
///
/// Implements `Debug + Clone + PartialEq + Eq + Hash` for comemo compatibility
/// (DI-010, CLAUDE.md hash+eq+clone rule).
///
/// See BC-3.04.001 postcondition 1 for conversion constants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapePosition {
    /// Horizontal position from the slide's left edge.
    pub x: ShapeUnit,
    /// Vertical position from the slide's top edge.
    pub y: ShapeUnit,
    /// Width of the shape.
    pub width: ShapeUnit,
    /// Height of the shape.
    pub height: ShapeUnit,
}

/// Specification for a shape content block.
///
/// Carries the full shape declaration from the `shape:` DSL block, including
/// type, position, fill, optional text content, and accessibility alt text.
///
/// ## Schema (BC-3.04.001 v1.5.2 / interface-definitions.md §9)
///
/// - `shape_type` is now a resolved [`ShapeType`] enum variant (not a raw
///   `Arc<str>`). The parser validates the keyword and rejects unknown values
///   with `E-PAR-012` before constructing a `ShapeSpec`. This eliminates the
///   class of "unknown shape type" bugs that previously could only be caught at
///   layout time.
/// - `fill` carries the fill specification from the `fill:` DSL field. Defaults
///   to [`crate::shape_types::FillSpec::None`] (transparent) when not declared.
/// - `text` carries optional inline text content from the `text:` DSL field.
///
/// Implements `Debug + Clone + PartialEq + Eq + Hash` for comemo compatibility
/// (DI-010, CLAUDE.md hash+eq+clone rule).
///
/// See BC-3.04.001 for the authoritative contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeSpec {
    /// The resolved geometric shape type.
    ///
    /// Corresponds to the closed v1.0 vocabulary: `rect`, `ellipse`, `arrow`,
    /// `line`, `star`, `roundRect`. Unknown keywords are rejected by the parser
    /// with `E-PAR-012` before a `ShapeSpec` is constructed — there is no
    /// `Custom` fallback (BC-3.04.001 invariant 4 / CLAUDE.md "no silent
    /// fallback" rule).
    pub shape_type: ShapeType,

    /// The shape's declared position and size in user units.
    ///
    /// The layout pass converts each [`ShapeUnit`] field to integer EMU using
    /// `slideforge_layout::shapes::unit_to_emu` (BC-3.04.001 postcondition 2).
    pub position: ShapePosition,

    /// The fill specification for this shape.
    ///
    /// Derived from the `fill:` DSL field. Defaults to
    /// [`crate::shape_types::FillSpec::None`] (transparent background) when
    /// the author does not declare a fill.
    pub fill: FillSpec,

    /// Optional inline text content rendered inside the shape.
    ///
    /// `None` means the shape has no text label. Populated from the `text:`
    /// DSL field when present.
    pub text: Option<Vec<InlineNode>>,

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
        use crate::shape_types::{FillSpec, ShapeType};

        let spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),       // 0.5in
                y: ShapeUnit::Inches(1000),      // 1.0in
                width: ShapeUnit::Inches(2000),  // 2.0in
                height: ShapeUnit::Inches(1000), // 1.0in
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("a rectangle"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        assert_eq!(spec.shape_type, ShapeType::Rect);
        // Verify position fields carry the declared user units.
        assert!(matches!(spec.position.x, ShapeUnit::Inches(500)));
        assert!(matches!(spec.position.y, ShapeUnit::Inches(1000)));
        assert!(matches!(spec.position.width, ShapeUnit::Inches(2000)));
        assert!(matches!(spec.position.height, ShapeUnit::Inches(1000)));
        assert!(matches!(spec.fill, FillSpec::None));
        assert!(spec.text.is_none());
    }

    /// BC-3.04.001 — `ShapePosition` and `ShapeUnit` implement `Hash + Eq + Clone`
    /// (comemo compatibility / DI-010).
    #[test]
    fn test_bc_3_04_001_shape_position_and_unit_implement_hash_eq_clone() {
        use std::collections::HashSet;

        let pos = ShapePosition {
            x: ShapeUnit::Inches(500),
            y: ShapeUnit::Em(1000),
            width: ShapeUnit::Inches(2000),
            height: ShapeUnit::Em(500),
        };
        let pos2 = pos.clone();
        assert_eq!(pos, pos2);

        let mut set = HashSet::new();
        set.insert(ShapeUnit::Inches(500));
        set.insert(ShapeUnit::Inches(500)); // duplicate
        assert_eq!(set.len(), 1, "ShapeUnit must deduplicate in HashSet");
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

    // -----------------------------------------------------------------------
    // NormalizedDiagramSvg placeholder helpers (F-MED-001)
    // -----------------------------------------------------------------------

    #[test]
    fn test_normalized_diagram_svg_is_placeholder_true_for_empty_placeholder() {
        let p = NormalizedDiagramSvg::empty_placeholder();
        assert!(
            p.is_placeholder(),
            "empty_placeholder() must return a value where is_placeholder() == true"
        );
    }

    #[test]
    fn test_normalized_diagram_svg_is_placeholder_false_for_real_svg() {
        let svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"/>",
        ));
        assert!(
            !svg.is_placeholder(),
            "a real SVG string must not be reported as a placeholder"
        );
    }

    #[test]
    fn test_normalized_diagram_svg_placeholder_doc_is_non_empty() {
        // Ensures the contract constant is present and non-trivial.
        assert!(
            !NormalizedDiagramSvg::PLACEHOLDER_DOC.is_empty(),
            "PLACEHOLDER_DOC must be a non-empty string"
        );
        assert!(
            NormalizedDiagramSvg::PLACEHOLDER_DOC.contains("is_placeholder"),
            "PLACEHOLDER_DOC must reference is_placeholder()"
        );
    }
}
