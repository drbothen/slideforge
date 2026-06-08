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
                // BC-1.17.001/002/003 PC2: message must include the slide's title value
                // so the user can identify which specific slide is missing its label.
                let title = match slide.fields.get("title") {
                    Some(FieldValue::Literal(Value::Str(s))) => s.as_ref(),
                    _ => "",
                };
                diagnostics.push(make_missing_label_error(
                    slide.slide_type.as_ref(),
                    title,
                    &slide.source_span,
                ));
            }

            // BC-1.17.003 postconditions 4+5 / AC-018 / AC-022 / DI-018:
            // For `weighted_composite` slides, also check per-component labels.
            // Iterate Slide.fields["components"] (a Value::List of Value::Maps).
            // Emit E-A11-002 per component missing its label (accumulate all).
            if slide.slide_type.as_ref() == "weighted_composite"
                && let Some(FieldValue::Literal(Value::List(components))) =
                    slide.fields.get("components")
            {
                for comp_val in components {
                    if let Value::Map(comp_map) = comp_val {
                        let comp_label_ok = match comp_map.get("label") {
                            Some(Value::Str(s)) => !is_blank(s.as_ref()),
                            _ => false,
                        };
                        if !comp_label_ok {
                            let comp_name = match comp_map.get("name") {
                                Some(Value::Str(s)) => s.as_ref().to_owned(),
                                _ => String::new(),
                            };
                            diagnostics.push(make_missing_component_label_error(
                                &comp_name,
                                &slide.source_span,
                            ));
                        }
                    }
                }
            }

            // Check WCAG AA contrast if both hex fg/bg colors are declared.
            // Brand palette references (non-hex strings) are skipped in Wave 2.
            if !opts.skip_contrast_check {
                let fg_hex = get_str_field(&slide.fields, "fg_color");
                let bg_hex = get_str_field(&slide.fields, "bg_color");

                if let (Some(fg_str), Some(bg_str)) = (fg_hex, bg_hex)
                    && let (Some(fg), Some(bg)) = (parse_hex_color(fg_str), parse_hex_color(bg_str))
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
///
/// # BC-1.17.001/002/003 PC2 — message format
///
/// The BC postcondition specifies the exact message format:
/// `Missing label on color-coded element '<slide_type>' '<title-value>' at <file>:<line>:<col>.
///  Color alone must not convey meaning. Add label "...".`
///
/// The `title` parameter carries the slide's `fields["title"]` string value.
/// Pass an empty string `""` when no title is available (synthetic test slides
/// constructed without a title field); the message will then read `''` as the
/// title token, which is correct for that case.
fn make_missing_label_error(slide_type: &str, title: &str, span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: Arc::from(E_A11_002),
        message: Arc::from(format!(
            "Missing label on color-coded element '{slide_type}' '{title}' at {span}. \
             Color alone must not convey meaning. Add label \"...\"."
        )),
        span: span.clone(),
        hint: Some(Arc::from(
            "Color-coded slide types require label \"...\" (WCAG 1.4.1: Use of Color)",
        )),
    }
}

/// Construct an `E-A11-002` error diagnostic for a missing or blank per-component label.
///
/// BC-1.17.003 postcondition 4: per-component label is mandatory; missing label
/// on a component emits E-A11-002 identifying the component by name.
fn make_missing_component_label_error(comp_name: &str, span: &SourceSpan) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: Arc::from(E_A11_002),
        message: Arc::from(format!(
            "Missing label on color-coded element 'weighted_composite.component' \
             '{comp_name}' at {span}. \
             Add label \"...\" to this component."
        )),
        span: span.clone(),
        hint: Some(Arc::from(
            "Each component in weighted_composite requires label \"...\" (WCAG 1.4.1: Use of Color)",
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
            overlay: None,
            register_content: vec![],
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
            overlay: None,
            register_content: vec![],
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

    /// BC-5.01.003: `ValidatorOptions::skip_contrast_check = true` suppresses
    /// E-A11-004 warnings even when fg/bg produce < 4.5:1 contrast.
    ///
    /// Same low-contrast setup as `test_BC_5_01_003_low_contrast_warning` (red on
    /// white, ≈4.0:1), but with `skip_contrast_check: true`. The label is present
    /// so E-A11-002 is also not emitted; the result must be zero diagnostics.
    #[test]
    fn test_BC_5_01_003_skip_contrast_check_suppresses_warning() {
        let slide = make_color_coded_slide(
            "severity_cards",
            Some("HIGH"),    // valid label → no E-A11-002
            Some("#FF0000"), // fg: red — ≈4.0:1 on white, would trigger E-A11-004
            Some("#FFFFFF"), // bg: white
            false,
        );
        let deck = make_deck(vec![slide]);
        let opts = ValidatorOptions {
            skip_contrast_check: true,
            ..ValidatorOptions::default()
        };
        let diags = LabelCheckValidator.validate(&deck, &opts);
        let contrast_warnings: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_004)
            .collect();
        assert!(
            contrast_warnings.is_empty(),
            "skip_contrast_check: true must suppress all E-A11-004 warnings; got {diags:?}"
        );
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

    // ── STORY-087 Red Gate tests ──────────────────────────────────────────────
    //
    // Tests for BC-1.17.001 (AC-006), BC-1.17.003 (AC-018, AC-022), and
    // F-G3-HIGH-003 (dead-guard regression + AC-024 consistency invariant).
    //
    // ALL tests in this block MUST FAIL before implementation begins.
    // - AC-018 fails because component iteration is NOT yet in validate().
    // - AC-022 fails for the same reason.
    // - AC-006 should PASS (status top-level label check already works).
    // - AC-024 should PASS (keywords.rs already has severity_cards from Step 1 stubs).
    // - F-G3-HIGH-003 regression test should PASS (severity_cards already works).
    // ─────────────────────────────────────────────────────────────────────────

    /// Helper: build a `weighted_composite` slide with resolved `components` in fields.
    fn make_weighted_composite_with_components(
        top_label: Option<&str>,
        components: Vec<OrderedMap<Arc<str>, FieldValue>>,
    ) -> Slide {
        use slideforge_types::Value;
        let mut fields: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Vendor A"))),
        );
        if let Some(lbl) = top_label {
            fields.insert(
                Arc::from("label"),
                FieldValue::Literal(Value::Str(Arc::from(lbl))),
            );
        }
        // components: List of Maps (each Map uses Value::* not FieldValue::*)
        let component_list: Vec<Value> = components
            .into_iter()
            .map(|comp_fv| {
                // Convert FieldValue::Literal(Value) map to Value::Map(OrderedMap<Arc<str>, Value>)
                let mut m: OrderedMap<Arc<str>, Value> = OrderedMap::new();
                for (k, fv) in &comp_fv {
                    if let FieldValue::Literal(v) = fv {
                        m.insert(Arc::clone(k), v.clone());
                    }
                }
                Value::Map(m)
            })
            .collect();
        fields.insert(
            Arc::from("components"),
            FieldValue::Literal(Value::List(component_list)),
        );
        Slide {
            slide_type: Arc::from("weighted_composite"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    /// Helper: build a component as `OrderedMap<Arc<str>, FieldValue>`.
    fn make_component_fv(name: &str, label: Option<&str>) -> OrderedMap<Arc<str>, FieldValue> {
        use slideforge_types::Value;
        let mut m: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();
        m.insert(
            Arc::from("name"),
            FieldValue::Literal(Value::Str(Arc::from(name))),
        );
        if let Some(lbl) = label {
            m.insert(
                Arc::from("label"),
                FieldValue::Literal(Value::Str(Arc::from(lbl))),
            );
        }
        m
    }

    // ── BC-1.17.001 AC-006 / invariant 3 ─────────────────────────────────────

    /// BC-1.17.001 AC-006 / invariant 3: `LabelCheck` fires E-A11-002 for `status`
    /// even when `Slide.blocks = vec![]` (pre-Stage-2b). Verifies `LabelCheck` reads
    /// `Slide.fields["label"]`, NOT `Slide.blocks`.
    ///
    /// This test exercises the Stage 2b independence guarantee. It should PASS at
    /// Red Gate because the basic status label check is already implemented.
    #[test]
    fn test_BC_1_17_001_ac006_status_label_check_reads_fields_not_blocks() {
        // No label AND no blocks — Stage 2b has NOT run
        let slide = Slide {
            slide_type: Arc::from("status"),
            fields: OrderedMap::new(), // no label
            blocks: vec![],            // Stage 2b has NOT populated blocks
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());

        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert_eq!(
            label_errors.len(),
            1,
            "BC-1.17.001 AC-006: LabelCheck must fire E-A11-002 for 'status' with \
             empty fields even when blocks=vec![] (Stage 2b independence); got {diags:?}"
        );
    }

    // ── BC-1.17.003 AC-018 / DI-018 accumulation ─────────────────────────────

    /// BC-1.17.003 AC-018 / invariant 5 / DI-018: When top-level label is absent
    /// AND both components are missing labels, `LabelCheckValidator` must accumulate
    /// exactly 3 E-A11-002 diagnostics (1 top-level + 2 component).
    ///
    /// FAILS at Red Gate: component iteration is NOT yet implemented in `validate()`.
    /// Current behavior: emits only 1 E-A11-002 (top-level). Expected: 3.
    #[test]
    fn test_BC_1_17_003_ac018_weighted_composite_accumulates_3_label_errors() {
        let comp1 = make_component_fv("Quality", None);
        let comp2 = make_component_fv("Price", None);
        // No top-level label AND no component labels → should produce 3 E-A11-002
        let slide = make_weighted_composite_with_components(None, vec![comp1, comp2]);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());

        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert_eq!(
            label_errors.len(),
            3,
            "BC-1.17.003 AC-018 / DI-018: top-label absent + 2 components missing labels \
             must produce exactly 3 E-A11-002 diagnostics (1 top + 2 component); \
             got {} E-A11-002 diagnostics: {diags:?}",
            label_errors.len()
        );
    }

    // ── BC-1.17.003 AC-022 / invariant 8 ─────────────────────────────────────

    /// BC-1.17.003 AC-022 / invariant 8: `LabelCheckValidator` iterates
    /// `Slide.fields["components"]` to check per-component labels.
    /// Top-level label is PRESENT; component 2 is missing its label.
    /// `Slide.blocks = vec![]` (pre-Stage-2b).
    ///
    /// FAILS at Red Gate: component iteration is NOT yet in `validate()`.
    /// Current behavior: 0 E-A11-002 (top-label present, no component iteration).
    /// Expected: 1 E-A11-002 for the missing component label.
    #[test]
    fn test_BC_1_17_003_ac022_weighted_composite_label_check_component_iteration() {
        let comp_ok = make_component_fv("Quality", Some("Excellent"));
        let comp_no_label = make_component_fv("Price", None); // missing label
        // Top-level label IS present; only one component is missing its label
        let slide = make_weighted_composite_with_components(
            Some("Overall: Good"),
            vec![comp_ok, comp_no_label],
        );
        assert!(
            slide.blocks.is_empty(),
            "test precondition: blocks must be empty (Stage 2b independence)"
        );
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());

        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert_eq!(
            label_errors.len(),
            1,
            "BC-1.17.003 AC-022: LabelCheck must fire exactly 1 E-A11-002 for the \
             component 'Price' missing its label (top-level label is present); \
             got {} diagnostics: {diags:?}",
            label_errors.len()
        );
        // The error message must identify the component
        assert!(
            label_errors[0].message.contains("Price")
                || label_errors[0].message.contains("component")
                || label_errors[0].message.contains("weighted_composite"),
            "E-A11-002 message must identify the failing component; got: {}",
            label_errors[0].message
        );
    }

    // ── F-G3-HIGH-003 regression ──────────────────────────────────────────────

    /// F-G3-HIGH-003 regression: `severity_cards` `LabelCheck` must STILL fire
    /// E-A11-002 for a missing label after STORY-087 changes. No regression
    /// from `COLOR_CODED_TYPES` modifications.
    #[test]
    fn test_F_G3_HIGH_003_severity_cards_label_check_regression() {
        let slide = make_color_coded_slide("severity_cards", None, None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());

        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert_eq!(
            label_errors.len(),
            1,
            "F-G3-HIGH-003: severity_cards must STILL produce E-A11-002 for missing \
             label after STORY-087 (no regression); got {diags:?}"
        );
    }

    // ── P05-LOW-001: per-component E-A11-002 message matches BC-1.17.003 PC4 ──
    //
    // BC-1.17.003 PC4 mandates the EXACT per-component E-A11-002 message format:
    //   `Missing label on color-coded element 'weighted_composite.component'
    //    '<component-name>' at <file>:<line>:<col>. Add label "..." to this component.`
    //
    // The current impl in `make_missing_component_label_error` emits:
    //   `"Missing label on color-coded element 'weighted_composite.component'
    //    '{comp_name}' at {span}.
    //    Color alone must not convey meaning. Add label \"...\" to this component."`
    //
    // The EXTRA SENTENCE "Color alone must not convey meaning." is NOT in PC4.
    // PC4 specifies only: "Add label \"...\" to this component." (no intermediate sentence).
    // This test asserts the message does NOT contain that extra sentence,
    // and DOES contain the component name, the type token, and "Add label".
    //
    // RED GATE (P05-LOW-001): the current impl inserts the extra sentence from
    // the top-level error template. The `!msg.contains("Color alone...")` assertion
    // FAILS. The implementer must remove that sentence from the component message.

    /// BC-1.17.003 PC4 / P05-LOW-001:
    /// `LabelCheckValidator` per-component E-A11-002 message for a missing component
    /// label MUST match BC-1.17.003 PC4 exactly:
    ///
    /// `Missing label on color-coded element 'weighted_composite.component' '<component-name>'
    ///  at <file>:<line>:<col>. Add label "..." to this component.`
    ///
    /// Required tokens (all must be present):
    /// - `"weighted_composite.component"` — the element type path
    /// - `"Quality"` — the component name (token `'<component-name>'` from PC4)
    /// - `"Add label"` — the remediation instruction
    ///
    /// Forbidden token (PC4 does NOT include this sentence):
    /// - `"Color alone must not convey meaning."` — this sentence appears in the
    ///   TOP-LEVEL error template (PC2) but NOT in the per-component template (PC4).
    ///   The current impl mistakenly inserts it, violating BC-1.17.003 PC4.
    ///
    /// ## RED GATE (P05-LOW-001)
    ///
    /// The current `make_missing_component_label_error` generates:
    ///   `"...'{comp_name}' at {span}. Color alone must not convey meaning.
    ///    Add label \"...\" to this component."`
    ///
    /// The forbidden sentence is present → `!msg.contains(...)` assertion FAILS.
    ///
    /// ## Post-implementation
    ///
    /// After removing the extra sentence from `make_missing_component_label_error`,
    /// the message matches PC4 exactly and all assertions pass.
    ///
    /// Traceability: BC-1.17.003 PC4; ADV-STORY087-P05-LOW-001.
    #[test]
    fn test_BC_1_17_003_pc4_component_label_error_message_format() {
        let comp_no_label = make_component_fv("Quality", None);
        let slide =
            make_weighted_composite_with_components(Some("Overall: Good"), vec![comp_no_label]);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());

        let component_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.code.as_ref() == E_A11_002 && d.message.contains("weighted_composite.component")
            })
            .collect();

        assert_eq!(
            component_errors.len(),
            1,
            "BC-1.17.003 PC4: must produce exactly 1 E-A11-002 for component 'Quality' \
             missing its label (top-level label is present); got: {diags:?}"
        );

        let msg = component_errors[0].message.as_ref();

        // Required: the type path token.
        assert!(
            msg.contains("weighted_composite.component"),
            "BC-1.17.003 PC4: message must contain 'weighted_composite.component'; \
             got: {msg:?}"
        );

        // Required: the component name (canonical 'Quality').
        assert!(
            msg.contains("Quality"),
            "BC-1.17.003 PC4: message must contain the component name 'Quality'; \
             got: {msg:?}"
        );

        // Required: the remediation instruction from PC4.
        assert!(
            msg.contains("Add label"),
            "BC-1.17.003 PC4: message must contain 'Add label' (remediation instruction); \
             got: {msg:?}"
        );

        // RED GATE (P05-LOW-001): PC4 does NOT include "Color alone must not convey meaning."
        // The top-level message (PC2) uses that phrase; the component message (PC4) must NOT.
        // Current impl inserts it from the top-level template → this assertion FAILS.
        assert!(
            !msg.contains("Color alone must not convey meaning."),
            "P05-LOW-001 RED GATE: per-component E-A11-002 message must NOT contain \
             'Color alone must not convey meaning.' — that sentence belongs to the \
             top-level message format (BC-1.17.001/002 PC2) but NOT to the per-component \
             format (BC-1.17.003 PC4). \
             BC-1.17.003 PC4 format: \
             \"Missing label on color-coded element 'weighted_composite.component' \
             '<component-name>' at <file>:<line>:<col>. Add label \\\"...\\\" to this component.\" \
             Current impl inserts the extra sentence. \
             Got message: {msg:?}"
        );
    }

    // ── AC-024 / F-G3-HIGH-003 dead-guard consistency invariant ──────────────

    /// BC-1.17.001 postcondition 5 / AC-024: Every entry in `COLOR_CODED_TYPES`
    /// must appear in `SLIDE_TYPE_KEYWORDS`.
    ///
    /// Before STORY-087, `"severity_cards"` was in `COLOR_CODED_TYPES` but ABSENT
    /// from `SLIDE_TYPE_KEYWORDS` (D4 gap), making `LabelCheck` a dead letter for it.
    /// After STORY-087, all 4 entries must be in both sets.
    ///
    /// This test directly closes F-G3-HIGH-003.
    #[test]
    fn test_BC_1_17_001_ac024_all_color_coded_types_in_slide_type_keywords() {
        use slideforge_syntax::keywords::is_slide_type_keyword;

        for color_type in COLOR_CODED_TYPES {
            assert!(
                is_slide_type_keyword(color_type),
                "F-G3-HIGH-003 / AC-024: COLOR_CODED_TYPES entry '{color_type}' must be present in \
                 SLIDE_TYPE_KEYWORDS — otherwise LabelCheck is a dead letter for this type",
            );
        }
        // Explicit membership assertions
        assert!(
            COLOR_CODED_TYPES.contains(&"status"),
            "must contain 'status'"
        );
        assert!(
            COLOR_CODED_TYPES.contains(&"progress_bar"),
            "must contain 'progress_bar'"
        );
        assert!(
            COLOR_CODED_TYPES.contains(&"weighted_composite"),
            "must contain 'weighted_composite'"
        );
        assert!(
            COLOR_CODED_TYPES.contains(&"severity_cards"),
            "must contain 'severity_cards' (D4 gap fix)"
        );
    }

    // ── NFR-021/022/023 positive paths ───────────────────────────────────────

    /// NFR-021: `status` with valid label → `LabelCheck` emits no E-A11-002.
    #[test]
    fn test_BC_1_17_001_nfr021_status_with_valid_label_passes_label_check() {
        let slide = make_color_coded_slide("status", Some("On Track"), None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert!(
            label_errors.is_empty(),
            "status with valid label must produce no E-A11-002; got {diags:?}"
        );
    }

    /// NFR-022: `progress_bar` with valid label → no E-A11-002.
    #[test]
    fn test_BC_1_17_002_nfr022_progress_bar_with_valid_label_passes_label_check() {
        let slide = make_color_coded_slide("progress_bar", Some("75% complete"), None, None, false);
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert!(
            label_errors.is_empty(),
            "progress_bar with valid label must produce no E-A11-002; got {diags:?}"
        );
    }

    /// NFR-023: `weighted_composite` with valid top-level label → no top-level E-A11-002.
    ///
    /// (Component label checking is the AC-022 behavior, tested separately above.)
    #[test]
    fn test_BC_1_17_003_nfr023_weighted_composite_top_label_present_no_top_error() {
        let slide = make_color_coded_slide(
            "weighted_composite",
            Some("Overall: Good"),
            None,
            None,
            false,
        );
        let deck = make_deck(vec![slide]);
        let diags = LabelCheckValidator.validate(&deck, &default_opts());
        let label_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .collect();
        assert!(
            label_errors.is_empty(),
            "weighted_composite with valid top-level label must produce no E-A11-002 \
             from top-level label check; got {diags:?}"
        );
    }
}
