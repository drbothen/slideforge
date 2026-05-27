//! Alt text enforcement validator (STORY-015).
//!
//! Checks all visual elements in a [`Deck`](slideforge_types::Deck) for
//! required alt text. Visual elements without non-empty alt text AND without
//! `decorative: true` produce an `E-A11-001` error diagnostic.
//!
//! ## Error codes
//!
//! | Code | Severity | Meaning |
//! |------|----------|---------|
//! | `E-A11-001` | Error | Visual element is missing alt text and is not marked decorative |
//! | `W-A11-001` | Warning | Visual element has both alt text AND `decorative: true` (alt is ignored) |

use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
use slideforge_types::{
    Deck, SourceSpan,
    specs::AltText,
};

use crate::utils::is_blank;

/// Error code for a visual element missing alt text.
///
/// Emitted for images, charts, diagrams, shapes, and tables that have neither
/// valid alt text nor `decorative: true`.
///
/// Used by the `validate()` implementation (STORY-015 implementer phase) and
/// exercised directly by the test suite.
pub(crate) const E_A11_001: &str = "E-A11-001";

/// Warning code emitted when a visual element has both alt text AND `decorative: true`.
///
/// `decorative: true` takes precedence — the provided alt text is silently
/// ignored. Authors should remove the `alt` field or remove `decorative: true`
/// to resolve the ambiguity.
///
/// Used by the `validate()` implementation (STORY-015 implementer phase) and
/// exercised directly by the test suite.
pub(crate) const W_A11_001: &str = "W-A11-001";

/// Validates that all visual elements have alt text or are marked decorative.
///
/// Implements the WCAG 1.1.1 (Non-text Content) requirement. Every image,
/// chart, diagram, and shape must either:
/// - Have a non-empty, non-whitespace `alt` text value, OR
/// - Be explicitly marked `decorative: true`
///
/// Non-visual blocks (Text, Bullets, Math, Table) are already readable content
/// and do not require alt text.
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_validator`].
pub struct AltTextValidator;

impl Validator for AltTextValidator {
    fn id(&self) -> &'static str {
        "alt-text"
    }

    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        use slideforge_types::ContentBlock;

        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for slide in &deck.slides {
            for block in &slide.blocks {
                match &block.content {
                    ContentBlock::Image(spec) => {
                        check_visual_element(
                            spec.alt.as_ref(),
                            spec.decorative,
                            "image",
                            spec.path.as_ref(),
                            &spec.span,
                            &mut diagnostics,
                        );
                    }
                    ContentBlock::Chart(spec) => {
                        check_visual_element(
                            spec.alt.as_ref(),
                            spec.decorative,
                            "chart",
                            spec.chart_type.as_ref(),
                            &spec.span,
                            &mut diagnostics,
                        );
                    }
                    ContentBlock::Diagram(spec) => {
                        // Truncate diagram source to first 30 *chars* (not bytes) for the
                        // identifier. Using byte indexing (&source[..30]) would panic if byte 30
                        // falls in the middle of a multi-byte UTF-8 character (e.g. CJK labels
                        // in Mermaid diagrams). chars().take(30) is always char-boundary-safe.
                        let identifier: std::borrow::Cow<str> =
                            if spec.source.chars().count() > 30 {
                                let truncated: String =
                                    spec.source.chars().take(30).collect();
                                std::borrow::Cow::Owned(format!("{truncated}…"))
                            } else {
                                std::borrow::Cow::Borrowed(spec.source.as_ref())
                            };
                        check_visual_element(
                            spec.alt.as_ref(),
                            spec.decorative,
                            "diagram",
                            identifier.as_ref(),
                            &spec.span,
                            &mut diagnostics,
                        );
                    }
                    ContentBlock::Shape(spec) => {
                        check_visual_element(
                            spec.alt.as_ref(),
                            spec.decorative,
                            "shape",
                            spec.shape_type.as_ref(),
                            &spec.span,
                            &mut diagnostics,
                        );
                    }
                    // Non-visual blocks: Text, Bullets, Math, Table — no alt text required.
                    // Tables are text content that is already readable by screen readers
                    // (story spec, STORY-015 line 309). Alt text on tables is not validated.
                    ContentBlock::Text(_)
                    | ContentBlock::Bullets(_)
                    | ContentBlock::Math(_)
                    | ContentBlock::Table(_) => {}
                }
            }
        }

        diagnostics
    }
}

/// Check a visual element with `decorative` support (Image, Chart, Diagram, Shape).
///
/// Logic (per AC-009, AC-006, AC-001 through AC-005):
///
/// 1. If `decorative: true` AND `alt` is `Some(AltText::Provided(s))` where `s` is
///    non-blank → emit W-A11-001 (non-empty alt is ignored).
///    Empty/whitespace alt with `decorative: true` does NOT emit W-A11-001 because
///    blank alt is not meaningful content worth warning about.
/// 2. If `decorative: true` (regardless) → skip error check (decorative exemption).
/// 3. If `alt` is `None` or blank `Provided` → emit E-A11-001.
/// 4. If `alt` is valid `Provided` or `Decorative` enum variant → valid, no diagnostic.
///
/// ## Design note: dual decorative representation
///
/// The IR carries BOTH `decorative: bool` (from `decorative: true` keyword in DSL) AND
/// `alt: Option<AltText>` (where `AltText::Decorative` can also express decorative intent).
/// This dual representation is necessary to detect the AC-009 case: when the user writes
/// BOTH `alt "..."` AND `decorative: true`, both fields are set and we can warn.
///
/// The contradictory state `alt: Some(AltText::Decorative)` with `decorative: false`
/// is an internal IR state that should not arise from well-formed DSL input (the parser
/// sets `decorative: false` only when the keyword is absent, and uses `AltText::Decorative`
/// only as the enum-level representation of the same concept). If it does occur,
/// `AltText::Decorative` wins — the element is treated as decorative with no error.
fn check_visual_element(
    alt: Option<&AltText>,
    decorative: bool,
    element_type: &str,
    identifier: &str,
    span: &SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // AC-009: both non-blank alt text AND decorative: true → warn that alt is ignored.
    // Blank alt with decorative: true is silently accepted (blank alt is not meaningful).
    if decorative {
        if let Some(AltText::Provided(s)) = alt
            && !is_blank(s)
        {
            diagnostics.push(make_warning(element_type, identifier, span));
        }
        // Decorative exemption: no E-A11-001 needed.
        return;
    }

    // Non-decorative: check that alt text is present and non-blank.
    // AltText::Decorative (enum variant) wins — element is treated as valid even if
    // `decorative: bool` is false. See design note on dual decorative representation above.
    let is_missing = match alt {
        None => true,
        Some(AltText::Provided(s)) => is_blank(s),
        Some(AltText::Decorative) => false,
    };

    if is_missing {
        diagnostics.push(make_error(element_type, identifier, span));
    }
}

/// Construct an `E-A11-001` error diagnostic for a missing alt text.
///
/// The `identifier` identifies the specific element:
/// - Image: file path (e.g., `"photo.png"`)
/// - Chart: chart type (e.g., `"bar"`)
/// - Diagram: first 30 chars of source (char-boundary-safe truncation)
/// - Shape: shape type (e.g., `"rect"`)
fn make_error(element_type: &str, identifier: &str, span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: std::sync::Arc::from(E_A11_001),
        message: std::sync::Arc::from(format!(
            "Missing alt text on {element_type} '{identifier}' at {span}. \
             Add alt \"...\" or mark decorative: true"
        )),
        span: span.clone(),
        hint: Some(std::sync::Arc::from(
            "All visual elements require alt text or decorative: true (DI-001)",
        )),
    }
}

/// Construct a `W-A11-001` warning diagnostic for conflicting non-blank alt + decorative.
fn make_warning(element_type: &str, identifier: &str, span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Warning,
        code: std::sync::Arc::from(W_A11_001),
        message: std::sync::Arc::from(format!(
            "Alt text ignored for decorative {element_type} '{identifier}' at {span}"
        )),
        span: span.clone(),
        hint: Some(std::sync::Arc::from(
            "Remove alt text or remove decorative: true",
        )),
    }
}

// ─── Test helpers ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DiagnosticSeverity, ValidatorOptions, Validator};
    use slideforge_types::{
        Block, ContentBlock, Deck, DeckMetadata, OrderedMap, Slide, SourceSpan,
        specs::{AltText, ChartSpec, DiagramSpec, ImageSpec, ShapeSpec, TableSpec},
    };

    use super::{AltTextValidator, E_A11_001, W_A11_001};

    // ── Deck/slide/block construction helpers ──────────────────────────────────

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
        }
    }

    fn make_deck(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
        }
    }

    fn make_slide(blocks: Vec<Block>) -> Slide {
        Slide {
            slide_type: Arc::from("content"),
            fields: OrderedMap::new(),
            blocks,
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
        }
    }

    fn make_block(content: ContentBlock) -> Block {
        Block {
            content,
            label: None,
            span: SourceSpan::default(),
        }
    }

    fn make_image_block(alt: Option<AltText>, decorative: bool) -> Block {
        make_block(ContentBlock::Image(ImageSpec {
            path: Arc::from("photo.png"),
            alt,
            decorative,
            span: SourceSpan::default(),
        }))
    }

    fn make_chart_block(alt: Option<AltText>, decorative: bool) -> Block {
        make_block(ContentBlock::Chart(ChartSpec {
            chart_type: Arc::from("bar"),
            alt,
            decorative,
            span: SourceSpan::default(),
        }))
    }

    fn make_diagram_block(alt: Option<AltText>, decorative: bool) -> Block {
        make_block(ContentBlock::Diagram(DiagramSpec {
            source: Arc::from("graph TD; A-->B"),
            alt,
            decorative,
            span: SourceSpan::default(),
        }))
    }

    fn make_diagram_block_with_source(
        source: &str,
        alt: Option<AltText>,
        decorative: bool,
    ) -> Block {
        make_block(ContentBlock::Diagram(DiagramSpec {
            source: Arc::from(source),
            alt,
            decorative,
            span: SourceSpan::default(),
        }))
    }

    fn make_shape_block(alt: Option<AltText>, decorative: bool) -> Block {
        make_block(ContentBlock::Shape(ShapeSpec {
            shape_type: Arc::from("rect"),
            alt,
            decorative,
            span: SourceSpan::default(),
        }))
    }

    fn make_table_block(alt: Option<AltText>) -> Block {
        make_block(ContentBlock::Table(TableSpec {
            headers: vec![Arc::from("Col1")],
            rows: vec![vec![Arc::from("val")]],
            alt,
            span: SourceSpan::default(),
        }))
    }

    fn default_opts() -> ValidatorOptions {
        ValidatorOptions::default()
    }

    // ── Validator ID ───────────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_validator_id() {
        let v = AltTextValidator;
        assert_eq!(v.id(), "alt-text");
    }

    // ── Missing alt — errors ───────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_missing_alt_single_image() {
        // 1 image with alt: None, decorative: false → exactly 1 E-A11-001
        let slide = make_slide(vec![make_image_block(None, false)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1, "expected exactly 1 diagnostic, got {diags:?}");
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_bc_5_03_015_missing_alt_multiple_images() {
        // 3 images all missing alt → 3 E-A11-001 diagnostics
        let slide = make_slide(vec![
            make_image_block(None, false),
            make_image_block(None, false),
            make_image_block(None, false),
        ]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 3, "expected 3 diagnostics, got {diags:?}");
        for d in &diags {
            assert_eq!(d.code.as_ref(), E_A11_001);
            assert_eq!(d.severity, DiagnosticSeverity::Error);
        }
    }

    #[test]
    fn test_bc_5_03_015_empty_string_alt_is_missing() {
        // alt: Some(AltText::Provided("")) is treated as missing (blank check)
        let slide = make_slide(vec![make_image_block(
            Some(AltText::Provided(Arc::from(""))),
            false,
        )]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1, "empty string alt should produce E-A11-001; got {diags:?}");
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    #[test]
    fn test_bc_5_03_015_whitespace_only_alt_is_missing() {
        // alt: Some(AltText::Provided("  ")) is treated as missing
        let slide = make_slide(vec![make_image_block(
            Some(AltText::Provided(Arc::from("  "))),
            false,
        )]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "whitespace-only alt should produce E-A11-001; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    // ── Present alt — no error ─────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_present_alt_no_error() {
        // image with valid alt → 0 diagnostics
        let slide = make_slide(vec![make_image_block(
            Some(AltText::Provided(Arc::from("Team photo"))),
            false,
        )]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(diags.is_empty(), "valid alt should produce no diagnostics; got {diags:?}");
    }

    // ── Decorative elements ────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_decorative_no_error() {
        // alt: None, decorative: true → 0 diagnostics
        let slide = make_slide(vec![make_image_block(None, true)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "decorative element should produce no error; got {diags:?}"
        );
    }

    #[test]
    fn test_bc_5_03_015_decorative_all_elements() {
        // 3 decorative images → 0 diagnostics
        let slide = make_slide(vec![
            make_image_block(None, true),
            make_image_block(None, true),
            make_image_block(None, true),
        ]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "all decorative elements should produce no diagnostics; got {diags:?}"
        );
    }

    #[test]
    fn test_bc_5_03_015_alt_and_decorative_together() {
        // alt: Some(Provided("text")), decorative: true → 1 W-A11-001 warning
        // Decorative wins, but the ignored non-blank alt text warrants a warning (AC-009).
        let slide = make_slide(vec![make_image_block(
            Some(AltText::Provided(Arc::from("this alt will be ignored"))),
            true,
        )]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "alt+decorative together should produce 1 W-A11-001; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), W_A11_001);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Warning);
    }

    // ── Other visual element types ─────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_chart_missing_alt() {
        // chart with alt: None → 1 E-A11-001
        let slide = make_slide(vec![make_chart_block(None, false)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1, "chart missing alt should produce 1 diagnostic; got {diags:?}");
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    #[test]
    fn test_bc_5_03_015_diagram_missing_alt() {
        // diagram with alt: None → 1 E-A11-001
        let slide = make_slide(vec![make_diagram_block(None, false)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "diagram missing alt should produce 1 diagnostic; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    #[test]
    fn test_bc_5_03_015_table_no_alt_required() {
        // Tables are text content — no alt text validation is performed (story spec line 309).
        // A table with no alt text must produce zero diagnostics.
        let slide = make_slide(vec![make_table_block(None)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "tables are non-visual text content and must produce no diagnostics; got {diags:?}"
        );
    }

    // ── Unicode diagram source truncation does not panic (FINDING-001) ─────────

    #[test]
    fn test_diagram_unicode_source_no_panic() {
        // Diagram source with multi-byte UTF-8 characters (CJK) exceeding 30 bytes.
        // Byte-indexing at position 30 would panic mid-character; chars().take(30) is safe.
        let source = "graph TD; A[\"日本語のラベル\"]-->B[\"中文標籤のテスト\"]";
        let deck = make_deck(vec![make_slide(vec![make_diagram_block_with_source(
            source,
            None,
            false,
        )])]);
        let diags = AltTextValidator.validate(&deck, &ValidatorOptions::default());
        // Should produce E-A11-001 (missing alt) without panicking
        assert_eq!(diags.len(), 1, "expected 1 E-A11-001 diagnostic; got {diags:?}");
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    // ── Error accumulation ─────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_error_accumulation() {
        // 2 images + 1 diagram all missing alt → 3 E-A11-001 (all accumulated)
        let slide = make_slide(vec![
            make_image_block(None, false),
            make_image_block(None, false),
            make_diagram_block(None, false),
        ]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            3,
            "should accumulate all 3 missing-alt errors; got {diags:?}"
        );
        for d in &diags {
            assert_eq!(d.code.as_ref(), E_A11_001);
        }
    }

    #[test]
    fn test_bc_5_03_015_error_accumulation_across_slides() {
        // 2 slides, each with 1 image missing alt → 2 E-A11-001
        let slide1 = make_slide(vec![make_image_block(None, false)]);
        let slide2 = make_slide(vec![make_image_block(None, false)]);
        let deck = make_deck(vec![slide1, slide2]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            2,
            "should accumulate errors across slides; got {diags:?}"
        );
        for d in &diags {
            assert_eq!(d.code.as_ref(), E_A11_001);
        }
    }

    // ── Non-visual elements need no alt ───────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_text_blocks_no_alt_needed() {
        // deck with only Text blocks → 0 diagnostics
        use slideforge_types::{InlineNode, block::TextBlock};
        let text_block = make_block(ContentBlock::Text(TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from("just text"))],
            span: SourceSpan::default(),
        }));
        let bullets_block = make_block(ContentBlock::Bullets(vec![]));
        let slide = make_slide(vec![text_block, bullets_block]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "text/bullets blocks need no alt text; got {diags:?}"
        );
    }

    #[test]
    fn test_bc_5_03_015_math_blocks_no_alt_needed() {
        // Math blocks do not need alt text (they render as MathML/OMML inline)
        use slideforge_types::MathNode;
        let math_block = make_block(ContentBlock::Math(MathNode::display(
            Arc::from("x^2 + y^2 = r^2"),
            SourceSpan::default(),
        )));
        let slide = make_slide(vec![math_block]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "math blocks need no alt text; got {diags:?}"
        );
    }

    // ── Empty deck ─────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_empty_deck_no_diagnostics() {
        let deck = make_deck(vec![]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(diags.is_empty(), "empty deck should produce no diagnostics");
    }

    #[test]
    fn test_bc_5_03_015_empty_slide_no_diagnostics() {
        // Slide with no blocks → no diagnostics
        let slide = make_slide(vec![]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(diags.is_empty(), "empty slide should produce no diagnostics");
    }

    // ── Mixed valid/invalid ────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_mixed_valid_and_invalid() {
        // 1 image with valid alt + 1 image missing alt → exactly 1 error
        let slide = make_slide(vec![
            make_image_block(Some(AltText::Provided(Arc::from("a logo"))), false),
            make_image_block(None, false),
        ]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "only the missing-alt image should produce an error; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    // ── Diagnostic hint ───────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_error_has_hint() {
        // E-A11-001 diagnostics must include a correction hint
        let slide = make_slide(vec![make_image_block(None, false)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].hint.is_some(),
            "E-A11-001 diagnostic must include a hint; got None"
        );
    }

    // ── Error message contains identifier (ADV-P01-HIGH-001) ──────────────────

    #[test]
    fn test_error_message_contains_identifier_image() {
        // E-A11-001 for an image must include the file path as identifier
        let deck = make_deck(vec![make_slide(vec![make_image_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("'photo.png'"),
            "error message must contain the image path; got: {}",
            diags[0].message
        );
    }

    #[test]
    fn test_error_message_contains_identifier_chart() {
        // E-A11-001 for a chart must include the chart type as identifier
        let deck = make_deck(vec![make_slide(vec![make_chart_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("'bar'"),
            "error message must contain the chart type; got: {}",
            diags[0].message
        );
    }

    #[test]
    fn test_error_message_contains_identifier_diagram() {
        // E-A11-001 for a diagram must include truncated source as identifier
        let deck = make_deck(vec![make_slide(vec![make_diagram_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("'graph TD; A-->B'"),
            "error message must contain the diagram source; got: {}",
            diags[0].message
        );
    }

    #[test]
    fn test_warning_message_contains_identifier() {
        // W-A11-001 for an image must include the file path as identifier
        let deck = make_deck(vec![make_slide(vec![make_image_block(
            Some(AltText::Provided(Arc::from("ignored alt"))),
            true,
        )])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code.as_ref(), W_A11_001);
        assert!(
            diags[0].message.contains("'photo.png'"),
            "warning message must contain the image path; got: {}",
            diags[0].message
        );
    }

    // ── Shape missing alt (ADV-P01-MED-003) ───────────────────────────────────

    #[test]
    fn test_shape_missing_alt() {
        // shape with alt: None, decorative: false → 1 E-A11-001
        let deck = make_deck(vec![make_slide(vec![make_shape_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1, "shape missing alt should produce 1 diagnostic; got {diags:?}");
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    // ── Blank alt with decorative — no warning (ADV-P01-MED-002) ─────────────

    #[test]
    fn test_blank_alt_with_decorative_no_warning() {
        // decorative: true + alt: Some(Provided("")) → no warning
        // Blank alt is not meaningful content, so no W-A11-001 is emitted.
        let deck = make_deck(vec![make_slide(vec![make_image_block(
            Some(AltText::Provided(Arc::from(""))),
            true,
        )])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "blank alt with decorative: true should produce no diagnostic; got {diags:?}"
        );
    }

    #[test]
    fn test_whitespace_alt_with_decorative_no_warning() {
        // decorative: true + alt: Some(Provided("   ")) → no warning (whitespace is blank)
        let deck = make_deck(vec![make_slide(vec![make_image_block(
            Some(AltText::Provided(Arc::from("   "))),
            true,
        )])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "whitespace-only alt with decorative: true should produce no diagnostic; got {diags:?}"
        );
    }

    // ── AltText::Decorative enum with decorative: false (ADV-P01-LOW-002) ─────

    #[test]
    fn test_alt_text_decorative_enum_with_decorative_false() {
        // AltText::Decorative in the enum but decorative: false
        // The enum wins — element is treated as decorative, no error.
        // This is an internal IR state that should not arise from well-formed DSL input.
        let deck = make_deck(vec![make_slide(vec![make_image_block(
            Some(AltText::Decorative),
            false,
        )])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "AltText::Decorative enum variant should be treated as valid even with decorative: false; got {diags:?}"
        );
    }

    // ── Snapshot test (ADV-P01-MED-004) ───────────────────────────────────────

    #[test]
    fn test_snapshot_alt_error_message() {
        let deck = make_deck(vec![make_slide(vec![make_image_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        insta::assert_snapshot!(diags[0].message.as_ref());
    }
}
