//! Canvas overflow validator (STORY-016).
//!
//! [`CanvasOverflowValidator`] detects when a slide's bullet content is likely
//! to overflow the body placeholder at standard font sizes. It uses a
//! conservative line-height estimate in EMU units, multiplied by the total
//! bullet count, and compares the result against the standard body placeholder
//! height.
//!
//! This is a heuristic check — it cannot know the exact rendered dimensions
//! before layout occurs, but it catches obvious overflow (e.g., 20 bullets
//! on a single slide) at validation time before any export runs.
//!
//! ## Overflow threshold
//!
//! The threshold is based on:
//! - Standard body placeholder height: ~4,500,000 EMU (≈ 4.9 inches)
//! - Conservative line height: 457,200 EMU (≈ 0.5 inch per bullet at 36pt)
//! - At ≥ 10 bullets: estimated height exceeds placeholder → overflow
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

/// Error code emitted when estimated content height exceeds the body placeholder.
pub(crate) const E_LAY_001: &str = "E-LAY-001";

/// Standard body placeholder height used for overflow estimation.
///
/// Approximately 4.9 inches (≈ 4,500,000 EMU), which covers the usable body
/// area for a standard 16:9 widescreen slide with a title.
// The constants are used in tests and will be used by the implementer's
// validate() body. The #[allow] is removed once validate() is implemented.
#[allow(dead_code)]
pub(crate) const BODY_PLACEHOLDER_HEIGHT: Emu = Emu(4_500_000);

/// Conservative per-bullet line height at 36pt (0.5 inch = 457,200 EMU).
///
/// This is deliberately conservative — it accounts for line spacing, sub-bullets,
/// and top/bottom padding. A slide with 10 bullets at this estimate reaches the
/// threshold.
#[allow(dead_code)]
pub(crate) const LINE_HEIGHT_PER_BULLET: Emu = Emu(457_200);

/// Validates that bullet content does not visually overflow the body placeholder.
///
/// Uses a heuristic EMU estimate: `bullet_count × LINE_HEIGHT_PER_BULLET`.
/// When the estimate exceeds [`BODY_PLACEHOLDER_HEIGHT`], an `E-LAY-001`
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

impl Validator for CanvasOverflowValidator {
    fn id(&self) -> &'static str {
        "canvas-overflow"
    }

    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for (index, slide) in deck.slides.iter().enumerate() {
            let bullet_count = count_bullets(slide);
            let estimated = estimate_height(bullet_count);
            if estimated > BODY_PLACEHOLDER_HEIGHT {
                let severity = if self.strict_overflow {
                    DiagnosticSeverity::Error
                } else {
                    DiagnosticSeverity::Warning
                };
                // Use 1-based index in the message for user-facing display
                diagnostics.push(make_overflow_diagnostic(
                    index + 1,
                    estimated,
                    BODY_PLACEHOLDER_HEIGHT,
                    severity,
                    &slide.source_span,
                ));
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

/// Estimate the rendered height of a slide's bullet content in EMU.
///
/// Uses `bullet_count × LINE_HEIGHT_PER_BULLET` as a conservative heuristic.
fn estimate_height(bullet_count: usize) -> Emu {
    LINE_HEIGHT_PER_BULLET * i64::try_from(bullet_count).unwrap_or(i64::MAX)
}

/// Construct an `E-LAY-001` diagnostic for an overflowing slide.
fn make_overflow_diagnostic(
    slide_index: usize,
    estimated_height: Emu,
    capacity: Emu,
    severity: DiagnosticSeverity,
    span: &SourceSpan,
) -> Diagnostic {
    Diagnostic {
        severity,
        code: std::sync::Arc::from(E_LAY_001),
        message: std::sync::Arc::from(format!(
            "Slide {slide_index}: estimated content height {estimated_height} exceeds \
             body placeholder capacity {capacity} EMU. Reduce bullet count or font size."
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
        Block, BulletItem, ContentBlock, Deck, DeckMetadata, InlineNode, OrderedMap, Slide,
        SourceSpan,
        block::TextBlock,
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
        }
    }

    /// Build a slide with only a `ContentBlock::Text` block (no bullets).
    fn make_slide_text_only() -> Slide {
        let text_block = Block {
            content: ContentBlock::Text(TextBlock {
                inlines: vec![InlineNode::Plain(Arc::from("Some paragraph text"))],
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

    /// 10 bullets × 457,200 EMU = 4,572,000 > 4,500,000 → overflow at 10 bullets.
    #[test]
    fn test_bc_5_03_016_overflow_threshold_constants_are_consistent() {
        let ten_bullets = LINE_HEIGHT_PER_BULLET * 10;
        assert!(
            ten_bullets > BODY_PLACEHOLDER_HEIGHT,
            "10 bullets must exceed placeholder height; \
             10 × {LINE_HEIGHT_PER_BULLET} = {ten_bullets} vs {BODY_PLACEHOLDER_HEIGHT}"
        );
    }

    /// 5 bullets × 457,200 EMU = 2,286,000 < 4,500,000 → no overflow at 5 bullets.
    #[test]
    fn test_bc_5_03_016_no_overflow_threshold_constants_are_consistent() {
        let five_bullets = LINE_HEIGHT_PER_BULLET * 5;
        assert!(
            five_bullets < BODY_PLACEHOLDER_HEIGHT,
            "5 bullets must not exceed placeholder height; \
             5 × {LINE_HEIGHT_PER_BULLET} = {five_bullets} vs {BODY_PLACEHOLDER_HEIGHT}"
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

    /// 20 bullets definitively overflow → 1 E-LAY-001 diagnostic.
    #[test]
    fn test_overflow_20_bullets() {
        let deck = make_deck(vec![make_slide_with_bullets(20)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "20 bullets must produce 1 E-LAY-001; got {diags:?}"
        );
        assert_eq!(
            diags[0].code.as_ref(),
            E_LAY_001,
            "diagnostic code must be E-LAY-001"
        );
    }

    /// 10 bullets is exactly at the threshold (10 × 457,200 = 4,572,000 > 4,500,000).
    #[test]
    fn test_overflow_at_threshold_10_bullets() {
        let deck = make_deck(vec![make_slide_with_bullets(10)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "10 bullets (at threshold) must produce 1 E-LAY-001; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_LAY_001);
    }

    // ── Severity — default (warn-only) vs strict ───────────────────────────────

    /// With `strict_overflow: false`, E-LAY-001 is a Warning.
    #[test]
    fn test_overflow_is_warning_default() {
        let deck = make_deck(vec![make_slide_with_bullets(20)]);
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
        let deck = make_deck(vec![make_slide_with_bullets(20)]);
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
    #[test]
    fn test_overflow_multiple_slides() {
        let deck = make_deck(vec![
            make_slide_with_bullets(20),
            make_slide_with_bullets(15),
            make_slide_with_bullets(12),
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
            make_slide_with_bullets(20), // overflows
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

    /// E-LAY-001 message must contain "EMU" (confirms EMU-based estimation).
    #[test]
    fn test_overflow_message_has_emu_estimate() {
        let deck = make_deck(vec![make_slide_with_bullets(20)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("EMU"),
            "E-LAY-001 message must reference EMU units; got: {}",
            diags[0].message
        );
    }

    /// E-LAY-001 must include a correction hint.
    #[test]
    fn test_overflow_has_hint() {
        let deck = make_deck(vec![make_slide_with_bullets(20)]);
        let diags = validator_default().validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].hint.is_some(),
            "E-LAY-001 must include a correction hint"
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
}
