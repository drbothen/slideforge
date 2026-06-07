//! Field-schema validator — wires `validate_fields` into the Stage-5 pipeline.
//!
//! [`FieldSchemaValidator`] is the `Validator`-trait bridge that connects the
//! per-slide-type field schemas (declared in `slideforge-plugin-api`) to the
//! Stage-5 validation loop in `build_inner`. Without this validator, the rich
//! E-VAL-104 (type-mismatch), E-VAL-101 (required field absent), E-VAL-102
//! (required field empty), and W-VAL-103 (unknown field) diagnostics produced by
//! [`slideforge_plugin_api::validate_fields`] would never be emitted at build
//! time — `validate_fields` would be dead code.
//!
//! ## Architecture: Route B (STORY-089 / ADR-020 Decision 4)
//!
//! This implementation uses **Route B**: look up each slide's type in the
//! process-wide [`slideforge_plugin_api::SLIDE_TYPE_REGISTRY`] (the
//! `SlideTypeRegistry::default()` singleton). This covers all 34 built-in slide
//! types without requiring access to the `PluginRegistry` (which is not passed
//! to validators at Stage 5).
//!
//! Slides whose keyword is not found in the registry are silently skipped — the
//! `UnknownSlideType` layout error (emitted at Stage 6) already handles that case.
//!
//! // TODO(future story): Route A — pass `Arc<PluginRegistry>` to validators so
//! // that `FieldSchemaValidator` can also validate externally-registered custom
//! // slide types added by third-party plugins. Route B covers only built-in types.
//!
//! ## Error codes emitted
//!
//! | Code | Severity | Meaning |
//! |------|----------|---------|
//! | `E-VAL-101` | Error | Required field absent |
//! | `E-VAL-102` | Error | Required field is empty string |
//! | `W-VAL-103` | Warning | Unknown field (not in required ∪ optional) |
//! | `E-VAL-104` | Error | Field value type mismatch or `OneOf` violation |
//!
//! ## Traceability
//!
//! - BC-1.18.001 postcondition 10: E-VAL-104 reachable from `build()` (STORY-089 AC-009)
//! - ADR-020 Decision 4: `FieldSchemaValidator` as Stage-5 `Validator` plugin
//! - STORY-089 AC-009, AC-010

use slideforge_plugin_api::{
    Diagnostic, SLIDE_TYPE_REGISTRY, Validator, ValidatorOptions, validate_fields,
};
use slideforge_types::Deck;

/// Stage-5 pipeline bridge that calls [`validate_fields`] for every slide in the
/// deck.
///
/// For each slide, the validator:
/// 1. Resolves the slide type via the built-in [`SLIDE_TYPE_REGISTRY`] singleton.
/// 2. Calls [`validate_fields`] to accumulate E-VAL-101, E-VAL-102, W-VAL-103,
///    and E-VAL-104 diagnostics for the slide.
/// 3. Slides with an unrecognised keyword are silently skipped (the `UnknownSlideType`
///    error is the `layout` stage's responsibility, not this validator's).
///
/// All diagnostics are accumulated before returning — no bail-on-first (DI-018).
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_validator`].
///
/// ## Route B note
///
/// This validator uses the process-wide built-in `SlideTypeRegistry`. It does
/// **not** validate slide types registered only in the `PluginRegistry` (custom
/// plugin types). Route A (future story) will pass `Arc<PluginRegistry>` to validators
/// to cover externally-registered types.
pub struct FieldSchemaValidator;

impl Validator for FieldSchemaValidator {
    fn id(&self) -> &'static str {
        "field-schema"
    }

    /// Pre-layout pass: validates every slide in the deck against its declared
    /// field schema.
    ///
    /// Accumulates all diagnostics (E-VAL-101, E-VAL-102, W-VAL-103, E-VAL-104)
    /// without short-circuiting. Slides with unrecognised keywords are skipped.
    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        for slide in &deck.slides {
            let keyword = slide.slide_type.as_ref();
            if let Some(slide_type) = SLIDE_TYPE_REGISTRY.lookup_by_keyword(keyword) {
                diagnostics.extend(validate_fields(slide, slide_type));
            }
            // Slides with unknown keyword: silently skip.
            // The layout engine emits UnknownSlideType (E-LAY-NNN) for these.
        }

        diagnostics
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)] // BC-traceability IDs use uppercase: test_BC_S_SS_NNN_xxx
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DiagnosticSeverity, Validator, ValidatorOptions};
    use slideforge_types::{Deck, DeckMetadata, FieldValue, OrderedMap, Slide, SourceSpan, Value};

    use super::FieldSchemaValidator;

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

    fn make_slide(slide_type: &str, fields: Vec<(&str, Value)>) -> Slide {
        let mut field_map = OrderedMap::new();
        for (k, v) in fields {
            field_map.insert(Arc::from(k), FieldValue::Literal(v));
        }
        Slide {
            slide_type: Arc::from(slide_type),
            fields: field_map,
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

    // ── Validator ID ───────────────────────────────────────────────────────────

    /// Validator id must be "field-schema".
    #[test]
    fn test_BC_1_18_001_field_schema_validator_id() {
        assert_eq!(FieldSchemaValidator.id(), "field-schema");
    }

    // ── Missing required field → E-VAL-101 ────────────────────────────────────

    /// BC-1.18.001 / AC-009: `title` slide with no fields emits E-VAL-101 for
    /// the missing `title` field.
    #[test]
    fn test_BC_1_18_001_missing_required_field_emits_e_val_101() {
        let slide = make_slide("title", vec![]);
        let deck = make_deck(vec![slide]);
        let diags = FieldSchemaValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-VAL-101")
            .collect();
        assert!(
            !errors.is_empty(),
            "title slide with no fields must produce E-VAL-101; got {diags:?}"
        );
        assert_eq!(
            errors[0].severity,
            DiagnosticSeverity::Error,
            "E-VAL-101 must be Error severity"
        );
    }

    // ── Type mismatch on progress_bar.value → E-VAL-104 ───────────────────────

    /// BC-1.18.001 postcondition 2 / AC-009: `progress_bar` slide with
    /// `value "fifty"` (Str on an Int-typed field) emits E-VAL-104 T1.
    #[test]
    fn test_BC_1_18_001_progress_bar_str_value_emits_e_val_104() {
        let slide = make_slide(
            "progress_bar",
            vec![
                ("title", Value::Str(Arc::from("Progress"))),
                ("label", Value::Str(Arc::from("fifty percent"))),
                ("value", Value::Str(Arc::from("fifty"))), // Str instead of Int
            ],
        );
        let deck = make_deck(vec![slide]);
        let diags = FieldSchemaValidator.validate(&deck, &default_opts());
        let e104: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-VAL-104")
            .collect();
        assert!(
            !e104.is_empty(),
            "progress_bar with value=\"fifty\" must emit E-VAL-104; got {diags:?}"
        );
        let msg = e104[0].message.as_ref();
        assert!(
            msg.contains("integer"),
            "E-VAL-104 T1 message must say 'integer'; got: {msg}"
        );
        assert!(
            msg.contains("string"),
            "E-VAL-104 T1 message must say 'string'; got: {msg}"
        );
    }

    // ── Valid slide → no diagnostics ───────────────────────────────────────────

    /// BC-1.18.001 postcondition 1: valid `title` slide produces no Error diagnostics.
    #[test]
    fn test_BC_1_18_001_valid_title_slide_no_errors() {
        let slide = make_slide(
            "title",
            vec![("title", Value::Str(Arc::from("Hello World")))],
        );
        let deck = make_deck(vec![slide]);
        let diags = FieldSchemaValidator.validate(&deck, &default_opts());
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "valid title slide must produce no Error diagnostics; got {errors:?}"
        );
    }

    // ── Unknown slide type → no diagnostics ───────────────────────────────────

    /// Slides with an unrecognised keyword are silently skipped (Route B).
    /// The layout engine handles `UnknownSlideType`.
    #[test]
    fn test_BC_1_18_001_unknown_slide_type_skipped() {
        let slide = make_slide("no_such_slide_type", vec![]);
        let deck = make_deck(vec![slide]);
        let diags = FieldSchemaValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "unknown slide type must produce no diagnostics from FieldSchemaValidator; \
             got {diags:?}"
        );
    }

    // ── Empty deck → no diagnostics ───────────────────────────────────────────

    #[test]
    fn test_BC_1_18_001_empty_deck_no_diagnostics() {
        let deck = make_deck(vec![]);
        let diags = FieldSchemaValidator.validate(&deck, &default_opts());
        assert!(
            diags.is_empty(),
            "empty deck must produce no diagnostics; got {diags:?}"
        );
    }

    // ── Accumulation: multiple slides, each with issues ────────────────────────

    /// Multiple slides, each with type mismatches, produce diagnostics for ALL of
    /// them (DI-018: no bail-on-first).
    #[test]
    fn test_BC_1_18_001_accumulation_across_slides() {
        let s1 = make_slide(
            "progress_bar",
            vec![
                ("title", Value::Str(Arc::from("P1"))),
                ("label", Value::Str(Arc::from("L1"))),
                ("value", Value::Str(Arc::from("bad"))), // type mismatch
            ],
        );
        let s2 = make_slide("title", vec![]); // missing required field
        let deck = make_deck(vec![s1, s2]);
        let diags = FieldSchemaValidator.validate(&deck, &default_opts());
        let e104_count = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-VAL-104")
            .count();
        let e101_count = diags
            .iter()
            .filter(|d| d.code.as_ref() == "E-VAL-101")
            .count();
        assert!(
            e104_count >= 1,
            "must accumulate E-VAL-104 from progress_bar slide; got {diags:?}"
        );
        assert!(
            e101_count >= 1,
            "must accumulate E-VAL-101 from title slide with no fields; got {diags:?}"
        );
    }
}
