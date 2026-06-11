//! Canvas overflow validator (STORY-016).
//!
//! [`CanvasOverflowValidator`] detects when a slide's bullet content is likely
//! to overflow the body placeholder at standard font sizes. It uses a
//! conservative line-height estimate in EMU units, multiplied by the total
//! bullet count, and compares the result against the standard body placeholder
//! height.
//!
//! This is a heuristic check — it cannot know the exact rendered dimensions
//! before layout occurs, but it catches obvious overflow (e.g., 30 bullets
//! on a single slide) at validation time before any export runs.
//!
//! ## Overflow threshold
//!
//! The threshold is based on:
//! - Standard body placeholder height: 4,343,400 EMU (~4.75 inches)
//! - Line height per bullet: 11pt body × 1.3 line-height = 14.3pt ≈ 181,610 EMU
//! - At ≥ 24 bullets: estimated height exceeds placeholder → overflow
//!
//! ## Severity
//!
//! | Condition | Severity |
//! |-----------|----------|
//! | `strict_overflow: false` (default) | `Warning` |
//! | `strict_overflow: true` | `Error` |
//!
//! ## Error codes
//!
//! | Code | Severity | Meaning |
//! |------|----------|---------|
//! | `E-LAY-001` | Warning (or Error if `strict_overflow`) | Estimated content height exceeds body placeholder |

use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
use slideforge_types::{Deck, Emu, Slide, SourceSpan};

use crate::ValidationConfig;

/// Error code emitted when estimated content height exceeds the body placeholder.
pub(crate) const E_LAY_001: &str = "E-LAY-001";

/// Standard body placeholder height used for overflow estimation.
///
/// Body placeholder height: 4,343,400 EMU (~4.75 inches).
/// This covers the usable body area for a standard 16:9 widescreen slide
/// with a title bar at the top.
pub(crate) const BODY_PLACEHOLDER_HEIGHT: Emu = Emu(4_343_400);

/// Per-bullet line height estimate: 11pt body × 1.3 line-height = 14.3pt ≈ 181,610 EMU.
///
/// Calculated as 14.3pt × 12,700 EMU/pt = 181,610 EMU. At ≥ 24 bullets the
/// estimated height exceeds the body placeholder and overflow is detected.
pub(crate) const LINE_HEIGHT_PER_BULLET: Emu = Emu(181_610);

/// Validates that bullet content does not visually overflow the body placeholder.
///
/// Uses a heuristic EMU estimate: `bullet_count × LINE_HEIGHT_PER_BULLET`.
/// When the estimate exceeds `BODY_PLACEHOLDER_HEIGHT`, an `E-LAY-001`
/// diagnostic is emitted for the slide.
///
/// Severity is controlled by [`CanvasOverflowValidator::strict_overflow`]:
/// - `false` (default): `Warning` — reported but does not abort export
/// - `true`: `Error` — aborts export in strict validation mode
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_validator`].
pub struct CanvasOverflowValidator {
    /// When `true`, overflow is `Error` severity; when `false`, it is `Warning`.
    pub strict_overflow: bool,
}

impl CanvasOverflowValidator {
    /// Construct a [`CanvasOverflowValidator`] from a [`ValidationConfig`].
    ///
    /// The `strict_overflow` field is taken directly from
    /// [`ValidationConfig::strict_overflow`].
    #[must_use]
    pub fn from_config(config: &ValidationConfig) -> Self {
        Self {
            strict_overflow: config.strict_overflow,
        }
    }
}

impl Validator for CanvasOverflowValidator {
    fn id(&self) -> &'static str {
        "canvas-overflow"
    }

    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for slide in &deck.slides {
            let bullet_count = count_bullets(slide);
            let estimated = estimate_height(bullet_count);
            if estimated > BODY_PLACEHOLDER_HEIGHT {
                let severity = if self.strict_overflow {
                    DiagnosticSeverity::Error
                } else {
                    DiagnosticSeverity::Warning
                };
                let title = slide_title(slide);
                let overflow_emu = estimated - BODY_PLACEHOLDER_HEIGHT;
                diagnostics.push(make_overflow_diagnostic(
                    &title,
                    overflow_emu,
                    severity,
                    &slide.source_span,
                ));
            }
        }

        diagnostics
    }

    /// Detect silently-stacked bullet frames after the layout pass (F-094-P3-002).
    ///
    /// The pre-layout `validate()` catches overflow by counting bullets against a
    /// heuristic threshold (≥ 24). However, that heuristic misses a subtler defect:
    /// when the layout engine overflows and clamps all excess frames to identical
    /// coordinates (y = `page_height` − 1, height = 1). The count may be below 24
    /// while the visual output is broken.
    ///
    /// This method detects that condition geometrically: it checks whether any
    /// [`slideforge_layout::FrameContent::TextRun`] frames within a single slide
    /// share an identical [`slideforge_layout::BoundingBox`]. Two or more
    /// identically-positioned frames in the same slide means at least one frame is
    /// visually invisible (stacked behind another) — silent overflow.
    ///
    /// When stacking is detected an `E-LAY-001` diagnostic is emitted.
    /// Severity respects [`CanvasOverflowValidator::strict_overflow`]:
    /// - `false` (default): `Warning`
    /// - `true`: `Error`
    ///
    /// ## Complexity
    ///
    /// O(N) per slide where N = number of frames in the slide, using a `HashSet`.
    fn validate_post_layout(
        &self,
        laid_out: &slideforge_layout::LaidOutDeck,
        _opts: &ValidatorOptions,
    ) -> Vec<Diagnostic> {
        use slideforge_layout::{BoundingBox, FrameContent};
        use std::collections::HashSet;

        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for (slide_idx, slide) in laid_out.slides.iter().enumerate() {
            // Collect bboxes from TextRun frames only.
            // Non-TextRun frames (Title, Subtitle, Body, etc.) are intentionally
            // excluded — stacking is a bullet-overflow defect, not a layout artifact
            // of multi-region slides.
            let mut seen_bboxes: HashSet<BoundingBox> = HashSet::new();
            let mut stacked = false;
            for frame in &slide.frames {
                if let FrameContent::TextRun(_) = &frame.content
                    && !seen_bboxes.insert(frame.bbox)
                {
                    // bbox was already present → duplicate → stacking detected.
                    stacked = true;
                    break;
                }
            }

            if stacked {
                let severity = if self.strict_overflow {
                    DiagnosticSeverity::Error
                } else {
                    DiagnosticSeverity::Warning
                };
                let slide_label = slide.slide_type_keyword.as_ref();
                let source_span = slideforge_types::SourceSpan::default();
                diagnostics.push(Diagnostic {
                    severity,
                    code: std::sync::Arc::from(E_LAY_001),
                    message: std::sync::Arc::from(format!(
                        "CanvasOverflow: slide {slide_idx} (type '{slide_label}') has \
                         geometrically stacked TextRun frames — bullet content overflowed \
                         the body placeholder and frames were clamped to identical positions. \
                         Count-based detection did not fire. \
                         Split the slide or reduce bullet count (E-LAY-001)."
                    )),
                    span: source_span,
                    hint: Some(std::sync::Arc::from(
                        "Split the slide or reduce the number of bullet points (E-LAY-001)",
                    )),
                });
            }
        }

        diagnostics
    }
}

/// Count the total number of top-level bullet items in a slide's blocks.
///
/// Nested bullet children are NOT counted separately — the heuristic treats
/// each `BulletItem` at the top level as one line. Sub-bullets are included
/// in the parent's line height implicitly.
fn count_bullets(slide: &Slide) -> usize {
    use slideforge_types::ContentBlock;

    slide
        .blocks
        .iter()
        .map(|block| match &block.content {
            ContentBlock::Bullets(items) => items.len(),
            _ => 0,
        })
        .sum()
}

/// Practical upper bound on bullets per slide.
///
/// Any bullet count above this value is clamped before the EMU multiplication
/// to prevent `Emu * i64` overflow. 10,000 bullets is an absurdly large slide
/// by any practical measure; the clamped result will still be detected as an
/// overflow by the comparator.
const MAX_BULLET_COUNT: usize = 10_000;

/// Estimate the rendered height of a slide's bullet content in EMU.
///
/// Uses `bullet_count × LINE_HEIGHT_PER_BULLET` as a conservative heuristic.
///
/// `bullet_count` is clamped to [`MAX_BULLET_COUNT`] before conversion to
/// `i64` to prevent arithmetic overflow in the [`Emu`] multiplication. Any
/// count above the cap is still large enough to trigger an overflow diagnostic.
fn estimate_height(bullet_count: usize) -> Emu {
    // Clamp before conversion: i64::MAX as a multiplier would overflow Emu * i64.
    let capped = bullet_count.min(MAX_BULLET_COUNT);
    // Safety: MAX_BULLET_COUNT (10_000) << i64::MAX; the try_from cannot fail.
    let count_i64 = i64::try_from(capped).unwrap_or(10_000_i64);
    LINE_HEIGHT_PER_BULLET * count_i64
}

/// Extract the slide's title as a string for use in diagnostic messages.
///
/// Returns the value of the `title` field if it is a resolved `Value::Str`,
/// or falls back to the slide type keyword if the field is absent or unresolved.
fn slide_title(slide: &Slide) -> String {
    slide
        .title_str()
        .map_or_else(|| slide.slide_type.as_ref().to_owned(), str::to_owned)
}

/// Construct an `E-LAY-001` diagnostic for an overflowing slide.
///
/// The message format matches AC-001:
/// `CanvasOverflow: slide '<title>' field 'bullets' overflows by ~<N> EMU (~<M>pt). ...`
fn make_overflow_diagnostic(
    title: &str,
    overflow_emu: Emu,
    severity: DiagnosticSeverity,
    span: &SourceSpan,
) -> Diagnostic {
    // Precision loss is acceptable for the heuristic display of point values.
    // EMU values used here are slide layout dimensions, well within f64 precision
    // for human-readable diagnostic output.
    #[allow(clippy::cast_precision_loss)]
    let overflow_pt = overflow_emu.0 as f64 / 12_700.0;
    Diagnostic {
        severity,
        code: std::sync::Arc::from(E_LAY_001),
        message: std::sync::Arc::from(format!(
            "CanvasOverflow: slide '{title}' field 'bullets' overflows by ~{} EMU (~{overflow_pt:.1}pt). \
             Consider reducing content or font size.",
            overflow_emu.0,
        )),
        span: span.clone(),
        hint: Some(std::sync::Arc::from(
            "Split the slide or reduce the number of bullet points (E-LAY-001)",
        )),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DiagnosticSeverity, Validator, ValidatorOptions};
    use slideforge_types::{
        Block, BulletItem, ContentBlock, Deck, DeckMetadata, Emu, InlineNode, OrderedMap, Slide,
        SourceSpan, block::TextBlock,
    };

    use super::{
        BODY_PLACEHOLDER_HEIGHT, CanvasOverflowValidator, E_LAY_001, LINE_HEIGHT_PER_BULLET,
    };

    // ── Construction helpers ───────────────────────────────────────────────────

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
            slide_sections: vec![],
        }
    }

    /// Build a slide carrying `n` top-level bullet items in a single
    /// `ContentBlock::Bullets` block.
    fn make_slide_with_bullets(n: usize) -> Slide {
        let bullets: Vec<BulletItem> = (0..n)
            .map(|i| BulletItem {
                inlines: vec![InlineNode::Plain(Arc::from(format!("Bullet {i}")))],
                children: vec![],
                span: SourceSpan::default(),
            })
            .collect();

        let block = Block {
            content: ContentBlock::Bullets(bullets),
            label: None,
            span: SourceSpan::default(),
        };

        Slide {
            slide_type: Arc::from("bullets"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
            field_spans: OrderedMap::new(),
        }
    }

    /// Build a slide with only a `ContentBlock::Text` block (no bullets).
    fn make_slide_text_only() -> Slide {
        let text_block = Block {
            content: ContentBlock::Text(TextBlock {
                inlines: vec![InlineNode::Plain(Arc::from("Some paragraph text"))],
                tag: slideforge_types::TextTag::Untagged,
                span: SourceSpan::default(),
            }),
            label: None,
            span: SourceSpan::default(),
        };
        Slide {
            slide_type: Arc::from("content"),
            fields: OrderedMap::new(),
            blocks: vec![text_block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
            field_spans: OrderedMap::new(),
        }
    }

    fn default_opts() -> ValidatorOptions {
        ValidatorOptions::default()
    }

    fn validator_default() -> CanvasOverflowValidator {
        CanvasOverflowValidator {
            strict_overflow: false,
        }
    }

    fn validator_strict() -> CanvasOverflowValidator {
        CanvasOverflowValidator {
            strict_overflow: true,
        }
    }

    // ── Validator ID ──────────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_016_canvas_overflow_validator_id() {
        assert_eq!(validator_default().id(), "canvas-overflow");
    }

    // ── Threshold sanity: verify constants are self-consistent ────────────────

    /// 24 bullets × 181,610 EMU = 4,358,640 > 4,343,400 → overflow at 24 bullets.
    #[test]
    fn test_bc_5_03_016_overflow_threshold_constants_are_consistent() {
        let twenty_four_bullets = LINE_HEIGHT_PER_BULLET * 24;
        assert!(
            twenty_four_bullets > BODY_PLACEHOLDER_HEIGHT,
            "24 bullets must exceed placeholder height; \
             24 × {LINE_HEIGHT_PER_BULLET} = {twenty_four_bullets} vs {BODY_PLACEHOLDER_HEIGHT}"
        );
    }

    /// 20 bullets × 181,610 EMU = 3,632,200 < 4,343,400 → no overflow at 20 bullets.
    #[test]
    fn test_bc_5_03_016_no_overflow_threshold_constants_are_consistent() {
        let twenty_bullets = LINE_HEIGHT_PER_BULLET * 20;
        assert!(
            twenty_bullets < BODY_PLACEHOLDER_HEIGHT,
            "20 bullets must not exceed placeholder height; \
             20 × {LINE_HEIGHT_PER_BULLET} = {twenty_bullets} vs {BODY_PLACEHOLDER_HEIGHT}"
        );
    }

    // ── No overflow cases ──────────────────────────────────────────────────────

    /// 5 bullets comfortably fit — 0 diagnostics.
    #[test]
    fn test_no_overflow_5_bullets() {
        let deck = make_deck(vec![make_slide_with_bullets(5)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "5 bullets must produce 0 E-LAY-001 diagnostics; got {diags:?}"
        );
    }

    /// 0 bullets (empty slide) — 0 diagnostics.
    #[test]
    fn test_no_overflow_zero_bullets() {
        let deck = make_deck(vec![make_slide_with_bullets(0)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "0 bullets must produce 0 E-LAY-001 diagnostics; got {diags:?}"
        );
    }

    /// Text-only slide (no bullets) → 0 diagnostics.
    ///
    /// Only `ContentBlock::Bullets` contributes to the overflow estimate.
    #[test]
    fn test_no_overflow_text_only() {
        let deck = make_deck(vec![make_slide_text_only()]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "text-only slide must produce 0 E-LAY-001 diagnostics; got {diags:?}"
        );
    }

    /// 1 bullet — well under threshold, 0 diagnostics.
    #[test]
    fn test_no_overflow_1_bullet() {
        let deck = make_deck(vec![make_slide_with_bullets(1)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "1 bullet must produce 0 E-LAY-001 diagnostics; got {diags:?}"
        );
    }

    // ── Overflow cases ─────────────────────────────────────────────────────────

    /// 30 bullets definitively overflow → 1 E-LAY-001 diagnostic.
    /// (30 × 181,610 = 5,448,300 > 4,343,400)
    #[test]
    fn test_overflow_30_bullets() {
        let deck = make_deck(vec![make_slide_with_bullets(30)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "30 bullets must produce 1 E-LAY-001; got {diags:?}"
        );
        assert_eq!(
            diags[0].code.as_ref(),
            E_LAY_001,
            "diagnostic code must be E-LAY-001"
        );
    }

    /// 24 bullets is exactly at the threshold (24 × 181,610 = 4,358,640 > 4,343,400).
    #[test]
    fn test_overflow_at_threshold_24_bullets() {
        let deck = make_deck(vec![make_slide_with_bullets(24)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "24 bullets (at threshold) must produce 1 E-LAY-001; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_LAY_001);
    }

    // ── Severity — default (warn-only) vs strict ───────────────────────────────

    /// With `strict_overflow: false`, E-LAY-001 is a Warning.
    #[test]
    fn test_overflow_is_warning_default() {
        let deck = make_deck(vec![make_slide_with_bullets(30)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1, "expected 1 diagnostic; got {diags:?}");
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Warning,
            "with strict_overflow: false, E-LAY-001 must be Warning"
        );
    }

    /// With `strict_overflow: true`, E-LAY-001 is an Error.
    #[test]
    fn test_strict_overflow_is_error() {
        let deck = make_deck(vec![make_slide_with_bullets(30)]);
        let diags = validator_strict().validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1, "expected 1 diagnostic; got {diags:?}");
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Error,
            "with strict_overflow: true, E-LAY-001 must be Error"
        );
    }

    // ── Multi-slide accumulation ──────────────────────────────────────────────

    /// 3 slides all overflowing → 3 E-LAY-001 diagnostics (error accumulation).
    /// Uses 30, 25, 24 bullets (all exceed the ~24-bullet threshold).
    #[test]
    fn test_overflow_multiple_slides() {
        let deck = make_deck(vec![
            make_slide_with_bullets(30),
            make_slide_with_bullets(25),
            make_slide_with_bullets(24),
        ]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            3,
            "3 overflowing slides must produce 3 E-LAY-001 diagnostics; got {diags:?}"
        );
        for d in &diags {
            assert_eq!(d.code.as_ref(), E_LAY_001);
        }
    }

    /// Mixed: 1 overflowing slide, 1 clean slide → 1 E-LAY-001.
    #[test]
    fn test_overflow_one_of_two_slides() {
        let deck = make_deck(vec![
            make_slide_with_bullets(30), // overflows (30 × 181,610 > 4,343,400)
            make_slide_with_bullets(3),  // clean
        ]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "only 1 of 2 slides overflows, so 1 E-LAY-001; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_LAY_001);
    }

    // ── Diagnostic message quality ─────────────────────────────────────────────

    /// E-LAY-001 message must contain "EMU" and match AC-001 format.
    #[test]
    fn test_overflow_message_has_emu_estimate() {
        let deck = make_deck(vec![make_slide_with_bullets(30)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("EMU"),
            "E-LAY-001 message must reference EMU units; got: {}",
            diags[0].message
        );
        assert!(
            diags[0].message.contains("CanvasOverflow"),
            "E-LAY-001 message must start with 'CanvasOverflow'; got: {}",
            diags[0].message
        );
        assert!(
            diags[0].message.contains("field 'bullets'"),
            "E-LAY-001 message must reference field 'bullets'; got: {}",
            diags[0].message
        );
    }

    /// E-LAY-001 must include a correction hint.
    #[test]
    fn test_overflow_has_hint() {
        let deck = make_deck(vec![make_slide_with_bullets(30)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].hint.is_some(),
            "E-LAY-001 must include a correction hint"
        );
    }

    // ── Message format — AC-001 ────────────────────────────────────────────────

    /// E-LAY-001 message format: `CanvasOverflow`: slide '<title>' field 'bullets' overflows by ~N EMU (~Mpt).
    #[test]
    fn test_overflow_message_format_with_title() {
        // Slide with a title field set.
        let mut slide = make_slide_with_bullets(30);
        slide.fields.insert(
            Arc::from("title"),
            slideforge_types::FieldValue::Literal(slideforge_types::Value::Str(Arc::from(
                "My Slide",
            ))),
        );
        let deck = make_deck(vec![slide]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        let msg = diags[0].message.as_ref();
        assert!(
            msg.starts_with("CanvasOverflow: slide 'My Slide' field 'bullets' overflows by ~"),
            "message must match AC-001 format; got: {msg}"
        );
        assert!(
            msg.contains("pt)"),
            "message must include point conversion; got: {msg}"
        );
    }

    /// E-LAY-001 uses `slide_type` as title fallback when no title field is set.
    #[test]
    fn test_overflow_message_format_title_fallback() {
        let slide = make_slide_with_bullets(30); // slide_type = "bullets", no title field
        let deck = make_deck(vec![slide]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        let msg = diags[0].message.as_ref();
        assert!(
            msg.contains("slide 'bullets'"),
            "message must use slide_type as title fallback; got: {msg}"
        );
    }

    // ── Empty deck ─────────────────────────────────────────────────────────────

    /// Empty deck produces 0 overflow diagnostics (nothing to check).
    #[test]
    fn test_overflow_empty_deck_no_diagnostics() {
        let deck = make_deck(vec![]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "empty deck must produce 0 overflow diagnostics; got {diags:?}"
        );
    }

    // ── Overflow safety: very large bullet count must not panic ────────────────

    /// A bullet count well above `MAX_BULLET_COUNT` must not cause an arithmetic
    /// overflow panic in debug mode or silent wrapping in release.
    ///
    /// The `estimate_height` function clamps the count to `MAX_BULLET_COUNT`
    /// before converting to `i64`, preventing `Emu * i64::MAX` overflow. The
    /// clamped count is still large enough to trigger an overflow diagnostic.
    #[test]
    fn test_overflow_very_large_bullet_count_no_panic() {
        // 100_001 > MAX_BULLET_COUNT (10_000): exercises the clamp path.
        // This is small enough to actually allocate in a test.
        let deck = make_deck(vec![make_slide_with_bullets(100_001)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "a large bullet count above MAX_BULLET_COUNT must produce exactly 1 E-LAY-001 (not panic); got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_LAY_001);
    }

    /// Verify that `estimate_height` with `usize::MAX` as input does not panic.
    ///
    /// This tests the clamp directly without allocating `usize::MAX` bullets.
    #[test]
    fn test_estimate_height_usize_max_no_panic() {
        use super::{MAX_BULLET_COUNT, estimate_height};
        // estimate_height must clamp MAX to MAX_BULLET_COUNT and not panic.
        let h = estimate_height(usize::MAX);
        // Result must equal estimate_height(MAX_BULLET_COUNT).
        let expected = estimate_height(MAX_BULLET_COUNT);
        assert_eq!(
            h, expected,
            "estimate_height(usize::MAX) must equal estimate_height(MAX_BULLET_COUNT)"
        );
    }

    // ── F-094-P3-002 — Geometric overflow detection (post-layout) ─────────────

    /// F-094-P3-002: `validate_post_layout` must detect silently-stacked bullet
    /// frames (identical bounding boxes) caused by layout overflow-clamping and
    /// emit an E-LAY-001 diagnostic.
    ///
    /// 16 bullet frames are used — below the count-based threshold of 24 — but
    /// frames 6..15 are identical (stacked at page bottom). The count-based
    /// `validate()` would NOT catch this; `validate_post_layout()` MUST.
    ///
    /// RED: currently `validate_post_layout` is the inherited no-op from Validator
    /// trait. Fails until `CanvasOverflowValidator::validate_post_layout` is
    /// overridden to detect geometric overflow on `LaidOutDeck`.
    #[test]
    fn test_f094_p3_002_geometric_overflow_detected_not_count_based() {
        use slideforge_layout::{
            BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
        };

        // Build a LaidOutDeck simulating broken overflow-clamping:
        // 16 bullet frames, frames 5..15 are clamped to identical positions.
        let line_h = 228_600_i64; // layout::LINE_HEIGHT_EMU
        let body_top: i64 = 1_143_000;
        let body_height: i64 = 5 * line_h; // fits exactly 5 bullets
        let page_w: i64 = 9_144_000;
        let page_h: i64 = 5_143_500;
        let bullet_x: i64 = 457_200;
        let bullet_w: i64 = page_w - bullet_x;

        let make_frame = |y: i64, h: i64| -> Frame {
            Frame {
                bbox: BoundingBox {
                    x: Emu(bullet_x),
                    y: Emu(y),
                    width: Emu(bullet_w),
                    height: Emu(h),
                },
                content: FrameContent::TextRun(vec![]),
                text_flow: None,
                region_role: None,
            }
        };

        let mut frames = Vec::new();
        for i in 0..16_i64 {
            let y = body_top + i * line_h;
            if y + line_h <= body_top + body_height {
                frames.push(make_frame(y, line_h));
            } else {
                // Broken behaviour: overflow bullets clamped to page bottom.
                frames.push(make_frame(page_h - 1, 1));
            }
        }

        // Verify setup: ≥2 frames share identical stacked position.
        let stacked = frames
            .iter()
            .filter(|f| f.bbox.y == Emu(page_h - 1) && f.bbox.height == Emu(1))
            .count();
        assert!(
            stacked >= 2,
            "test setup: expected ≥2 stacked frames; got {stacked}"
        );

        // Confirm count-based would NOT fire (16 < 24).
        assert!(frames.len() < 24, "test setup: {}", frames.len());

        let laid_out = LaidOutDeck {
            page_size: PageSize {
                width: Emu(page_w),
                height: Emu(page_h),
            },
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("content"),
                frames,
                speaker_notes: None,
                register_tags: vec![],
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
        };

        let opts = ValidatorOptions::default();
        let diags = CanvasOverflowValidator {
            strict_overflow: false,
        }
        .validate_post_layout(&laid_out, &opts);

        assert!(
            !diags.is_empty(),
            "F-094-P3-002: geometric overflow with stacked frames (16 bullets, <24 count) \
             must produce ≥1 diagnostic from validate_post_layout; got 0. \
             Count-based detection is insufficient."
        );
        assert!(
            diags.iter().any(|d| d.code.as_ref() == E_LAY_001),
            "F-094-P3-002: overflow diagnostic must carry E-LAY-001 code; got: {diags:?}"
        );
    }

    /// F-094-P3-002 (b): When all frames have distinct positions, `validate_post_layout`
    /// must NOT produce a false-positive stacking diagnostic.
    #[test]
    fn test_f094_p3_002_no_false_positive_when_frames_are_distinct() {
        use slideforge_layout::{
            BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
        };

        let line_h = 228_600_i64;
        let page_w: i64 = 9_144_000;
        let page_h: i64 = 5_143_500;
        let bullet_x: i64 = 457_200;
        let bullet_w: i64 = page_w - bullet_x;

        // 5 bullets, all within the page — each with a distinct y position.
        let frames: Vec<Frame> = (0..5_i64)
            .map(|i| Frame {
                bbox: BoundingBox {
                    x: Emu(bullet_x),
                    y: Emu(i * line_h),
                    width: Emu(bullet_w),
                    height: Emu(line_h),
                },
                content: FrameContent::TextRun(vec![]),
                text_flow: None,
                region_role: None,
            })
            .collect();

        let laid_out = LaidOutDeck {
            page_size: PageSize {
                width: Emu(page_w),
                height: Emu(page_h),
            },
            slides: vec![LaidOutSlide {
                source_index: 0,
                slide_type_keyword: Arc::from("content"),
                frames,
                speaker_notes: None,
                register_tags: vec![],
                register_content: vec![],
            }],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
        };

        let opts = ValidatorOptions::default();
        let diags = CanvasOverflowValidator {
            strict_overflow: false,
        }
        .validate_post_layout(&laid_out, &opts);

        assert!(
            diags.is_empty(),
            "F-094-P3-002: 5 distinct frames must produce 0 stacking diagnostics; got: {diags:?}"
        );
    }
}
