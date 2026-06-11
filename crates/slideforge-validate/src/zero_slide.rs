//! Zero-slide validator (STORY-016).
//!
//! A [`Deck`] with no slides cannot be exported. [`ZeroSlideValidator`]
//! detects this condition and emits `E-LAY-002` so the pipeline can produce a
//! meaningful error message instead of a silent empty export.
//!
//! ## Error codes
//!
//! | Code | Severity | Meaning |
//! |------|----------|---------|
//! | `E-LAY-002` | Error | The deck contains zero slides |

use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
use slideforge_types::{Deck, SourceSpan};

/// Error code emitted when a deck contains zero slides.
pub(crate) const E_LAY_002: &str = "E-LAY-002";

/// Validates that a deck contains at least one slide.
///
/// A presentation with zero slides cannot be exported; this validator surfaces
/// the problem at the validation stage so the user receives a clear error
/// rather than a corrupt or empty output file.
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_validator`].
pub struct ZeroSlideValidator;

impl Validator for ZeroSlideValidator {
    fn id(&self) -> &'static str {
        "zero-slide"
    }

    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        if deck.slides.is_empty() {
            vec![make_zero_slide_error(&SourceSpan::default())]
        } else {
            vec![]
        }
    }
}

/// Construct an `E-LAY-002` diagnostic for an empty deck.
///
/// Note: the filename is not included in this message because the [`Validator`]
/// trait does not provide the source filename — only the [`Deck`] and its
/// [`SourceSpan`] are available at validation time.
fn make_zero_slide_error(span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: std::sync::Arc::from(E_LAY_002),
        message: std::sync::Arc::from(
            "Zero-slide deck: no slide blocks found. A deck must contain at least one slide.",
        ),
        span: span.clone(),
        hint: Some(std::sync::Arc::from(
            "Every exported deck must have at least one slide (E-LAY-002)",
        )),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DiagnosticSeverity, Validator, ValidatorOptions};
    use slideforge_types::{Deck, DeckMetadata, OrderedMap, Slide, SourceSpan};

    use super::{E_LAY_002, ZeroSlideValidator};

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

    fn make_slide() -> Slide {
        Slide {
            slide_type: Arc::from("content"),
            fields: OrderedMap::new(),
            blocks: vec![],
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

    // ── Validator ID ──────────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_03_016_zero_slide_validator_id() {
        assert_eq!(ZeroSlideValidator.id(), "zero-slide");
    }

    // ── Zero slides → 1 E-LAY-002 ─────────────────────────────────────────────

    /// BC-5.03.016 postcondition: empty deck emits exactly one E-LAY-002.
    #[test]
    fn test_zero_slide_deck() {
        let deck = make_deck(vec![]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "empty deck must emit exactly 1 E-LAY-002; got {diags:?}"
        );
        assert_eq!(
            diags[0].code.as_ref(),
            E_LAY_002,
            "diagnostic code must be E-LAY-002"
        );
    }

    /// BC-5.03.016 postcondition: E-LAY-002 has severity Error (not Warning).
    #[test]
    fn test_zero_slide_severity_is_error() {
        let deck = make_deck(vec![]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1, "expected 1 diagnostic; got {diags:?}");
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Error,
            "E-LAY-002 must be Error severity, not Warning"
        );
    }

    // ── Non-zero slides → 0 diagnostics ───────────────────────────────────────

    /// BC-5.03.016 postcondition: 1 slide produces 0 diagnostics.
    #[test]
    fn test_single_slide_no_error() {
        let deck = make_deck(vec![make_slide()]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "1 slide must produce no diagnostics; got {diags:?}"
        );
    }

    /// BC-5.03.016 postcondition: multiple slides produce 0 diagnostics.
    #[test]
    fn test_multiple_slides_no_error() {
        let deck = make_deck(vec![make_slide(), make_slide(), make_slide()]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "3 slides must produce no diagnostics; got {diags:?}"
        );
    }

    /// Boundary: exactly 2 slides produces 0 diagnostics (boundary above zero).
    #[test]
    fn test_two_slides_no_error() {
        let deck = make_deck(vec![make_slide(), make_slide()]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "2 slides must produce no diagnostics; got {diags:?}"
        );
    }

    // ── E-LAY-002 diagnostic quality ──────────────────────────────────────────

    /// E-LAY-002 must include a correction hint.
    #[test]
    fn test_zero_slide_has_hint() {
        let deck = make_deck(vec![]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].hint.is_some(),
            "E-LAY-002 must include a correction hint"
        );
    }

    /// E-LAY-002 message must be non-empty and meaningful.
    #[test]
    fn test_zero_slide_message_is_non_empty() {
        let deck = make_deck(vec![]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert!(
            !diags[0].message.is_empty(),
            "E-LAY-002 message must be non-empty"
        );
    }

    /// E-LAY-002 message must match the spec-prescribed prefix and phrasing.
    #[test]
    fn test_zero_slide_message_matches_spec() {
        let deck = make_deck(vec![]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        let msg = diags[0].message.as_ref();
        assert!(
            msg.starts_with("Zero-slide deck:"),
            "message should start with spec prefix, got: {msg}"
        );
        assert!(
            msg.contains("at least one slide"),
            "message should contain required phrasing, got: {msg}"
        );
    }

    // ── Error accumulation: only one E-LAY-002 per deck ──────────────────────

    /// Zero-slide condition is a deck-level invariant — only one diagnostic
    /// is emitted regardless (there's nothing to accumulate over).
    #[test]
    fn test_zero_slide_emits_exactly_one_diagnostic() {
        let deck = make_deck(vec![]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        // Must be exactly 1 — not 0 (missed), not 2+ (duplicated).
        assert_eq!(
            diags.len(),
            1,
            "must emit exactly 1 E-LAY-002, not more; got {diags:?}"
        );
    }

    // ── E-LAY-002 never demoted by warn-only (I06) ────────────────────────────

    /// E-LAY-002 is always Error severity — not demotable by `WarnOnly` mode.
    ///
    /// The demotion decision belongs to the CLI, not the validator. The validator
    /// always emits Error. If the CLI is in `WarnOnly` mode, it logs all diagnostics
    /// but does not abort — however that is CLI behaviour, not validator behaviour.
    /// The validator itself must always emit Error for E-LAY-002.
    #[test]
    fn test_zero_slide_warn_only_still_blocks() {
        // Even in warn-only mode, E-LAY-002 is always Error severity.
        // The validator emits Error regardless of the ValidatorOptions — the CLI
        // decides whether to abort or merely report.
        let deck = make_deck(vec![]);
        let diags = ZeroSlideValidator.validate(&deck, &default_opts());
        assert_eq!(diags.len(), 1);
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Error,
            "E-LAY-002 must be Error even in warn-only context — demotion is CLI responsibility"
        );
    }
}
