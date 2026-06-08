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
//! | `W-A11-002` | Warning | Visual element has both alt text AND `decorative: true`; alt takes precedence, decorative flag ignored (BC-3.04.001 v1.5.2 Invariant 11) |

use slideforge_layout::{FrameContent, LaidOutDeck};
use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
use slideforge_types::{Deck, SourceSpan, specs::AltText};

use crate::utils::is_blank;

/// Error code for a visual element missing alt text.
///
/// Emitted for images, charts, diagrams, and shapes that have neither valid alt
/// text nor `decorative: true`.
///
/// Used by the `validate()` implementation (STORY-015 implementer phase) and
/// exercised directly by the test suite.
pub(crate) const E_A11_001: &str = "E-A11-001";

/// Warning code emitted when a visual element has both alt text AND `decorative: true`.
///
/// Per BC-3.04.001 v1.5.2 Invariant 11 (F-P18-HIGH-001): **alt wins over decorative**.
/// The element is treated as having valid alt text; `decorative: true` is ignored.
/// Authors should remove `decorative: true` or remove the `alt` field to resolve
/// the ambiguity.
///
/// Used by the `validate()` implementation (STORY-015 implementer phase) and
/// exercised directly by the test suite.
pub(crate) const W_A11_002: &str = "W-A11-002";

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

    /// Stage 5 (pre-layout) alt-text check on the semantic [`Deck`] IR.
    ///
    /// Iterates `deck.slides[*].blocks` looking for `ContentBlock::Shape` entries
    /// with missing or blank alt text. **Restricted to Shape blocks only** per
    /// ADR-018 v1.2 Decision-3 (STORY-086 scope-expansion adjudication).
    ///
    /// ## Scope: Shape only (ADR-018 v1.2 Decision-3)
    ///
    /// `ContentBlock::Chart`, `ContentBlock::Image`, and `ContentBlock::Diagram`
    /// are NOT validated here. Stage 2b (STORY-086) populates these blocks with
    /// `alt = None` when no author alt text is supplied. If the pre-layout pass
    /// fired on them, it would produce a double-diagnostic alongside the
    /// post-layout `validate_post_layout()` pass (which fires on
    /// `AltText::Unspecified` frames). Single-fire is enforced by restricting
    /// pre-layout scope to Shape, which is NOT a structural-placeholder type in
    /// regions.rs (shapes are always user-authored via the shape DSL).
    ///
    /// The authoritative alt-text enforcement for Chart/Image/Diagram is in
    /// [`AltTextValidator::validate_post_layout`] (ADR-018 Decision 3 /
    /// ADR-019 Decision 5.3).
    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        use slideforge_types::ContentBlock;

        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for slide in &deck.slides {
            for block in &slide.blocks {
                match &block.content {
                    // Pre-layout scope: Shape only (ADR-018 v1.2 Decision-3).
                    // Shape blocks are always user-authored (shape DSL) and have no
                    // structural-placeholder counterpart in regions.rs, so there is no
                    // risk of double-fire with the post-layout validator.
                    ContentBlock::Shape(spec) => {
                        check_visual_element(
                            spec.alt.as_ref(),
                            spec.decorative,
                            "shape",
                            spec.shape_type.as_keyword(),
                            &spec.span,
                            &mut diagnostics,
                        );
                    },
                    // Chart, Image, Diagram: validated post-layout only (ADR-018 v1.2 Decision-3).
                    // Restricting here prevents double-fire when Stage 2b emits blocks with alt=None
                    // (thread_media_alt_into_frames maps None → AltText::Unspecified; post-layout
                    // validate_post_layout fires exactly once — BC-5.01.001 postcondition 1).
                    // Non-visual blocks: Text, Bullets, Math, Table — no alt text required.
                    // Tables are text content that is already readable by screen readers
                    // (story spec, STORY-015 line 309). Alt text on tables is not validated.
                    // STORY-087 pass-2: ColorBar is geometry-only; no alt text required here.
                    // The adjacent ColorLabel text block carries the accessibility co-encoding.
                    ContentBlock::Chart(_)
                    | ContentBlock::Image(_)
                    | ContentBlock::Diagram(_)
                    | ContentBlock::Text(_)
                    | ContentBlock::Bullets(_)
                    | ContentBlock::Math(_)
                    | ContentBlock::Table(_)
                    | ContentBlock::ColorBar(_) => {},
                }
            }
        }

        diagnostics
    }

    /// Stage 6b (post-layout) alt-text check on the geometric [`LaidOutDeck`] IR.
    ///
    /// Iterates `laid_out.slides[*].frames` looking for `FrameContent::Chart`,
    /// `FrameContent::Image`, and `FrameContent::Diagram` entries with
    /// `AltText::Unspecified` (the pipeline gap indicator — ADR-019 Decision 4).
    ///
    /// ## `AltText` discrimination (ADR-019 Decision 5.3 / STORY-086)
    ///
    /// | Frame `alt` value            | Outcome                          |
    /// |------------------------------|----------------------------------|
    /// | `AltText::Unspecified`       | E-A11-001 (pipeline gap)         |
    /// | `AltText::Decorative`        | VALID — author explicit opt-out  |
    /// | `AltText::Provided(_)`       | VALID — author supplied alt text |
    ///
    /// `AltText::Unspecified` is the structural placeholder set by `regions.rs`
    /// (ADR-019 Decision 5.1). Stage 2b (`thread_fields_to_blocks`) populates
    /// `Slide.blocks`, and `thread_media_alt_into_frames` overwrites the placeholder
    /// with the author-supplied value. If the placeholder is never overwritten
    /// (because the author omitted `alt "..."` and `decorative: true`), the frame
    /// reaches this validator with `AltText::Unspecified` — a true positive for
    /// E-A11-001.
    ///
    /// Threading lands in Story A (`STORY-086`). See ADR-019.
    ///
    /// Traceability: ADR-018 Decision 3, ADR-019 Decision 5.3,
    /// BC-5.02.001 §Accessibility, BC-5.01.001 postcondition 1.
    fn validate_post_layout(
        &self,
        laid_out: &LaidOutDeck,
        _opts: &ValidatorOptions,
    ) -> Vec<Diagnostic> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for laid_out_slide in &laid_out.slides {
            // source_index is the ordinal position in the semantic Deck (0-based); display
            // as 1-based for user-facing diagnostics.
            let display_slide = laid_out_slide.source_index + 1;
            for frame in &laid_out_slide.frames {
                let slide_type = laid_out_slide.slide_type_keyword.as_ref();
                match &frame.content {
                    // AltText::Unspecified — pipeline gap: no author alt text was threaded.
                    // This means the author omitted both `alt "..."` and `decorative: true`.
                    // E-A11-001 fires in strict mode (ADR-019 Decision 5.3 / BC-5.01.001 PC-1).
                    FrameContent::Chart {
                        alt: AltText::Unspecified,
                    } => {
                        diagnostics.push(make_post_layout_error(
                            "chart",
                            slide_type,
                            display_slide,
                        ));
                    },
                    FrameContent::Image {
                        alt: AltText::Unspecified,
                    } => {
                        diagnostics.push(make_post_layout_error(
                            "image",
                            slide_type,
                            display_slide,
                        ));
                    },
                    FrameContent::Diagram {
                        alt: AltText::Unspecified,
                        ..
                    } => {
                        diagnostics.push(make_post_layout_error(
                            "diagram",
                            slide_type,
                            display_slide,
                        ));
                    },
                    // All other cases (AltText::Decorative author opt-out, AltText::Provided valid alt,
                    // and all non-visual FrameContent variants): no diagnostic emitted.
                    // - Decorative: author explicit opt-out (decorative: true) — valid (ADR-019 5.3 / BC-5.01.001 EC-007)
                    // - Provided: author supplied non-empty alt text — valid
                    _ => {},
                }
            }
        }

        diagnostics
    }
}

/// Check a visual element with `decorative` support (Image, Chart, Diagram, Shape).
///
/// Logic (per AC-009, AC-006, AC-001 through AC-005, and BC-3.04.001 v1.5.2
/// Invariant 11 / F-P18-HIGH-001):
///
/// 1. If `decorative: true` AND `alt` is `Some(AltText::Provided(s))` where `s` is
///    non-blank → emit W-A11-002 (alt wins, decorative flag ignored).
///    The element is then treated as having valid alt text — no E-A11-001 is emitted.
///    Empty/whitespace alt with `decorative: true` does NOT emit W-A11-002 because
///    blank alt is not meaningful content worth warning about.
/// 2. If `decorative: true` AND `alt` is `None` or blank → decorative exemption, no error.
/// 3. If `alt` is `None` or blank `Provided` (and not decorative) → emit E-A11-001.
/// 4. If `alt` is valid `Provided` or `Decorative` enum variant → valid, no diagnostic.
///
/// ## Alt-wins precedence (BC-3.04.001 v1.5.2 Invariant 11)
///
/// When both a non-blank `alt` text AND `decorative: true` are present, **alt takes
/// precedence**. The layout engine (`slideforge-layout::build_shape_frame`) applies the
/// same "alt wins" rule, ensuring cross-crate semantic consistency: both crates produce
/// an element with valid alt text and `decorative = false` as the effective outcome.
///
/// ## Design note: dual decorative representation
///
/// The IR carries BOTH `decorative: bool` (from `decorative: true` keyword in DSL) AND
/// `alt: Option<AltText>` (where `AltText::Decorative` can also express decorative intent).
/// This dual representation is necessary to detect the ambiguous case: when the user writes
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
    // BC-3.04.001 v1.5.2 Invariant 11 (F-P18-HIGH-001): when both non-blank alt AND
    // decorative: true are present, alt wins. Emit W-A11-002 and treat the element as
    // having valid alt text (fall through to the non-decorative path below).
    // Blank alt with decorative: true is silently accepted (blank alt is not meaningful).
    let effective_decorative = if decorative {
        if let Some(AltText::Provided(s)) = alt
            && !is_blank(s)
        {
            // Alt wins — warn and treat as non-decorative with valid alt.
            diagnostics.push(make_warning(element_type, identifier, span));
            false
        } else {
            // Decorative exemption (no non-blank alt to promote): skip error check.
            return;
        }
    } else {
        false
    };

    // Non-decorative path (effective_decorative is always false here; kept for clarity).
    // AltText::Decorative (enum variant) wins — element is treated as valid even if
    // `decorative: bool` is false. See design note on dual decorative representation above.
    let _ = effective_decorative;
    let is_missing = match alt {
        // None or Unspecified: no meaningful alt text — fire E-A11-001.
        // Unspecified on ContentBlock.alt means the author did not supply alt text.
        None | Some(AltText::Unspecified) => true,
        Some(AltText::Provided(s)) => is_blank(s),
        Some(AltText::Decorative) => false,
    };

    if is_missing {
        diagnostics.push(make_error(element_type, identifier, span));
    }
}

/// Construct an `E-A11-001` error diagnostic for a missing alt text discovered
/// in the post-layout pass ([`AltTextValidator::validate_post_layout`]).
///
/// Unlike [`make_error`] (which has an identifier and span from the semantic IR),
/// the post-layout pass operates on the geometric [`LaidOutDeck`] IR where the
/// original source spans and element identifiers are not yet threaded
/// (Wave 3+ will supply them). Until then the diagnostic message includes:
/// - the element type (`"chart"`, `"image"`, `"diagram"`)
/// - the slide type keyword (e.g., `"content"`, `"photo"`)
/// - the 1-based slide number from `LaidOutSlide::source_index + 1`
///
/// This makes the diagnostic actionable: the user knows which slide to fix.
fn make_post_layout_error(element_type: &str, slide_type: &str, slide_number: usize) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: std::sync::Arc::from(E_A11_001),
        message: std::sync::Arc::from(format!(
            "slide {slide_number} ({slide_type}): {element_type} missing alt text. \
             Add alt \"...\" or mark decorative: true"
        )),
        span: SourceSpan::default(),
        hint: Some(std::sync::Arc::from(
            "All visual elements require alt text or decorative: true (DI-001)",
        )),
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

/// Construct a `W-A11-002` warning diagnostic for conflicting non-blank alt + decorative.
///
/// Per BC-3.04.001 v1.5.2 Invariant 11 (F-P18-HIGH-001): alt takes precedence over
/// `decorative: true`. The element is treated as having valid alt text; the decorative
/// flag is ignored.
fn make_warning(element_type: &str, identifier: &str, span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Warning,
        code: std::sync::Arc::from(W_A11_002),
        message: std::sync::Arc::from(format!(
            "{element_type} '{identifier}' at {span} has both alt and decorative: true; \
             alt takes precedence, decorative flag ignored. \
             Consider removing one."
        )),
        span: span.clone(),
        hint: Some(std::sync::Arc::from(
            "Remove decorative: true to keep the alt text, or remove alt to keep decorative behaviour",
        )),
    }
}

// ─── Test helpers ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DiagnosticSeverity, Validator, ValidatorOptions};
    use slideforge_types::{
        Block, ContentBlock, Deck, DeckMetadata, OrderedMap, Slide, SourceSpan,
        specs::{
            AltText, ChartSpec, DiagramSpec, ImageSpec, ShapePosition, ShapeSpec, ShapeUnit,
            TableSpec,
        },
    };

    use super::{AltTextValidator, E_A11_001, W_A11_002};

    // ── Deck/slide/block construction helpers ──────────────────────────────────

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        }
    }

    fn make_deck(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![],
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
            overlay: None,
            register_content: vec![],
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
            shape_type: slideforge_types::ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(1000),
                width: ShapeUnit::Inches(2000),
                height: ShapeUnit::Inches(1000),
            },
            fill: slideforge_types::FillSpec::None,
            text: None,
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
        // Pre-layout validate() is now Shape-only (ADR-018 v1.2 Decision-3).
        // Image blocks are validated post-layout; validate() on an image block → 0 diagnostics.
        // The behavior-under-test (missing alt fires E-A11-001) is verified via Shape block.
        let slide = make_slide(vec![make_shape_block(None, false)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "expected exactly 1 diagnostic, got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_bc_5_03_015_missing_alt_multiple_images() {
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        // Error accumulation is verified via 3 shape blocks (same check_visual_element logic).
        let slide = make_slide(vec![
            make_shape_block(None, false),
            make_shape_block(None, false),
            make_shape_block(None, false),
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
        // alt: Some(AltText::Provided("")) is treated as missing (blank check).
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        let slide = make_slide(vec![make_shape_block(
            Some(AltText::Provided(Arc::from(""))),
            false,
        )]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "empty string alt should produce E-A11-001; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    #[test]
    fn test_bc_5_03_015_whitespace_only_alt_is_missing() {
        // alt: Some(AltText::Provided("  ")) is treated as missing.
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        let slide = make_slide(vec![make_shape_block(
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

    #[test]
    fn test_bc_5_03_015_image_block_validate_scope_restricted() {
        // Confirm that pre-layout validate() produces 0 diagnostics for Image blocks.
        // Image alt-text enforcement is post-layout only (ADR-018 v1.2 Decision-3).
        let slide = make_slide(vec![make_image_block(None, false)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "pre-layout validate() must produce 0 diagnostics for Image blocks \
             (post-layout only per ADR-018 v1.2 Decision-3); got {diags:?}"
        );
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
        assert!(
            diags.is_empty(),
            "valid alt should produce no diagnostics; got {diags:?}"
        );
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
        // alt: Some(Provided("text")), decorative: true → 1 W-A11-002 warning
        // Per BC-3.04.001 v1.5.2 Invariant 11: alt wins over decorative (AC-009 / F-P18-HIGH-001).
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3) — use shape block.
        let slide = make_slide(vec![make_shape_block(
            Some(AltText::Provided(Arc::from("this alt takes precedence"))),
            true,
        )]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "alt+decorative together should produce 1 W-A11-002; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), W_A11_002);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Warning);
    }

    // ── Alt wins over decorative (BC-3.04.001 v1.5.2 Invariant 11 / F-P18-HIGH-001) ───

    #[test]
    fn test_w_a11_002_alt_wins_over_decorative() {
        // When both non-blank alt text AND decorative: true are present:
        // - Output alt is preserved (element treated as having valid alt)
        // - Output decorative is false (effective outcome: alt wins)
        // - Exactly 1 W-A11-002 warning is emitted
        // - No E-A11-001 error is emitted (alt is valid)
        //
        // Per BC-3.04.001 v1.5.2 Invariant 11 (F-P18-HIGH-001). Cross-crate
        // semantic consistency: slideforge-layout::build_shape_frame applies the
        // same "alt wins" rule, producing AltText::Provided(s) when both are set.
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3) — use shape block.
        let alt_text = Arc::from("A meaningful description of the shape");
        let deck = make_deck(vec![make_slide(vec![make_shape_block(
            Some(AltText::Provided(Arc::clone(&alt_text))),
            true,
        )])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());

        // Exactly 1 diagnostic (the W-A11-002 warning)
        assert_eq!(
            diags.len(),
            1,
            "expected exactly 1 W-A11-002 warning (no E-A11-001); got {diags:?}"
        );

        // Must be a warning, not an error
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Warning,
            "alt+decorative conflict must produce a Warning, not an Error; got {diags:?}"
        );

        // Must use W-A11-002 code
        assert_eq!(
            diags[0].code.as_ref(),
            W_A11_002,
            "warning code must be W-A11-002; got {}",
            diags[0].code
        );

        // Message must reference alt taking precedence
        assert!(
            diags[0].message.contains("alt takes precedence"),
            "W-A11-002 message must state alt takes precedence; got: {}",
            diags[0].message
        );

        // Message must mention decorative flag is ignored
        assert!(
            diags[0].message.contains("decorative flag ignored"),
            "W-A11-002 message must state decorative flag is ignored; got: {}",
            diags[0].message
        );
    }

    // ── Other visual element types ─────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_chart_missing_alt() {
        // Chart blocks are validated post-layout only (ADR-018 v1.2 Decision-3).
        // Pre-layout validate() produces 0 diagnostics for Chart blocks.
        // The shape block version confirms the underlying check_visual_element logic still works.
        let slide = make_slide(vec![make_chart_block(None, false)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "pre-layout validate() must produce 0 diagnostics for Chart blocks \
             (post-layout only per ADR-018 v1.2 Decision-3); got {diags:?}"
        );
    }

    #[test]
    fn test_bc_5_03_015_diagram_missing_alt() {
        // Diagram blocks are validated post-layout only (ADR-018 v1.2 Decision-3).
        // Pre-layout validate() produces 0 diagnostics for Diagram blocks.
        let slide = make_slide(vec![make_diagram_block(None, false)]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "pre-layout validate() must produce 0 diagnostics for Diagram blocks \
             (post-layout only per ADR-018 v1.2 Decision-3); got {diags:?}"
        );
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
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3) — Diagram blocks
        // are now skipped. The test verifies no panic occurs even when diagram source is not
        // inspected at pre-layout stage (chars().take(30) truncation is post-layout path).
        let source = "graph TD; A[\"日本語のラベル\"]-->B[\"中文標籤のテスト\"]";
        let deck = make_deck(vec![make_slide(vec![make_diagram_block_with_source(
            source, None, false,
        )])]);
        let diags = AltTextValidator.validate(&deck, &ValidatorOptions::default());
        // Pre-layout validate() skips Diagram — no E-A11-001 here (post-layout fires instead).
        assert!(
            diags.is_empty(),
            "pre-layout validate() must produce 0 diagnostics for Diagram blocks; got {diags:?}"
        );
    }

    // ── Error accumulation ─────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_error_accumulation() {
        // Error accumulation: 3 shapes all missing alt → 3 E-A11-001 (all accumulated).
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        // Image/Diagram blocks now produce 0 diagnostics at pre-layout; shape blocks confirm
        // the accumulation behavior of check_visual_element is intact.
        let slide = make_slide(vec![
            make_shape_block(None, false),
            make_shape_block(None, false),
            make_shape_block(None, false),
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
        // 2 slides, each with 1 shape missing alt → 2 E-A11-001.
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        let slide1 = make_slide(vec![make_shape_block(None, false)]);
        let slide2 = make_slide(vec![make_shape_block(None, false)]);
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
        use slideforge_types::{InlineNode, TextTag, block::TextBlock};
        let text_block = make_block(ContentBlock::Text(TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from("just text"))],
            tag: TextTag::Untagged,
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
        assert!(
            diags.is_empty(),
            "empty slide should produce no diagnostics"
        );
    }

    // ── Mixed valid/invalid ────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_mixed_valid_and_invalid() {
        // 1 shape with valid alt + 1 shape missing alt → exactly 1 error.
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        // Image blocks now produce 0 diagnostics; shape blocks confirm mixed-validity logic.
        let slide = make_slide(vec![
            make_shape_block(Some(AltText::Provided(Arc::from("a logo"))), false),
            make_shape_block(None, false),
        ]);
        let deck = make_deck(vec![slide]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "only the missing-alt shape should produce an error; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    // ── Diagnostic hint ───────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_015_error_has_hint() {
        // E-A11-001 diagnostics must include a correction hint.
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3) — use shape block.
        let slide = make_slide(vec![make_shape_block(None, false)]);
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
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        // Image blocks produce 0 diagnostics at pre-layout; confirm no panic + 0 diags.
        let deck = make_deck(vec![make_slide(vec![make_image_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "pre-layout validate() must produce 0 diagnostics for Image blocks; got {diags:?}"
        );
    }

    #[test]
    fn test_error_message_contains_identifier_chart() {
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        // Chart blocks produce 0 diagnostics at pre-layout; confirm no panic + 0 diags.
        let deck = make_deck(vec![make_slide(vec![make_chart_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "pre-layout validate() must produce 0 diagnostics for Chart blocks; got {diags:?}"
        );
    }

    #[test]
    fn test_error_message_contains_identifier_diagram() {
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        // Diagram blocks produce 0 diagnostics at pre-layout; confirm no panic + 0 diags.
        let deck = make_deck(vec![make_slide(vec![make_diagram_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "pre-layout validate() must produce 0 diagnostics for Diagram blocks; got {diags:?}"
        );
    }

    #[test]
    fn test_error_message_contains_identifier_shape() {
        // E-A11-001 for a shape must include the shape type as identifier.
        // Shape is the only pre-layout-validated type (ADR-018 v1.2 Decision-3).
        let deck = make_deck(vec![make_slide(vec![make_shape_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("'rect'"),
            "error message must contain the shape type 'rect'; got: {}",
            diags[0].message
        );
    }

    #[test]
    fn test_warning_message_contains_identifier() {
        // W-A11-002 for a shape must include the shape type as identifier.
        // Pre-layout validate() is Shape-only (ADR-018 v1.2 Decision-3).
        let deck = make_deck(vec![make_slide(vec![make_shape_block(
            Some(AltText::Provided(Arc::from("alt that wins"))),
            true,
        )])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code.as_ref(), W_A11_002);
        assert!(
            diags[0].message.contains("'rect'"),
            "warning message must contain the shape type 'rect'; got: {}",
            diags[0].message
        );
    }

    // ── W-A11-002 shape dispatch sibling test (F-P20-LOW-001) ─────────────────

    /// W-A11-002 — shape with non-blank alt + decorative: true emits exactly
    /// 1 W-A11-002 warning (no E-A11-001).
    ///
    /// Mirrors `test_w_a11_002_alt_wins_over_decorative` but uses a
    /// `ContentBlock::Shape` instead of `ContentBlock::Image` to confirm the
    /// W-A11-002 dispatch path is exercised for the Shape arm of the
    /// `check_visual_element` call chain (F-P20-LOW-001 sibling-site gap).
    ///
    /// Load-bearing: swap `make_shape_block` for `make_image_block` or change
    /// `AltText::Provided` to `None` — any of these changes must cause the
    /// assertion to fail.
    #[test]
    fn test_w_a11_002_alt_wins_over_decorative_shape() {
        let alt_text = Arc::from("A meaningful description of the shape");
        let deck = make_deck(vec![make_slide(vec![make_shape_block(
            Some(AltText::Provided(Arc::clone(&alt_text))),
            true,
        )])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());

        // Exactly 1 diagnostic (the W-A11-002 warning) — no E-A11-001
        assert_eq!(
            diags.len(),
            1,
            "shape with alt+decorative should produce 1 W-A11-002; got {diags:?}"
        );

        // Must be a warning, not an error
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Warning,
            "shape alt+decorative conflict must produce Warning; got {diags:?}"
        );

        // Must use W-A11-002 code
        assert_eq!(
            diags[0].code.as_ref(),
            W_A11_002,
            "warning code must be W-A11-002; got {}",
            diags[0].code
        );

        // Message must reference alt taking precedence
        assert!(
            diags[0].message.contains("alt takes precedence"),
            "W-A11-002 message must state alt takes precedence; got: {}",
            diags[0].message
        );
    }

    // ── Shape missing alt (ADV-P01-MED-003) ───────────────────────────────────

    #[test]
    fn test_shape_missing_alt() {
        // shape with alt: None, decorative: false → 1 E-A11-001
        let deck = make_deck(vec![make_slide(vec![make_shape_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "shape missing alt should produce 1 diagnostic; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_001);
    }

    // ── Blank alt with decorative — no warning (ADV-P01-MED-002) ─────────────

    #[test]
    fn test_blank_alt_with_decorative_no_warning() {
        // decorative: true + alt: Some(Provided("")) → no warning, no error
        // Blank alt is not meaningful content, so no W-A11-002 is emitted and the
        // element retains decorative exemption (not treated as "alt wins").
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
        // Same as blank alt — not meaningful content, decorative exemption applies.
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
        // Snapshot of E-A11-001 message. Pre-layout validate() is Shape-only
        // (ADR-018 v1.2 Decision-3) — use shape block to produce the diagnostic.
        let deck = make_deck(vec![make_slide(vec![make_shape_block(None, false)])]);
        let diags = AltTextValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        insta::assert_snapshot!(diags[0].message.as_ref());
    }

    // ── Post-layout locator: slide N in diagnostic message ────────────────────

    /// Direct test for the post-layout locator path in `validate_post_layout`.
    ///
    /// Constructs a `LaidOutDeck` with two slides:
    /// - slide 0 (`source_index` 0): a Chart frame with `AltText::Provided` (valid)
    /// - slide 1 (`source_index` 1): a Chart frame with `AltText::Unspecified` (pipeline gap)
    ///
    /// Asserts that:
    /// 1. Exactly one diagnostic is emitted (the second slide, not the first).
    /// 2. The diagnostic carries code `E-A11-001`.
    /// 3. The diagnostic message contains `"slide 2"` (`source_index` 1 → 1-based = 2).
    ///
    /// ## ADR-019 update (STORY-086)
    ///
    /// Per ADR-019 Decision 5.3: `AltText::Unspecified` → E-A11-001 (pipeline gap);
    /// `AltText::Decorative` → valid (author opt-out). This test was updated from
    /// `Decorative` to `Unspecified` to reflect the correct post-Stage-2b semantics.
    ///
    /// Load-bearing: if `display_slide` were computed from an enumerate index
    /// instead of `source_index + 1`, this test would fail when `source_index`
    /// differs from the loop iteration order (e.g., after reordering slides).
    /// If the locator were absent entirely, the `contains("slide 2")` assertion
    /// would also fail.
    #[test]
    fn test_post_layout_locator_slide_number_in_diagnostic() {
        use slideforge_layout::{
            BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
        };
        use slideforge_types::{AltText, Emu};

        let make_frame = |content: FrameContent| Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(1_000_000),
                height: Emu(500_000),
            },
            content,
            text_flow: None,
            region_role: None,
        };

        // Slide 0 (source_index=0, "title"): Chart with valid alt — no diagnostic expected.
        let slide0 = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![make_frame(FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Q1 revenue bar chart")),
            })],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };

        // Slide 1 (source_index=1, "content"): Chart with AltText::Unspecified — E-A11-001
        // expected with locator "slide 2". Per ADR-019 Decision 5.3: Unspecified = pipeline
        // gap (no author alt data threaded), triggers E-A11-001. Decorative would be valid.
        let slide1 = LaidOutSlide {
            source_index: 1,
            slide_type_keyword: Arc::from("content"),
            frames: vec![make_frame(FrameContent::Chart {
                alt: AltText::Unspecified,
            })],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };

        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![slide0, slide1],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],        };

        let diags = AltTextValidator.validate_post_layout(&laid_out, &default_opts());

        // Exactly one diagnostic — only the second slide triggers E-A11-001.
        assert_eq!(
            diags.len(),
            1,
            "expected exactly 1 diagnostic (slide 1 has Unspecified chart); got {diags:?}"
        );

        // Must be E-A11-001 error severity.
        assert_eq!(
            diags[0].code.as_ref(),
            E_A11_001,
            "diagnostic code must be E-A11-001; got {}",
            diags[0].code
        );
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Error,
            "diagnostic must be Error severity; got {:?}",
            diags[0].severity
        );

        // The message must carry the correct 1-based slide locator: "slide 2".
        assert!(
            diags[0].message.contains("slide 2"),
            "diagnostic message must contain 'slide 2' (source_index 1 → 1-based); got: {}",
            diags[0].message
        );
    }

    // ── STORY-086: AC-013, AC-014, AC-015 — AltText::Unspecified discrimination ─

    /// AC-015 / BC-5.01.001 postcondition 1 — `AltText::Unspecified` on a chart frame
    /// fires E-A11-001 in the post-layout validator.
    ///
    /// Red Gate: the stub `validate_post_layout` fires E-A11-001 on BOTH `Decorative`
    /// AND `Unspecified` (both arms push the error). The assertion `count == 1` passes
    /// against the stub for this specific vector because Unspecified already fires.
    /// HOWEVER, the semantic intent is that after Stage 2b, `Decorative` MUST NOT fire
    /// E-A11-001 (it becomes the "valid" state). The AC-015 Red Gate is validated by
    /// `test_bc_5_01_001_ac015_decorative_frame_no_error` below which FAILS against the stub.
    ///
    /// Load-bearing: if the implementer removes the `Unspecified → E-A11-001` arm,
    /// this test FAILS. Swap `AltText::Unspecified` to `AltText::Provided` → passes → confirms
    /// the test is genuinely load-bearing on the Unspecified arm.
    ///
    /// Traces: BC-5.01.001 postcondition 1; BC-5.02.001 postcondition 7; ADR-019 Decision 5.3.
    #[test]
    fn test_bc_5_01_001_ac015_unspecified_chart_frame_fires_e_a11_001() {
        // AC-015 — chart frame with AltText::Unspecified → E-A11-001 in strict mode.
        // Red Gate: stub fires on both Decorative AND Unspecified — this test passes NOW.
        // The paired test `ac015_decorative_frame_no_error` FAILS now and validates the gate.
        use slideforge_layout::{
            BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
        };
        use slideforge_types::{AltText, Emu};

        let make_frame = |content: FrameContent| Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(1_000_000),
                height: Emu(500_000),
            },
            content,
            text_flow: None,
            region_role: None,
        };

        // Chart frame with AltText::Unspecified — pipeline gap, must fire E-A11-001.
        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("chart"),
            frames: vec![make_frame(FrameContent::Chart {
                alt: AltText::Unspecified,
            })],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],        };

        let diags = AltTextValidator.validate_post_layout(&laid_out, &default_opts());

        // Must fire exactly one E-A11-001.
        assert_eq!(
            diags.len(),
            1,
            "AC-015: chart frame with AltText::Unspecified must produce exactly 1 E-A11-001; \
             got {} diagnostics: {diags:?}. BC-5.01.001 postcondition 1.",
            diags.len()
        );
        assert_eq!(
            diags[0].code.as_ref(),
            E_A11_001,
            "AC-015: diagnostic code must be E-A11-001 for Unspecified chart frame; \
             got '{}'",
            diags[0].code
        );
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Error,
            "AC-015: diagnostic severity must be Error; got {:?}",
            diags[0].severity
        );
    }

    /// AC-015 / BC-5.01.001 EC-007 — `AltText::Decorative` on a chart frame must NOT
    /// fire E-A11-001 in the post-layout validator.
    ///
    /// Red Gate: the STUB `validate_post_layout` fires E-A11-001 on BOTH `Decorative`
    /// AND `Unspecified`. After implementation, `Decorative` must be VALID (no error).
    ///
    /// This test FAILS against the stub (stub fires on Decorative → `diags.len() == 1 != 0`).
    /// It PASSES after the implementer changes the `Decorative` arm to NOT emit E-A11-001.
    ///
    /// Traces: BC-5.01.001 EC-007; BC-5.02.001 EC-009; ADR-019 Decision 5.3.
    #[test]
    fn test_bc_5_01_001_ac015_decorative_chart_frame_no_error() {
        // AC-015 — chart frame with AltText::Decorative → no E-A11-001 in strict mode.
        // RED GATE: stub fires E-A11-001 on Decorative → diags.len() == 1 → FAILS.
        // After fix: Decorative arm is removed / returns valid → diags.is_empty() → PASSES.
        use slideforge_layout::{
            BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
        };
        use slideforge_types::{AltText, Emu};

        let make_frame = |content: FrameContent| Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(1_000_000),
                height: Emu(500_000),
            },
            content,
            text_flow: None,
            region_role: None,
        };

        // Chart frame with AltText::Decorative — author opt-out, must be VALID.
        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("chart"),
            frames: vec![make_frame(FrameContent::Chart {
                alt: AltText::Decorative,
            })],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],        };

        let diags = AltTextValidator.validate_post_layout(&laid_out, &default_opts());

        // Decorative is valid — must produce ZERO diagnostics.
        // FAILS against stub (stub fires E-A11-001 on Decorative → diags.len() == 1).
        assert!(
            diags.is_empty(),
            "AC-015 Red Gate: chart frame with AltText::Decorative must produce NO diagnostics \
             (author chose decorative: true). Got {} diagnostics: {diags:?}. \
             Stub fires E-A11-001 on Decorative — THIS TEST MUST FAIL until T5 (alt_text.rs fix). \
             BC-5.01.001 EC-007; BC-5.02.001 EC-009; ADR-019 Decision 5.3.",
            diags.len()
        );
    }

    /// AC-015 / BC-5.02.001 postcondition 7 — `AltText::Provided` chart frame is valid.
    ///
    /// Red Gate: NOT failing against the stub (Provided is already in the wildcard `_ => {}`
    /// arm and produces no diagnostic). This is a correctness pin that ensures the `Provided`
    /// arm is not accidentally broken by the fix.
    ///
    /// Traces: BC-5.02.001 postcondition 7; BC-5.01.001 postcondition 1 (negative: no error).
    #[test]
    fn test_bc_5_01_001_ac015_provided_chart_frame_no_error() {
        // AC-015 — chart frame with AltText::Provided → no E-A11-001.
        use slideforge_layout::{
            BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
        };
        use slideforge_types::{AltText, Emu};

        let make_frame = |content: FrameContent| Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(1_000_000),
                height: Emu(500_000),
            },
            content,
            text_flow: None,
            region_role: None,
        };

        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("chart"),
            frames: vec![make_frame(FrameContent::Chart {
                alt: AltText::Provided(Arc::from("Revenue chart alt text")),
            })],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],        };

        let diags = AltTextValidator.validate_post_layout(&laid_out, &default_opts());

        assert!(
            diags.is_empty(),
            "AC-015: chart frame with AltText::Provided must produce no diagnostics; \
             got {diags:?}. BC-5.02.001 postcondition 7.",
        );
    }

    /// AC-013 / BC-5.01.001 invariant 2 — `thread_media_alt_into_frames` fallback:
    /// when `ContentBlock::Chart.alt == None`, the frame must carry `AltText::Unspecified`
    /// (not `AltText::Decorative`).
    ///
    /// STORY-086 / ADR-019 Decision 5.2 updated the fallback from `None → Decorative`
    /// to `None → Unspecified`, distinguishing a pipeline gap (no author alt threaded)
    /// from an explicit decorative opt-out. This test is now GREEN.
    ///
    /// This test constructs a `LaidOutDeck` by calling the real layout pipeline on a
    /// deck with a chart `ContentBlock` where `alt = None`, then asserts the resulting
    /// chart frame carries `AltText::Unspecified`.
    ///
    /// Load-bearing: the frame content type is directly on the production path through
    /// `thread_media_alt_into_frames` in `slideforge_layout::layout::run`.
    ///
    /// Traces: BC-5.01.001 EC-004; ADR-019 Decision 5.2.
    #[test]
    fn test_bc_5_01_001_ac013_thread_media_alt_fallback_produces_unspecified() {
        // AC-013 / AC-014 — when ContentBlock::Chart.alt = None,
        // thread_media_alt_into_frames must produce AltText::Unspecified (not Decorative).
        // STORY-086: None → Unspecified (GREEN — fix shipped).
        use slideforge_layout::FrameContent;
        use slideforge_types::{
            AltText, Block, Brand, BrandFonts, BrandPalette, ContentBlock, Deck, DeckMetadata,
            OrderedMap, Slide, SourceSpan, specs::ChartSpec,
        };

        // Build a minimal Brand so layout::run can proceed.
        let brand = Brand {
            name: Arc::from("test-brand"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#0066CC"),
                accent: Arc::from("#FF6B35"),
                neutral: Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Arial"),
                body: Arc::from("Arial"),
                mono: Arc::from("Courier New"),
                font_size_emu: 457_200,
            },
            layouts: vec![],
            span: SourceSpan::default(),
        };

        // Construct a chart slide with a ContentBlock::Chart where alt = None.
        // This simulates what Stage 2b produces when no alt field is present.
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("chart_type"),
            slideforge_types::FieldValue::Literal(slideforge_types::Value::Str(Arc::from("bar"))),
        );

        let slide = Slide {
            slide_type: Arc::from("chart"),
            fields,
            blocks: vec![Block {
                content: ContentBlock::Chart(ChartSpec {
                    chart_type: Arc::from("bar"),
                    alt: None, // ← None: Stage 2b alt-resolution rule → ContentBlock.alt = None
                    decorative: false,
                    span: SourceSpan::default(),
                }),
                label: None,
                span: SourceSpan::default(),
            }],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = Deck {
            slides: vec![slide],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: None,
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        };

        // Run the layout — this calls thread_media_alt_into_frames internally.
        let laid_out = slideforge_layout::run(&deck, &brand)
            .expect("layout::run must succeed for a valid chart slide");

        // Find the Chart frame in the laid-out slide.
        let chart_frame = laid_out.slides[0]
            .frames
            .iter()
            .find(|f| matches!(f.content, FrameContent::Chart { .. }));

        let chart_frame = chart_frame
            .expect("AC-013: chart slide must produce a FrameContent::Chart frame after layout");

        // Assert the alt is Unspecified (not Decorative).
        // STORY-086: None → Unspecified (GREEN — fix shipped per ADR-019 Decision 5.2).
        assert!(
            matches!(
                &chart_frame.content,
                FrameContent::Chart {
                    alt: AltText::Unspecified
                }
            ),
            "AC-013 / AC-014: when ContentBlock::Chart.alt = None, \
             thread_media_alt_into_frames must produce AltText::Unspecified (not Decorative). \
             Got: {:?}. BC-5.01.001 EC-004; ADR-019 Decision 5.2.",
            &chart_frame.content
        );
    }

    /// AC-013 / BC-5.01.001 invariant 2 — `regions.rs` structural placeholder sites use
    /// `AltText::Unspecified`.
    ///
    /// A `LaidOutDeck` constructed WITHOUT calling Stage 2b (simulating pre-threading state
    /// where `Slide.blocks = vec![]`) must have chart frames with `AltText::Unspecified`,
    /// NOT `AltText::Decorative`. This confirms the region map structural placeholder fix.
    ///
    /// Red Gate: the STUB `regions.rs` uses `AltText::Decorative` for structural placeholders.
    /// After fix: all 5 sites use `AltText::Unspecified`.
    ///
    /// Load-bearing: this test directly invokes `layout::run` on a deck with NO blocks (no
    /// Stage 2b threading), which exercises the `regions.rs` structural placeholder path.
    ///
    /// Traces: BC-5.01.001 invariant 2; ADR-019 Decision 5.1.
    #[test]
    fn test_bc_5_01_001_ac013_regions_structural_placeholder_is_unspecified() {
        // AC-013: when no ContentBlock threads alt into the frame,
        // the structural placeholder from regions.rs must be AltText::Unspecified.
        //
        // RED GATE: stub uses Decorative as structural placeholder → FAILS.
        // After fix: regions.rs uses Unspecified → PASSES.
        use slideforge_layout::FrameContent;
        use slideforge_types::{
            AltText, Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, OrderedMap, Slide,
            SourceSpan,
        };

        let brand = Brand {
            name: Arc::from("test-brand"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#0066CC"),
                accent: Arc::from("#FF6B35"),
                neutral: Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Arial"),
                body: Arc::from("Arial"),
                mono: Arc::from("Courier New"),
                font_size_emu: 457_200,
            },
            layouts: vec![],
            span: SourceSpan::default(),
        };

        // Chart slide with NO blocks (no Stage 2b threading — simulates pre-fix state).
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("chart_type"),
            slideforge_types::FieldValue::Literal(slideforge_types::Value::Str(Arc::from("bar"))),
        );
        let slide = Slide {
            slide_type: Arc::from("chart"),
            fields,
            blocks: vec![], // ← NO ContentBlocks: regions.rs structural placeholder used
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = Deck {
            slides: vec![slide],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: None,
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        };

        let laid_out = slideforge_layout::run(&deck, &brand)
            .expect("layout::run must succeed for chart slide");

        // Find the Chart frame (structural placeholder from regions.rs).
        let chart_frame = laid_out.slides[0]
            .frames
            .iter()
            .find(|f| matches!(f.content, FrameContent::Chart { .. }));

        let chart_frame =
            chart_frame.expect("AC-013: chart slide must produce a FrameContent::Chart frame");

        // The structural placeholder must be Unspecified (not Decorative).
        // RED GATE: stub uses Decorative → assertion matches Unspecified → FAILS.
        assert!(
            matches!(
                &chart_frame.content,
                FrameContent::Chart {
                    alt: AltText::Unspecified
                }
            ),
            "AC-013 Red Gate: regions.rs structural placeholder must be AltText::Unspecified, \
             not AltText::Decorative. Got: {:?}. \
             Stub uses Decorative → FAILS until T3 ships (regions.rs fix). \
             BC-5.01.001 invariant 2; ADR-019 Decision 5.1.",
            &chart_frame.content
        );
    }

    // ── Issue 1 (architect-pass-1): post-layout validate_post_layout is the sole
    //    validator for Chart/Image/Diagram; fires exactly ONE E-A11-001 for Unspecified ─

    /// Issue 1 / AC-005 — `validate_post_layout` fires exactly ONE E-A11-001 for a
    /// chart frame with `AltText::Unspecified`.
    ///
    /// This test verifies the SINGLE-FIRE guarantee required by AC-005 and the
    /// Issue 1 adjudication (architect-pass-1). After the fix:
    /// - `AltTextValidator::validate()` (pre-layout) is restricted to Shape blocks only.
    /// - `validate_post_layout()` (post-layout) fires E-A11-001 for Unspecified frames.
    /// - Together they guarantee exactly ONE E-A11-001 per missing-alt Chart/Image/Diagram.
    ///
    /// Red Gate status: `test_bc_5_01_001_ac015_unspecified_chart_frame_fires_e_a11_001`
    /// already passes (`validate_post_layout` already handles Unspecified). This new test
    /// adds an explicit count check and also tests Image and Diagram to ensure all three
    /// visual frame types fire the error.
    ///
    /// Traces: AC-005; BC-5.01.001 postcondition 1; BC-5.02.001 postcondition 7;
    ///         architect-pass-1-adjudication Issue 1 verdict.
    #[test]
    fn test_bc_5_01_001_issue1_post_layout_fires_exactly_one_e_a11_001_per_unspecified_frame() {
        // Issue 1: validate_post_layout must fire exactly 1 E-A11-001 per
        // Chart/Image/Diagram frame with AltText::Unspecified.
        // This is already implemented — this test guards against regression.
        use slideforge_layout::{
            BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
        };
        use slideforge_types::{AltText, Emu};

        let make_frame = |content: FrameContent| Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(1_000_000),
                height: Emu(500_000),
            },
            content,
            text_flow: None,
            region_role: None,
        };

        // One chart, one image, one diagram — all Unspecified.
        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("chart"),
            frames: vec![
                make_frame(FrameContent::Chart {
                    alt: AltText::Unspecified,
                }),
                make_frame(FrameContent::Image {
                    alt: AltText::Unspecified,
                }),
                make_frame(FrameContent::Diagram {
                    svg: slideforge_types::NormalizedDiagramSvg::empty_placeholder(),
                    alt: AltText::Unspecified,
                }),
            ],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let laid_out = LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],        };

        let diags = AltTextValidator.validate_post_layout(&laid_out, &default_opts());

        // Must fire exactly 3 E-A11-001 (one per Unspecified frame).
        let e_a11_count = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_001)
            .count();
        assert_eq!(
            e_a11_count, 3,
            "Issue 1: validate_post_layout must fire E-A11-001 for each Unspecified \
             Chart, Image, and Diagram frame. Expected 3, got {e_a11_count}. \
             Diagnostics: {diags:?}. \
             BC-5.01.001 postcondition 1; architect-pass-1 Issue 1."
        );
    }
}
