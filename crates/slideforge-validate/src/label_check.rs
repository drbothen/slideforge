//! Color-coded label enforcement validator (STORY-017).
//!
//! [`LabelCheckValidator`] checks every slide whose `slide_type` is one of the
//! [`COLOR_CODED_TYPES`] for a valid `label "..."` field. Slide types that use
//! color to convey meaning (WCAG 1.4.1 — Use of Color) MUST co-encode that
//! meaning in text via a non-empty, non-whitespace `label` field.
//!
//! Optionally checks declared foreground/background hex colors for WCAG AA
//! contrast ratio compliance and emits an advisory `E-A11-004` warning when
//! contrast is insufficient.
//!
//! ## Error codes
//!
//! | Code | Severity | Meaning |
//! |------|----------|---------|
//! | `E-A11-002` | Error | Color-coded element missing label or label is blank |
//! | `E-A11-004` | Warning | Color pair fails WCAG AA contrast threshold |

use std::sync::Arc;

use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
use slideforge_types::{Deck, FieldValue, SourceSpan, Value};

use crate::utils::is_blank;
use crate::wcag::{parse_hex_color, wcag_aa_passes};

/// Error code for a color-coded element with missing or blank label.
///
/// Traces to BC-5.01.003.
pub(crate) const E_A11_002: &str = "E-A11-002";

/// Warning code emitted when a color pair fails WCAG AA contrast.
///
/// This is advisory only — it does not block export. Traces to BC-5.01.003
/// (WCAG 1.4.3 Contrast Minimum).
pub(crate) const E_A11_004: &str = "E-A11-004";

/// Slide types that use color to convey meaning and therefore require a
/// non-empty `label "..."` field (WCAG 1.4.1: Use of Color).
///
/// When adding a new slide type that uses color semantically, add it here.
pub const COLOR_CODED_TYPES: &[&str] = &[
    "severity_cards",
    "status",
    "progress_bar",
    "weighted_composite",
];

/// Validates that all color-coded slides have a non-empty `label` field.
///
/// Checks each slide against [`COLOR_CODED_TYPES`]. For matching slides:
/// 1. Verifies the `label` field is present, non-empty, and non-whitespace.
///    Missing or blank → `E-A11-002` (blocking error).
/// 2. If hex `fg_color` and `bg_color` fields are declared, checks WCAG AA
///    contrast. Insufficient contrast → `E-A11-004` (advisory warning).
///    Note: brand palette references are skipped (brand not loaded in Wave 2).
///
/// `decorative: true` does NOT exempt color-coded elements from the label
/// requirement (BC-5.01.003 invariant 1, AC-004).
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_validator`].
pub struct LabelCheckValidator;

impl Validator for LabelCheckValidator {
    fn id(&self) -> &'static str {
        "label-check"
    }

    fn validate(&self, deck: &Deck, opts: &ValidatorOptions) -> Vec<Diagnostic> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for slide in &deck.slides {
            // Only check slides whose type is in the color-coded register.
            if !COLOR_CODED_TYPES.contains(&slide.slide_type.as_ref()) {
                continue;
            }

            // Check label field — None or blank is a blocking E-A11-002.
            // decorative: true does NOT exempt from the label requirement (AC-004).
            let label_valid = match slide.fields.get("label") {
                Some(FieldValue::Literal(Value::Str(s))) => !is_blank(s.as_ref()),
                _ => false,
            };

            if !label_valid {
                diagnostics.push(make_missing_label_error(
                    slide.slide_type.as_ref(),
                    &slide.source_span,
                ));
            }

            // Check WCAG AA contrast if both hex fg/bg colors are declared.
            // Brand palette references (non-hex strings) are skipped in Wave 2.
            if !opts.skip_contrast_check {
                let fg_hex = get_str_field(&slide.fields, "fg_color");
                let bg_hex = get_str_field(&slide.fields, "bg_color");

                if let (Some(fg_str), Some(bg_str)) = (fg_hex, bg_hex)
                    && let (Some(fg), Some(bg)) =
                        (parse_hex_color(fg_str), parse_hex_color(bg_str))
                {
                    // Use wcag_aa_passes with large_text=false: font-size is not
                    // available in Wave 2, so we always apply the 4.5:1 normal-text
                    // threshold. This eliminates threshold duplication with wcag.rs.
                    if !wcag_aa_passes(fg, bg, false) {
                        // Compute ratio for the warning message. The ratio must be
                        // computed here only for display; the pass/fail decision
                        // is delegated entirely to wcag_aa_passes above.
                        let ratio = {
                            use crate::wcag::{contrast_ratio, relative_luminance};
                            contrast_ratio(
                                relative_luminance(fg.0, fg.1, fg.2),
                                relative_luminance(bg.0, bg.1, bg.2),
                            )
                        };
                        diagnostics.push(make_low_contrast_warning(
                            slide.slide_type.as_ref(),
                            ratio,
                            &slide.source_span,
                        ));
                    }
                }
            }
        }

        diagnostics
    }
}

/// Construct an `E-A11-002` error diagnostic for a missing or blank label.
fn make_missing_label_error(slide_type: &str, span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: Arc::from(E_A11_002),
        message: Arc::from(format!(
            "Missing label on color-coded element '{slide_type}' at {span}. \
             Color alone must not convey meaning. Add label \"...\"."
        )),
        span: span.clone(),
        hint: Some(Arc::from(
            "Color-coded slide types require label \"...\" (WCAG 1.4.1: Use of Color)",
        )),
    }
}

/// Construct an `E-A11-004` warning diagnostic for insufficient WCAG contrast.
fn make_low_contrast_warning(slide_type: &str, ratio: f64, span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Warning,
        code: Arc::from(E_A11_004),
        message: Arc::from(format!(
            "Low WCAG contrast ratio {ratio:.2}:1 on '{slide_type}' (threshold 4.5:1 normal text). \
             Increase contrast between fg_color and bg_color."
        )),
        span: span.clone(),
        hint: Some(Arc::from(
            "Use a color pair with contrast ratio ≥ 4.5:1 for normal text (WCAG 1.4.3)",
        )),
    }
}

/// Extract a string field value from `fields`, returning `None` if absent or not a plain string.
fn get_str_field<'a>(
    fields: &'a slideforge_types::OrderedMap<Arc<str>, FieldValue>,
    key: &str,
) -> Option<&'a str> {
    match fields.get(key) {
        Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
        _ => None,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)] // BC-traceability IDs use uppercase: test_BC_S_SS_NNN_xxx
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DiagnosticSeverity, Validator, ValidatorOptions};
    use slideforge_types::{Deck, DeckMetadata, FieldValue, OrderedMap, Slide, SourceSpan, Value};

    use super::{COLOR_CODED_TYPES, E_A11_002, E_A11_004, LabelCheckValidator};

    // ── Deck/slide construction helpers ───────────────────────────────────────

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

    /// Build a slide of `slide_type` with optional label and color fields.
    fn make_color_coded_slide(
        slide_type: &str,
        label: Option<&str>,
        fg_color: Option<&str>,
        bg_color: Option<&str>,
        decorative: bool,
    ) -> Slide {
        let mut fields: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();

        if let Some(lbl) = label {
            fields.insert(
                Arc::from("label"),
                FieldValue::Literal(Value::Str(Arc::from(lbl))),
            );
        }
        if let Some(fg) = fg_color {
            fields.insert(
                Arc::from("fg_color"),
                FieldValue::Literal(Value::Str(Arc::from(fg))),
            );
        }
        if let Some(bg) = bg_color {
            fields.insert(
                Arc::from("bg_color"),
                FieldValue::Literal(Value::Str(Arc::from(bg))),
            );
        }
        if decorative {
            fields.insert(
                Arc::from("decorative"),
                FieldValue::Literal(Value::Bool(true)),
            );
        }

        Slide {
            slide_type: Arc::from(slide_type),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
        }
    }

    fn default_opts() -> ValidatorOptions {
        ValidatorOptions::default()
    }

    // ── Missing label — blocking errors ───────────────────────────────────────

    /// BC-5.01.003 AC-001: `severity_cards` with no `label` field → E-A11-002.
    #[test]
    fn test_BC_5_01_003_severity_cards_missing_label() {
        let slide = make_color_coded_slide("severity_cards", None, None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "severity_cards missing label must produce exactly 1 E-A11-002; got {diags:?}"
        );
        assert_eq!(
            diags[0].code.as_ref(),
            E_A11_002,
            "diagnostic code must be E-A11-002; got {}",
            diags[0].code
        );
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Error,
            "E-A11-002 must be Error severity"
        );
    }

    /// BC-5.01.003 AC-002: `status` with no `label` field → E-A11-002.
    #[test]
    fn test_BC_5_01_003_status_missing_label() {
        let slide = make_color_coded_slide("status", None, None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "status missing label must produce 1 E-A11-002; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_002);
    }

    /// BC-5.01.003 AC-002: `progress_bar` with no `label` field → E-A11-002.
    #[test]
    fn test_BC_5_01_003_progress_bar_missing_label() {
        let slide = make_color_coded_slide("progress_bar", None, None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "progress_bar missing label must produce 1 E-A11-002; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_002);
    }

    /// BC-5.01.003 AC-002: `weighted_composite` with no `label` field → E-A11-002.
    #[test]
    fn test_BC_5_01_003_weighted_composite_missing_label() {
        let slide = make_color_coded_slide("weighted_composite", None, None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "weighted_composite missing label must produce 1 E-A11-002; got {diags:?}"
        );
        assert_eq!(diags[0].code.as_ref(), E_A11_002);
    }

    /// BC-5.01.003: `severity_cards` with a valid label → 0 diagnostics.
    #[test]
    fn test_BC_5_01_003_label_present_no_error() {
        let slide = make_color_coded_slide(
            "severity_cards",
            Some("HIGH: action required"),
            None,
            None,
            false,
        );
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        assert!(
            diags.iter().all(|d| d.code.as_ref() != E_A11_002),
            "valid label must produce no E-A11-002; got {diags:?}"
        );
    }

    /// BC-5.01.003 AC-003 EC-001: `label ""` (empty string) → E-A11-002.
    #[test]
    fn test_BC_5_01_003_label_empty_string() {
        let slide = make_color_coded_slide("severity_cards", Some(""), None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert_eq!(
            label_errors.len(),
            1,
            "empty string label must produce 1 E-A11-002; got {diags:?}"
        );
    }

    /// BC-5.01.003 EC-002: `label "  "` (whitespace-only) → E-A11-002.
    #[test]
    fn test_BC_5_01_003_label_whitespace() {
        let slide = make_color_coded_slide("severity_cards", Some("  "), None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert_eq!(
            label_errors.len(),
            1,
            "whitespace-only label must produce 1 E-A11-002; got {diags:?}"
        );
    }

    /// BC-5.01.003 AC-004 EC-005: `decorative: true` does NOT exempt a
    /// color-coded element from the label requirement.
    #[test]
    fn test_BC_5_01_003_decorative_does_not_exempt_label() {
        // decorative: true but no label → still E-A11-002
        let slide = make_color_coded_slide("severity_cards", None, None, None, true);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert_eq!(
            label_errors.len(),
            1,
            "decorative: true must NOT exempt a color-coded element from label check; got {diags:?}"
        );
    }

    /// BC-5.01.003 AC-005 EC-004: 3 `status` slides, 2 missing `label` → 2× E-A11-002.
    ///
    /// ## IR model note (AC-005 wording vs. test structure)
    ///
    /// The story spec states "on one slide" to describe the scenario where multiple
    /// color-coded elements are present simultaneously. In the slideforge IR model,
    /// however, each color-coded element IS a separate `Slide` — there is no
    /// sub-slide "element" concept at the validation layer. Accordingly, this test
    /// correctly creates 3 separate `Slide` instances (one per color-coded status
    /// element) rather than placing them inside a single slide. The behavioral
    /// intent of AC-005 — that 2-of-3 failing elements each produce their own
    /// E-A11-002 — is preserved exactly; only the IR framing differs from the
    /// natural-language spec wording.
    #[test]
    fn test_BC_5_01_003_multiple_missing_labels() {
        let slide_ok = make_color_coded_slide("status", Some("Green: on track"), None, None, false);
        let slide_missing1 = make_color_coded_slide("status", None, None, None, false);
        let slide_missing2 = make_color_coded_slide("status", None, None, None, false);
        let deck = make_deck(vec![slide_ok, slide_missing1, slide_missing2]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert_eq!(
            label_errors.len(),
            2,
            "2 of 3 status slides missing label → exactly 2× E-A11-002; got {diags:?}"
        );
    }

    // ── WCAG contrast checks — advisory warnings ───────────────────────────────

    /// BC-5.01.003 AC-007: `severity_cards` with fg/bg producing < 4.5:1 ratio →
    /// E-A11-004 warning (not blocking).
    ///
    /// Using red (#FF0000) on white (#FFFFFF): contrast ≈ 4.0:1 < 4.5.
    #[test]
    fn test_BC_5_01_003_low_contrast_warning() {
        let slide = make_color_coded_slide(
            "severity_cards",
            Some("HIGH"),
            Some("#FF0000"), // fg: red — ≈4.0:1 on white
            Some("#FFFFFF"), // bg: white
            false,
        );
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let contrast_warnings: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_004)
            .collect();
        assert_eq!(
            contrast_warnings.len(),
            1,
            "low contrast fg/bg must produce 1 E-A11-004 warning; got {diags:?}"
        );
        assert_eq!(
            contrast_warnings[0].severity,
            DiagnosticSeverity::Warning,
            "E-A11-004 must be Warning severity (not blocking)"
        );
    }

    /// BC-5.01.003: fg/bg with adequate contrast (≥ 4.5:1) produces no E-A11-004.
    ///
    /// Using dark red (#CC0000) on white (#FFFFFF): contrast ≈ 5.9:1 ≥ 4.5.
    #[test]
    fn test_BC_5_01_003_adequate_contrast_no_warning() {
        let slide = make_color_coded_slide(
            "severity_cards",
            Some("HIGH"),
            Some("#CC0000"), // fg: dark red — ≈5.9:1 on white
            Some("#FFFFFF"), // bg: white
            false,
        );
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let contrast_warnings: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_004)
            .collect();
        assert!(
            contrast_warnings.is_empty(),
            "adequate contrast must produce no E-A11-004; got {diags:?}"
        );
    }

    // ── Non-color-coded slides are not checked ─────────────────────────────────

    /// BC-5.01.003: Slides of non-color-coded types (e.g., "content", "title")
    /// are not checked for label — no diagnostics.
    #[test]
    fn test_BC_5_01_003_non_color_coded_slide_no_check() {
        // "content" is not in COLOR_CODED_TYPES — no E-A11-002
        let slide = Slide {
            slide_type: Arc::from("content"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
        };
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "non-color-coded slide must produce no diagnostics; got {diags:?}"
        );
    }

    /// BC-5.01.003: Empty deck produces no diagnostics.
    #[test]
    fn test_BC_5_01_003_empty_deck_no_diagnostics() {
        let deck = make_deck(vec![]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        assert!(diags.is_empty(), "empty deck must produce no diagnostics");
    }

    /// BC-5.01.003: `COLOR_CODED_TYPES` contains exactly the 4 required types.
    ///
    /// Verifies the const matches the spec register (BC-5.01.003 architecture rule).
    #[test]
    fn test_BC_5_01_003_color_coded_types_const() {
        assert!(COLOR_CODED_TYPES.contains(&"severity_cards"));
        assert!(COLOR_CODED_TYPES.contains(&"status"));
        assert!(COLOR_CODED_TYPES.contains(&"progress_bar"));
        assert!(COLOR_CODED_TYPES.contains(&"weighted_composite"));
        assert_eq!(
            COLOR_CODED_TYPES.len(),
            4,
            "COLOR_CODED_TYPES must contain exactly 4 types"
        );
    }

    /// BC-5.01.003: Validator ID is "label-check".
    #[test]
    fn test_BC_5_01_003_validator_id() {
        assert_eq!(LabelCheckValidator.id(), "label-check");
    }

    /// BC-5.01.003 EC-009: Low contrast AND missing label on same slide →
    /// both E-A11-002 (error) and E-A11-004 (warning) are emitted.
    #[test]
    fn test_BC_5_01_003_low_contrast_and_missing_label() {
        // No label + low contrast fg/bg → both codes
        let slide = make_color_coded_slide(
            "severity_cards",
            None,            // no label → E-A11-002
            Some("#FF0000"), // low contrast → E-A11-004
            Some("#FFFFFF"),
            false,
        );
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        let contrast_warnings: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_004)
            .collect();
        assert_eq!(
            label_errors.len(),
            1,
            "missing label must still produce E-A11-002 even when contrast also fails; got {diags:?}"
        );
        assert_eq!(
            contrast_warnings.len(),
            1,
            "low contrast must still produce E-A11-004 even when label is also missing; got {diags:?}"
        );
    }
}
